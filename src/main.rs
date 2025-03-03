use clap::*;
use libbcsv::*;
use libbcsv::Endian as BinRWEndian;
use std::{io::Cursor, path::*};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Endian {
    Big,
    Little
}

impl Endian {
    #[cfg(target_endian = "big")]
    const NATIVE : Endian = Endian::Big;
    #[cfg(target_endian = "little")]
    const NATIVE : Endian = Endian::Little;
}

impl From<Endian> for BinRWEndian {
    fn from(value: Endian) -> Self {
        match value {
            Endian::Big => Self::Big,
            Endian::Little => Self::Little
        }
    }
}

impl std::ops::Not for Endian {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Big => Self::Little,
            Self::Little => Self::Big
        }
    }
}

#[derive(Debug, Default, Clone, Parser)]
#[command(version)]
#[command(about="Converts bcsv files to and from csv.")]
struct ProgArgs {
    /// The file to convert from. (Supported: *.csv, any bcsv extension)
    pub infile: PathBuf,
    /// The file to convert to. (Supported: *.csv, *.xlsx, any bcsv extension)
    pub outfile: PathBuf,
    #[arg(short, long)]
    /// The hash lookup file to use, 
    /// 
    /// this SHOULD be provided when doing bcsv to csv,
    /// but isn't required.
    pub lookup: Option<PathBuf>,
    #[arg(short, long)]
    /// The Endian to use for parsing. Defaults to the reverse Native endian.
    pub endian: Option<Endian>,
    #[arg(short, long)]
    /// If enabled, the SHORT and CHAR datatypes will be UNSIGNED.
    pub unsigned: bool,
    #[arg(short, long, default_value_t = ',')]
    /// Delimiter for CSV reading/writing
    pub delim: char
}


fn main() -> Result<(), BcsvError> {
    let args = ProgArgs::parse();
    let inpath = &args.infile;
    let outpath = &args.outfile;
    let lookup = &args.lookup;
    let signed = !args.unsigned;
    let delim = args.delim;
    let endian = args.endian.unwrap_or(!Endian::NATIVE);
    if let Some(inext) = inpath.extension() {
        if inext.to_string_lossy() == "csv" {
            // csv to bcsv, extension for output not checked because bcsv has a few oddball extensions
            let bcsv = csv_parse::CSV::from_path(inpath, delim)?;
            let data = bcsv.to_bytes(endian.into())?;
            std::fs::write(outpath, data)?;
        }
        if let Some(oext) = outpath.extension() {
            if oext == inext {
                return Err("Extensions cannot match.".into());
            }
            if oext.to_string_lossy() == "csv" {
                // bcsv to csv
                let data = std::fs::read(inpath)?;
                let mut reader = Cursor::new(data);
                let mut bcsv = types::BCSV::new();
                bcsv.read(&mut reader, endian.into())?;
                let hashes = lookup.as_ref()
                .map(|x| hash::read_hashes(x).unwrap_or_default()).unwrap_or_default();
                bcsv.hash_table = hashes;
                let text = bcsv.convert_to_csv(signed, delim);
                std::fs::write(outpath, text)?;
                return Ok(());
            } else if oext.to_string_lossy() == "xlsx" {
                // bcsv to xlsx
                let data = std::fs::read(inpath)?;
                let mut reader = Cursor::new(data);
                let mut bcsv = types::BCSV::new();
                bcsv.read(&mut reader, endian.into())?;
                let hashes = lookup.as_ref()
                .map(|x| hash::read_hashes(x).unwrap_or_default()).unwrap_or_default();
                bcsv.hash_table = hashes;
                bcsv.convert_to_xlsx(outpath.as_os_str().to_string_lossy(), signed)?;
                return Ok(());
            }
        }
    }
    // There are bcsv files with no extension, so this has to happen..
    if let Some(oext) = outpath.extension() {
        if oext.to_string_lossy() == "csv" {
            // bcsv to csv
            let data = std::fs::read(inpath)?;
            let mut reader = Cursor::new(data);
            let mut bcsv = types::BCSV::new();
            bcsv.read(&mut reader, endian.into())?;
            let hashes = lookup.as_ref()
            .map(|x| hash::read_hashes(x).unwrap_or_default()).unwrap_or_default();
            bcsv.hash_table = hashes;
            let text = bcsv.convert_to_csv(signed, delim);
            std::fs::write(outpath, text)?;
        } else if oext.to_string_lossy() == "xlsx" {
            // bcsv to xlsx
            let data = std::fs::read(inpath)?;
            let mut reader = Cursor::new(data);
            let mut bcsv = types::BCSV::new();
            bcsv.read(&mut reader, endian.into())?;
            let hashes = lookup.as_ref()
            .map(|x| hash::read_hashes(x).unwrap_or_default()).unwrap_or_default();
            bcsv.hash_table = hashes;
            bcsv.convert_to_xlsx(outpath.as_os_str().to_string_lossy(), signed)?;
        }
    }
    Ok(())
}