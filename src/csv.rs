use std::path::Path;
use std::collections::HashMap;
use crate::*;
use crate::types::*;

pub fn parsecsv<P: AsRef<Path>>(path: P) -> BinResult<String> {
    let text = std::fs::read_to_string(path)?;
    let mut result = String::new();
    for char in text.chars() {
        if char != '\r' {
            result.push(char);
        }
    }
    Ok(result)
}

#[derive(Debug, Default, Clone)]
pub struct CSV {
    header: Header,
    fields: Vec<Field>,
    values: Vec<Vec<String>>,
    text: String
}

impl CSV {
    pub fn new<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        let mut result = Self::default();
        let text = parsecsv(path)?;
        let mut s = String::from(&text[0..text.find('\n').unwrap_or_default()]);
        result.text = String::from(&text[text.find('\n').unwrap_or_default()+1..]);
        let mut names = vec![];
        while let Some(pos) = s.find(',') {
            let token = &s[..pos];
            names.push(String::from(token));
            s.replace_range(..pos+1, "");
        }
        names.push(s);
        result.fields.reserve_exact(names.len());
        for name in &names {
            let mut field = Field::new();
            let datatype = &name[name.find(':').unwrap_or_default()+1..];
            let datatype = datatype.parse::<u8>().unwrap_or_default();
            let n = &name[0..name.find(':').unwrap_or_default()];
            if n.starts_with("0x") {
                let hash = u32::from_str_radix(&n[2..], 16).unwrap_or_default();
                field.hash = hash;
            } else {
                field.hash = hash::calchash(n);
            }
            field.datatype = datatype;
            result.fields.push(field);
        }
        let mut doff = 0;
        let mut clone = result.fields.clone();
        clone.sort_by(|x, y| x.hash.cmp(&y.hash));
        for mut f in clone {
            f.dataoff = doff;
            doff += f.getdtsize();
            let og = result.fields.iter_mut().find(|x| x.hash == f.hash).unwrap();
            og.dataoff = f.dataoff;
        }
        result.header.fieldcount = result.fields.len() as u32;
        result.header.entrysize = doff as u32;
        let off = 16 + 12 * result.header.fieldcount;
        result.header.entrydataoff = off;
        result.generate_values();
        Ok(result)
    }
    fn generate_values(&mut self) {
        let Self {text, values, header, ..} = self;
        while let Some(pos) = text.find('\n') {
            let mut vec = vec![];
            let mut line = String::from(&text[..pos]);
            while let Some(lpos) = line.find(',') {
                vec.push(String::from(&line[..lpos]));
                line.replace_range(..lpos+1, "");
            }
            vec.push(line);
            values.push(vec);
            text.replace_range(..pos+1, "");
        }
        if !text.is_empty() {
            let mut vec = vec![];
            while let Some(pos) = text.find(',') {
                vec.push(String::from(&text[..pos]));
                text.replace_range(..pos+1, "");
            }
            vec.push(text.clone());
            values.push(vec);
        }
        header.entrycount = values.len() as u32;
    }
    /// Creates the stringtable using the fields as our helpers.
    /// 
    /// Stringtable rules:
    /// - Strings are unique (appear ONE time only)
    /// - Strings are nul terminated (c style string)
    pub fn create_stringtable(&mut self) -> String {
        let mut result = String::new();
        let mut offs = HashMap::new();
        let mut off = 0;
        let Self {values, fields, ..} = self;
        for row in 0..values.len() {
            for i in 0..fields.len() {
                let f = &fields[i];
                if f.is_stringoff() {
                    let str = &values[row][i];
                    let val = format!("{}\0", str);
                    if !offs.contains_key(&val) {
                        result.push_str(&val);
                        offs.insert(val.clone(), off);
                        off += str.len() + 1;
                    }
                    values[row][i] = format!("{}", offs[&val]);
                }
            }
        }
        result
    }
    /// Generates a bcsv using info from the csv.
    pub fn generate_bcsv(&self) -> BCSV {
        BCSV { header: self.header, fields: self.fields.clone(), values: Vec::new() }
    }
    /// NOTE: Stringoffs are handled elsewhere.
    /// This method also CONSUMES the CSV struct as it is no longer needed once this method is called.
    pub fn create_values(self, bcsv: &mut BCSV){
        let Self {values, ..} = self;
        for row in 0..values.len() {
            for i in 0..bcsv.fields.len() {
                let f = &bcsv.fields[i];
                let val = &values[row][i];
                match f.datatype {
                    0 | 3 | 6 => bcsv.values.push(Value::LONG(val.parse().unwrap_or_default())),
                    1 => bcsv.values.push(Value::STRING(val.as_bytes().try_into().unwrap_or_default())),
                    2 => bcsv.values.push(Value::FLOAT(val.parse().unwrap_or_default())),
                    4 => bcsv.values.push(Value::SHORT(val.parse().unwrap_or_default())),
                    5 => bcsv.values.push(Value::CHAR(val.parse().unwrap_or_default())),
                    _ => {}
                }
            }
        }
    }
}