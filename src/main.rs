use clap::*;
use libbcsv::*;
use std::{io::Cursor, path::*};

#[derive(Debug, Default, Clone, Parser)]
#[command(version)]
#[command(about="Converts bcsv files to and from csv.")]
struct ProgArgs {
    #[arg(short, long)]
    /// The file to convert from. (Supported: *.csv, any bcsv extension)
    pub infile: String,
    #[arg(short, long)]
    /// The file to convert to. (Supported: *.csv, *.xlsx, any bcsv extension)
    pub outfile: String,
    #[arg(short, long)]
    /// The hash lookup file to use, 
    /// 
    /// this SHOULD be provided when doing bcsv to csv,
    /// but isn't required.
    pub lookup: Option<String>,
    #[arg(short, long)]
    /// If enabled, use the OPPOSITE endian to the system's endian.
    /// 
    /// Little Endian becomes Big Endian, and vice versa.
    pub endian: bool
}


fn main() -> Result<(), BcsvError> {
    let args = ProgArgs::parse();
    let inpath = Path::new(&args.infile);
    let outpath = Path::new(&args.outfile);
    let lookup = &args.lookup;
    let endian = match args.endian {
        false => Endian::NATIVE,
        true => match Endian::NATIVE {
            Endian::Big => Endian::Little,
            Endian::Little => Endian::Big
        }
    };
    if let Some(inext) = inpath.extension() {
        if inext.to_string_lossy().ends_with("csv") {
            // csv to bcsv, extension for output not checked because bcsv has a few oddball extensions
            let csv = csv_parse::CSV::from_path(inpath)?;
            let data = csv.create_bcsv().to_bytes(endian)?;
            std::fs::write(outpath, data)?;
        }
        if let Some(oext) = outpath.extension() {
            if oext == inext {
                return Err("Extensions cannot match.".into());
            }
            if oext.to_string_lossy().ends_with("csv") {
                // bcsv to csv
                let data = std::fs::read(inpath)?;
                let mut reader = Cursor::new(data);
                let mut bcsv = types::BCSV::new();
                bcsv.read(&mut reader, endian)?;
                let hashes = lookup.as_ref()
                .map(|x| hash::read_hashes(x).unwrap_or_default()).unwrap_or_default();
                let text = bcsv.convert_to_csv(&hashes);
                std::fs::write(outpath, text)?;
            } else if oext.to_string_lossy().ends_with("xlsx") {
                // bcsv to xlsx
                let data = std::fs::read(inpath)?;
                let mut reader = Cursor::new(data);
                let mut bcsv = types::BCSV::new();
                bcsv.read(&mut reader, endian)?;
                let hashes = lookup.as_ref()
                .map(|x| hash::read_hashes(x).unwrap_or_default()).unwrap_or_default();
                bcsv.convert_to_xlsx(outpath.as_os_str().to_string_lossy(), &hashes)?;
            }
        }
    }
    // There are bcsv files with no extension, so this has to happen..
    if let Some(oext) = outpath.extension() {
        if oext.to_string_lossy().ends_with("csv") {
            // bcsv to csv
            let data = std::fs::read(inpath)?;
            let mut reader = Cursor::new(data);
            let mut bcsv = types::BCSV::new();
            bcsv.read(&mut reader, endian)?;
            let hashes = lookup.as_ref()
            .map(|x| hash::read_hashes(x).unwrap_or_default()).unwrap_or_default();
            let text = bcsv.convert_to_csv(&hashes);
            std::fs::write(outpath, text)?;
        } else if oext.to_string_lossy().ends_with("xlsx") {
            // bcsv to xlsx
            let data = std::fs::read(inpath)?;
            let mut reader = Cursor::new(data);
            let mut bcsv = types::BCSV::new();
            bcsv.read(&mut reader, endian)?;
            let hashes = lookup.as_ref()
            .map(|x| hash::read_hashes(x).unwrap_or_default()).unwrap_or_default();
            bcsv.convert_to_xlsx(outpath.as_os_str().to_string_lossy(), &hashes)?;
        }
    }
    Ok(())
}