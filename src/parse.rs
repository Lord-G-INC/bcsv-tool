use crate::*;

use binrw::prelude::*;
use binrw::Endian;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::Cursor;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::mem::size_of;

pub fn parsebcsv<A: AsRef<[u8]>>(buffer: A, endian: Endian) 
-> DynamicResult<(BCSV, Cursor<Vec<u8>>)> {
    let vec = Vec::<u8>::from(buffer.as_ref());
    let mut stream = Cursor::new(vec);
    let bcsv = stream.read_type(endian)?;
    Ok((bcsv, stream))
}

pub fn read_field<'a, T: BinRead, R: BinReaderExt>(bcsv: &BCSV, column: usize, row: u32,
    stream: &mut R, endian: Endian) -> DynamicResult<T>
    where T::Args<'a> : Default + Clone {
    let field = &bcsv.fields[column];
    let off = (bcsv.header.entrydataoff + row *
        bcsv.header.entrysize + field.dataoff as u32) as u64;
    let result = stream.seektask(SeekFrom::Start(off), |x| {
        x.read_type::<T>(endian)
    })?;
    Ok(result?)
}

pub fn read_stringoff<R: SeekTask + BufRead>(bcsv: &BCSV, column: usize, row: u32,
    stream: &mut R, endian: Endian) -> DynamicResult<String> {
    let off: u32 = read_field(bcsv, column, row, stream, endian)?;
    let stringoff = (bcsv.header.entrydataoff + bcsv.header.entrycount*bcsv.header.entrysize)
    as u64;
    let realoff = off as u64 + stringoff;
    let result = stream.seektask(SeekFrom::Start(realoff), |x| {
        let mut buf = vec![];
        x.read_until(0, &mut buf).unwrap();
        if buf.len() > 1 {
            buf.remove(buf.len() - 1);
        }
        String::from(String::from_utf8_lossy(&buf))
    })?;
    Ok(result)
}

pub fn read_hashes<P: AsRef<Path>>(path: P) -> DynamicResult<HashMap<u32, String>> {
    let text = std::fs::read_to_string(path)?;
    let mut result = HashMap::new();
    for line in text.split('\n') {
        if line.starts_with('#') {
            continue;
        }
        let hash = calchash(line);
        result.insert(hash, String::from(line));
    }
    Ok(result)
}

pub fn calchash(text: &str) -> u32 {
    let mut output = 0u32;
    for char in text.bytes() {
        output = (char as u32).wrapping_add(output.wrapping_mul(0x1F));
    }
    output
}

pub fn parsecsv<P: AsRef<Path>>(path: P) -> DynamicResult<String> {
    let text = std::fs::read_to_string(path)?;
    let chars = text.chars().collect::<Vec<_>>();
    let mut allowed = vec![];
    for i in 0..chars.len() {
        if chars[i] != '\r' {
            allowed.push(chars[i]);
        }
    }
    Ok(allowed.into_iter().map(|x| String::from(x)).collect::<Vec<_>>().join(""))
}

pub fn generatefields(text: &mut String, bcsv: &mut BCSV) {
    let mut s = String::from(&text[0..text.find('\n').unwrap_or_default()]);
    *text = String::from(&text[text.find('\n').unwrap_or_default()+1..]);
    let mut names = vec![];
    while let Some(pos) = s.find(',') {
        let token = &s[..pos];
        names.push(String::from(token));
        s.replace_range(..pos+1, "");
    }
    names.push(s);
    bcsv.fields.reserve_exact(names.len());
    for name in &names {
        let mut field = Field::default();
        let datatype = &name[name.find(':').unwrap_or_default()+1..];
        let datatype = datatype.parse::<u8>().unwrap_or_default();
        let n = &name[0..name.find(':').unwrap_or_default()];
        if n.starts_with("0x") {
            let hash = u32::from_str_radix(&n[2..], 16).unwrap_or_default();
            field.hash = hash;
        } else {
            field.hash = calchash(n);
        }
        field.datatype = unsafe { std::mem::transmute(datatype) };
        bcsv.fields.push(field);
    }
    let mut doff = 0;
    let mut clone = bcsv.fields.clone();
    clone.sort_by(|x, y| x.hash.cmp(&y.hash));
    for mut f in clone {
        f.dataoff = doff;
        doff += f.datatype.getdtsize();
        let og = bcsv.fields.iter_mut().find(|x| x.hash == f.hash).unwrap();
        og.dataoff = f.dataoff;
    }
    bcsv.header.fieldcount = bcsv.fields.len() as u32;
    bcsv.header.entrysize = doff as u32;
    let dataoff = size_of::<Header>() + size_of::<Field>() * bcsv.fields.len();
    bcsv.header.entrydataoff = dataoff as u32;
}

