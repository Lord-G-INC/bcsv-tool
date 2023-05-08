use std::io::Seek;
use std::io::SeekFrom;

pub trait SeekTask : Seek {
    fn seektask<R, F: FnMut(&mut Self) -> R>(&mut self, pos: SeekFrom, func : F) -> std::io::Result<R>;
}

impl<S: Seek> SeekTask for S {
    fn seektask<R, F: FnMut(&mut Self) -> R>(&mut self, pos: SeekFrom, mut func : F) -> std::io::Result<R> {
        let opos = self.seek(SeekFrom::Current(0))?;
        self.seek(pos)?;
        let result = func(self);
        self.seek(SeekFrom::Start(opos))?;
        Ok(result)
    }
}