pub use binrw::prelude::*;
pub use binrw::Endian;
use clap::*;

use std::io::Cursor;
use std::path::*;

mod hash;
mod convert;
mod csv;
mod types;

#[derive(Debug, Default, Clone, Parser)]
#[command(version)]
#[command(about="Converts bcsv files to and from csv.")]
struct ProgArgs {
    #[arg(short, long)]
    /// The file to convert from
    pub infile: String,
    #[arg(short, long)]
    /// The file to convert to
    pub outfile: String,
    #[arg(short, long)]
    /// The hash lookup file to use
    pub lookup: Option<String>,
    #[arg(short, long)]
    /// If enabled, will use the OPPOSITE endian to the system's endian.
    pub endian: bool,
    /// The mask to use for values, by default this is u32::MAX.
    #[arg(short, long, default_value_t = u32::MAX)]
    pub mask: u32
}


fn main() -> binrw::BinResult<()> {
    let args = ProgArgs::parse();
    let inpath = Path::new(&args.infile);
    let endian = match args.endian {
        false => Endian::NATIVE,
        true => match Endian::NATIVE {
            Endian::Big => Endian::Little,
            Endian::Little => Endian::Big
        }
    };
    let ext = inpath.extension().unwrap_or_default().to_string_lossy().to_string();
    if ext == "bcsv" || ext == "tbl" || ext == "banmt" || ext == "" || ext == "pa" {
        let mut buffer = Cursor::new(std::fs::read(inpath)?);
        let hashes = hash::read_hashes(&args.lookup.unwrap())?;
        let bcsv = types::BCSV::read_options(&mut buffer, endian, ())?;
        let csv = convert::convert_to_csv(bcsv, &hashes);
        std::fs::write(args.outfile, csv)?;
    } else if ext == "csv" {
        let csv = csv::CSV::new(inpath)?;
        let bcsv = convert::convert_to_bcsv(csv, endian, args.mask)?;
        std::fs::write(args.outfile, bcsv)?;
    }
    Ok(())
}