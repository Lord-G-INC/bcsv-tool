use clap::*;
use libbcsv::*;
use libbcsv::binrw::prelude::*;
use std::io::Cursor;
use std::path::*;

#[derive(Debug, Default, Clone, Parser)]
#[command(version)]
#[command(about="Converts bcsv files to and from csv.")]
struct ProgArgs {
    #[arg(short, long)]
    /// The file to convert from.
    pub infile: String,
    #[arg(short, long)]
    /// The file to convert to.
    pub outfile: String,
    #[arg(short, long)]
    /// The hash lookup file to use, 
    /// 
    /// this SHOULD be provided when doing bcsv -> csv,
    /// but isn't required.
    pub lookup: Option<String>,
    #[arg(short, long)]
    /// If enabled, use the OPPOSITE endian to the system's endian.
    /// 
    /// Little Endian becomes Big Endian, and vice versa.
    pub endian: bool,
    /// The mask to use for values, by default this is u32::MAX.
    #[arg(short, long, default_value_t = u32::MAX)]
    pub mask: u32
}


fn main() -> Result<(), BcsvError> {
    let args = ProgArgs::parse();
    let inpath = Path::new(&args.infile);
    let outpath = Path::new(&args.outfile);
    let endian = match args.endian {
        false => Endian::NATIVE,
        true => match Endian::NATIVE {
            Endian::Big => Endian::Little,
            Endian::Little => Endian::Big
        }
    };
    let ext = inpath.extension().unwrap_or_default().to_string_lossy().to_string();
    let oext = outpath.extension().unwrap_or_default().to_string_lossy().to_string();
    if ext == "bcsv" || ext == "tbl" || ext == "banmt" || ext == "" || ext == "pa" {
        let mut buffer = Cursor::new(std::fs::read(inpath)?);
        let hashes = args.lookup
        .map(|x| hash::read_hashes(x).ok())
        .unwrap_or(None).unwrap_or_default();
        let bcsv = types::BCSV::read_options(&mut buffer, endian, ())?;
        if oext == "csv" {
            let csv = bcsv.convert_to_csv(hashes);
            std::fs::write(args.outfile, csv)?;
        } else if oext == "xlsx" {
            bcsv.convert_to_xlsx(hashes, args.outfile)?;
        }
    } else if ext == "csv" {
        let csv = csv::CSV::new(inpath)?;
        let bcsv = csv.convert_to_bcsv(endian, args.mask)?;
        std::fs::write(args.outfile, bcsv)?;
    }
    Ok(())
}