pub fn generatevalues(mut text: String, bcsv: &mut BCSV) -> Vec<Vec<String>> {
    let mut result = vec![];
    while let Some(pos) = text.find('\n') {
        let mut vec = vec![];
        let mut line = String::from(&text[..pos]);
        while let Some(lpos) = line.find(',') { 
            vec.push(String::from(&line[..lpos]));
            line.replace_range(..lpos+1, "");
        }
        vec.push(line);
        result.push(vec);
        text.replace_range(..pos+1, "");
    }
    if !text.is_empty() {
        let mut vec = vec![];
        while let Some(pos) = text.find(',') {
            vec.push(String::from(&text[..pos]));
            text.replace_range(..pos+1, "");
        }
        vec.push(text);
        result.push(vec);
    }
    bcsv.header.entrycount = result.len() as u32;
    result
}

pub fn createstringtable(vals: &mut Vec<Vec<String>>, bcsv: &mut BCSV) -> String {
    let mut result = String::new();
    let mut offs = HashMap::<String, usize>::new();
    let mut off = 0;
    for row in 0..vals.len() {
        for i in 0..bcsv.fields.len() {
            let f = &bcsv.fields[i];
            if f.datatype == DataType::STRINGOFF {
                let str = &vals[row][i];
                let val = format!("{}\0", str);
                if !offs.contains_key(&val) {
                    result.push_str(&val);
                    offs.insert(val.clone(), off);
                    off += str.len() + 1;
                }
                vals[row][i] = format!("{}", offs[&val]);
            }
        }
    }
    result
}

pub fn createbuffer(table: String, bcsv: &mut BCSV) -> DynamicResult<Cursor<Vec<u8>>> {
    let stringoff = bcsv.header.entrydataoff as usize + bcsv.header.entrysize as usize 
    * bcsv.header.entrycount as usize;
    let mut size = table.len() + stringoff;
    size += (size + 31 & !31) - size;
    let buffer = vec![0x40; size];
    let mut stream = Cursor::new(buffer);
    stream.set_position(stringoff as u64);
    stream.write_all(table.as_bytes())?;
    stream.set_position(0);
    Ok(stream)
}

pub fn fillheaderandfields(stream: &mut Cursor<Vec<u8>>, bcsv: &mut BCSV, endian: Endian) 
-> DynamicResult<u64> {
    bcsv.write_options(stream, endian, ())?;
    Ok(stream.position())
}

pub fn fillfieldtable(stream: &mut Cursor<Vec<u8>>, dataoff: u64, vals: Vec<Vec<String>>, bcsv: BCSV,
    endian: Endian) -> DynamicResult<()> {
        for row in 0..vals.len() {
            for i in 0..bcsv.fields.len() {
                let f = &bcsv.fields[i];
                let msg = &vals[row][i];
                let off = dataoff + row as u64 * bcsv.header.entrysize as u64 + f.dataoff as u64;
                use DataType::*;
                match f.datatype {
                    LONG |
                    LONG2 |
                    STRINGOFF => {
                        let num = u32::from_str_radix(msg, 10)?;
                        stream.seektask(SeekFrom::Start(off), |x| {
                            num.write_options(x, endian, ())
                        })??;
                    }
                    SHORT => {
                        let num = u16::from_str_radix(msg, 10)?;
                        stream.seektask(SeekFrom::Start(off), |x| {
                            num.write_options(x, endian, ())
                        })??;
                    }
                    FLOAT => {
                        let num = msg.parse::<f32>()?;
                        stream.seektask(SeekFrom::Start(off), |x| {
                            num.write_options(x, endian, ())
                        })??;
                    }
                    CHAR => {
                        let num = i8::from_str_radix(msg, 10)?;
                        stream.seektask(SeekFrom::Start(off), |x| {
                            num.write_options(x, endian, ())
                        })??;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
}