use std::collections::HashMap;
use std::io::BufRead;

use crate::*;

use binrw::prelude::*;
use binrw::Endian;

pub fn convert_to_csv<R: BinReaderExt + BufRead>(bcsv: BCSV, stream: &mut R,
    hashes: HashMap<u32, String>, endian: Endian) -> DynamicResult<String> {
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
        text.push_str(&format!("{}", bcsv.fields[i].datatype as u8));
        if !last {
            text.push(',');
        } else {
            text.push_str("\n");
        }
    }
    for row in 0..bcsv.header.entrycount {
        for i in 0..bcsv.fields.len() {
            let last = i == bcsv.fields.len() - 1;
            let f = &bcsv.fields[i];
            use DataType::*;
            match f.datatype {
                LONG |
                LONG2 => {
                    let mut val: u32 = read_field(&bcsv, i, row, stream, endian)?;
                    val = (val & f.mask) >> f.shift as u32;
                    val |= match val & 0x80000000 {
                        0x80000000 => 0,
                        _ => val
                    };
                    let txt = format!("{}", val) + match last { false => ",", true => "\n" };
                    text.push_str(&txt);
                    
                }
                CHAR => {
                    let val: u8 = read_field(&bcsv, i, row, stream, endian)?;
                    let mut val = val as i32;
                    val >>= f.shift as i32;
                    val |= match val & 0x80 {
                        0x80 => 0,
                        _ => val
                    };
                    let txt = format!("{}", val) + match last { false => ",", true => "\n" };
                    text.push_str(&txt);
                }
                SHORT => {
                    let mut val: u16 = read_field(&bcsv, i, row, stream, endian)?;
                    val = (val & f.mask as u16) >> f.shift as u16;
                    val |= match val & 0x8000 {
                        0x8000 => 0,
                        _ => val
                    };
                    let txt = format!("{}", val) + match last { false => ",", true => "\n" };
                    text.push_str(&txt);
                }
                FLOAT => {
                    let val: f32 = read_field(&bcsv, i, row, stream, endian)?;
                    let txt = format!("{}", val) + match last { false => ",", true => "\n" };
                    text.push_str(&txt);
                }
                STRINGOFF => {
                    let str = read_stringoff(&bcsv, i, row, stream, endian)?;
                    let txt = str + match last { false => ",", true => "\n" };
                    text.push_str(&txt);
                }
                _ => {}
            };
        }
    }
    Ok(text)
}