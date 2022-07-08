use std::ops::Index;

#[derive(Debug, Clone)]
pub struct BootRom(Vec<u8>);

impl BootRom {
    pub fn new(data: Vec<u8>) -> Self {
        BootRom(data)
    }
}

impl From<Vec<u8>> for BootRom {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl Index<u8> for BootRom {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}
