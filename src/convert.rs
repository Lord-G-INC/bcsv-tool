use std::collections::HashMap;
use crate::*;
use crate::types::*;
use std::io::{Cursor, Write};

pub fn convert_to_csv(bcsv: BCSV, hashes: &HashMap<u32, String>) -> String {
    let mut text = String::new();
    let mut names = vec![];
    for field in &bcsv.fields {
        let hash = field.hash;
        if hashes.contains_key(&hash) {
            names.push(hashes[&hash].clone());
        } else {
            names.push(format!("0x{:x}", hash));
        }
    }
    for i in 0..names.len() {
        let last = i == names.len() - 1;
        text.push_str(&names[i]);
        text.push(':');
        text.push_str(&bcsv.fields[i].datatype.to_string());
        if !last {
            text.push(',');
        } else {
            text.push('\n');
        }
    }
    let mut v = 0;
    while v < bcsv.values.len() {
        for i in 0..bcsv.fields.len() {
            let last = i == bcsv.fields.len() - 1;
            let shift = bcsv.fields[i].shift;
            let mask = bcsv.fields[i].mask;
            match &bcsv.values[v] {
                Value::LONG(l) => {
                    let mut l = *l;
                    l = (l & mask) >> shift as u32;
                    l |= match l & 0x80000000 {
                        0x80000000 => 0,
                        _ => l
                    };
                    let txt = format!("{}", l) + match last { false => ",", true => "\n" };
                    text.push_str(&txt);
                }
                Value::STRING(s) => {
                    text.push_str(&String::from(String::from_utf8_lossy(s)));
                    text.push(match last { false => ',', true => '\n' });
                }
                Value::FLOAT(f) => {
                    let txt = format!("{}", f) + match last { false => ",", true => "\n" };
                    text.push_str(&txt);
                }
                Value::SHORT(sh) => {
                    let mut sh = *sh;
                    sh = (sh & mask as u16) >> shift as u16;
                    sh |= match sh & 0x8000 {
                        0x8000 => 0,
                        _ => sh
                    };
                    let txt = format!("{}", sh) + match last { false => ",", true => "\n" };
                    text.push_str(&txt);
                }
                Value::CHAR(c) => {
                    let mut c = *c as u32;
                    c >>= shift as u32;
                    c |= match c & 0x80 {
                        0x80 => 0,
                        _ => c
                    };
                    let txt = format!("{}", c) + match last { false => ",", true => "\n" };
                    text.push_str(&txt);
                }
                Value::STRINGOFF(st) => {
                    text.push_str(st);
                    text.push(match last { false => ',', true => '\n' });
                }
            }
            v += 1;
        }
    }
    text
}

pub fn convert_to_bcsv(mut csv: csv::CSV, endian: Endian, mask: u32) -> BinResult<Vec<u8>> {
    let mut bcsv = csv.generate_bcsv();
    let mut buffer = Cursor::new(vec![]);
    let table = csv.create_stringtable();
    csv.create_values(&mut bcsv);
    for field in &mut bcsv.fields {
        field.mask = mask;
    }
    bcsv.write_options(&mut buffer, endian, ())?;
    let stroff = bcsv.header.entrydataoff as usize + bcsv.header.entrysize as usize *
        bcsv.header.entrycount as usize;
    let mut size = table.len() + stroff;
    size += (size + 31 & !31) - size;
    buffer.get_mut().resize(size, 0x40);
    buffer.write_all(table.as_bytes())?;
    Ok(buffer.into_inner())
}