mod types;
mod parse;
mod seektask;
mod convert;

pub(crate) type DynamicBoxError = Box<dyn std::error::Error>;
pub(crate) type DynamicResult<T> = Result<T, DynamicBoxError>;
pub(crate) use types::*;
pub(crate) use seektask::*;
pub(crate) use parse::*;
use std::path::Path;
use binrw::Endian;
use convert::convert_to_csv;

fn main() -> DynamicResult<()> {
    let envargs = std::env::args().collect::<Vec<String>>();
    let endian = Endian::Big;
    for arg in envargs.into_iter().skip(1) {
        let path = Path::new(&arg);
        let ext = path.extension().unwrap_or_default().to_string_lossy();
        if ext == "bcsv" || ext == "tbl" || ext == "banmt" || ext == "" {
            let buffer = std::fs::read(path)?;
            let (bcsv, mut stream) = parsebcsv(buffer, endian)?;
            let hashes = read_hashes("lookup_supermariogalaxy.txt")?;
            let text = convert_to_csv(bcsv, &mut stream, hashes, endian)?;
            let new = path.to_path_buf().with_extension("csv");
            std::fs::write(new, text)?;
        } else if ext == "csv" {
            let mut bcsv = BCSV::default();
            let mut text = parsecsv(path)?;
            generatefields(&mut text, &mut bcsv);
            let mut vals = generatevalues(text, &mut bcsv);
            let table = createstringtable(&mut vals, &mut bcsv);
            let mut stream = createbuffer(table, &mut bcsv)?;
            let dataoff = fillheaderandfields(&mut stream, &mut bcsv, endian)?;
            fillfieldtable(&mut stream, dataoff, vals, bcsv, endian)?;
            let new = path.to_path_buf().with_extension("bcsv");
            std::fs::write(new, stream.into_inner())?;
        }
    }
    Ok(())
}