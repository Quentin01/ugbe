use std::ops::Index;
use std::{fs, io, path::Path};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("file have an invalid size (got '{0}', expected '{1}')")]
    InvalidSize(usize, usize),

    #[error("failed to read the file")]
    ReadError(#[from] io::Error),
}

#[derive(Debug, Clone)]
pub struct BootRom([u8; 0x100]);

impl BootRom {
    pub fn from_path<P: ?Sized + AsRef<Path>>(path: &P) -> Result<Self, Error> {
        let file = fs::File::open(path)?;
        let mut reader = io::BufReader::new(file);
        let mut buffer = Vec::new();

        io::Read::read_to_end(&mut reader, &mut buffer)?;

        if buffer.len() != 0x100 {
            return Err(Error::InvalidSize(buffer.len(), 0x100));
        }

        // We unwrap as we already checked the size before
        Ok(Self(buffer.try_into().unwrap()))
    }
}

impl Index<u8> for BootRom {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}
