use crate::*;

use std::io::{SeekFrom, Read, Seek, Write};

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
pub struct Header {
    /// Amount of entries a field has.
    pub entrycount: u32,
    /// Amount of fields within the BCSV.
    pub fieldcount: u32,
    /// Exact position where entries begin, should AWLAYS be after field data.
    pub entrydataoff: u32,
    /// Total size of a entry row, should be the sum of all fields datatype size.
    pub entrysize: u32
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
pub struct Field {
    pub hash: u32,
    pub mask: u32,
    pub dataoff: u16,
    pub shift: u8,
    pub datatype: u8
}

impl Field {
    pub const fn getdtsize(&self) -> u16 {
        match self.datatype {
            0 | 2 | 3 | 6 => 4,
            1 => 32,
            4 => 2,
            5 => 1,
            _ => 0
        }
    }
    pub const fn is_stringoff(&self) -> bool {
        self.datatype == 6
    }
    pub const fn new() -> Self {
        Self { hash: 0, mask: u32::MAX, dataoff: 0, shift: 0, datatype: 0 }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    LONG(u32),
    STRING([u8; 32]),
    FLOAT(f32),
    SHORT(u16),
    CHAR(u8),
    STRINGOFF(String)
}

impl Value {
    pub const fn is_stringoff(&self) -> bool {
        use Value::*;
        match self {
            STRINGOFF(_) => true,
            _ => false
        }
    }
    /// Reads a string off, reader position **NEEDS** to be the start of the stringtable.
    /// 
    /// Will match to `Value::STRINGOFF` on success
    pub fn read_string_off<R: BinReaderExt>(reader: &mut R, off: i64) -> BinResult<Value> {
        let mut bytes = vec![];
        let pos = reader.seek(SeekFrom::Current(0))?;
        reader.seek(SeekFrom::Current(off))?;
        let mut byte = 1u8;
        while byte != 0 {
            byte = reader.read_ne()?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
        reader.seek(SeekFrom::Start(pos))?;
        Ok(Value::STRINGOFF(String::from_utf8_lossy(&bytes).into()))
    }
    pub fn write_value<W: BinWriterExt>(&self, writer: &mut W, endian: Endian) -> BinResult<()> {
        match self {
            Value::LONG(l) => writer.write_type(l, endian),
            Value::STRING(s) => writer.write_ne(s),
            Value::FLOAT(f) => writer.write_type(f, endian),
            Value::SHORT(s) => writer.write_type(s, endian),
            Value::CHAR(c) => writer.write_ne(c),
            _ => Ok(())
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::LONG(0)
    }
}

impl BinRead for Value {
    type Args<'a> = (&'a Field, &'a mut Vec<u32>, i64, u32);
    fn read_options<R: Read + Seek>(
            reader: &mut R,
            endian: Endian,
            args: Self::Args<'_>,
        ) -> BinResult<Self> {
            let (field, stroffs, row, entrysize) = args;
            let off = row * entrysize as i64 + field.dataoff as i64;
            let pos = reader.seek(SeekFrom::Current(0))?;
            reader.seek(SeekFrom::Current(off))?;
            let res = match field.datatype {
                0 | 3 => Ok(Value::LONG(reader.read_type(endian)?)),
                1 => Ok(Value::STRING(reader.read_ne()?)),
                4 => Ok(Value::FLOAT(reader.read_type(endian)?)),
                5 => Ok(Value::CHAR(reader.read_ne()?)),
                6 => {
                    stroffs.push(reader.read_type(endian)?);
                    Ok(Value::STRINGOFF(String::default()))
                }
                _ => Err(binrw::Error::NoVariantMatch { pos: 112 })
            };
            reader.seek(SeekFrom::Start(pos))?;
            res
    }
}

#[derive(Debug, Default, Clone)]
pub struct BCSV {
    pub header: Header,
    pub fields: Vec<Field>,
    pub values: Vec<Value>
}

impl BinRead for BCSV {
    type Args<'a> = ();
    fn read_options<R: Read + Seek>(
            reader: &mut R,
            endian: Endian,
            _: Self::Args<'_>,
        ) -> BinResult<Self> {
        let mut result = Self::default();
        {
        let Self {header, fields, ..} = &mut result;
        *header = reader.read_type(endian)?;
        *fields = vec![Field::default(); header.fieldcount as usize];
        for i in 0..fields.len() {
            fields[i] = reader.read_type(endian)?;
        }
        }
        // SAFETY: position needs to be entrydataoff.
        reader.seek(SeekFrom::Start(result.header.entrydataoff as u64))?;
        let entrysize = result.header.entrycount as usize * result.fields.len();
        let mut v = 0;
        let mut entryoffs = vec![];
        let mut row = 0;
        while v != entrysize {
            if v >= entrysize {
                break;
            }
            for field in &result.fields {
                let args = 
                (field, &mut entryoffs, row, result.header.entrysize);
                result.values.push(Value::read_options(reader, endian, args)?);
                v += 1;
            }
            row += 1;
        }
        // SAFETY: reader needs to be at start of stringtable
        let stringoff = (result.header.entrydataoff +
            result.header.entrycount*result.header.entrysize) as u64;
        reader.seek(SeekFrom::Start(stringoff))?;
        let pos = result.values.iter()
        .enumerate().filter(|(_, x)| x.is_stringoff())
        .map(|(x, _)| x).collect::<Vec<_>>();
        for i in 0..pos.len() {
            let off = entryoffs[i] as i64;
            result.values[pos[i]] = Value::read_string_off(reader, off)?;
        }
        Ok(result)
    }
}

impl BinWrite for BCSV {
    type Args<'a> = ();
    fn write_options<W: Write + Seek>(
            &self,
            writer: &mut W,
            endian: Endian,
            _: Self::Args<'_>,
        ) -> BinResult<()> {
            writer.write_type(&self.header, endian)?;
            for field in &self.fields {
                writer.write_type(field, endian)?;
            }
            let mut v = 0;
            while v != self.values.len() {
                if v == self.values.len() {
                    break;
                }
                for i in 0..self.fields.len() {
                    let f = &self.fields[i];
                    let val = &self.values[v];
                    let pos = writer.seek(SeekFrom::Current(0))?;
                    writer.seek(SeekFrom::Current(f.dataoff as i64))?;
                    val.write_value(writer, endian)?;
                    writer.seek(SeekFrom::Start(pos))?;
                    v += 1;
                }
                writer.seek(SeekFrom::Current(self.header.entrysize as i64))?;
            }
            Ok(())
    }
}