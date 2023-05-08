use binrw::prelude::*;

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
pub struct Header {
    pub entrycount: u32,
    pub fieldcount: u32,
    pub entrydataoff: u32,
    pub entrysize: u32
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite, PartialEq, Eq)]
#[brw(repr(u8))]
pub enum DataType {
    #[default] LONG = 0x0,
    STRING = 0x1,
    FLOAT = 0x2,
    LONG2 = 0x3,
    SHORT = 0x4,
    CHAR = 0x5,
    STRINGOFF = 0x6,
}

impl DataType {
    pub const fn getdtsize(&self) -> u16 {
        use DataType::*;
        match self {
            LONG |
            LONG2 |
            FLOAT |
            STRINGOFF => 4,
            STRING => 32,
            SHORT => 2,
            CHAR => 1
        }
    }
}

#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
pub struct Field {
    pub hash: u32,
    pub mask: u32,
    pub dataoff: u16,
    pub shift: u8,
    pub datatype: DataType
}

impl Default for Field {
    fn default() -> Self {
        Self { mask: u32::MAX, hash: 0, dataoff: 0, shift: 0, datatype: DataType::LONG }
    }
}

#[derive(Debug, Default, Clone, BinRead, BinWrite)]
pub struct BCSV {
    pub header: Header,
    #[br(count = header.fieldcount as usize)]
    pub fields: Vec<Field>,
}