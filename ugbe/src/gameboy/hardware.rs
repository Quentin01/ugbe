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

#[derive(Debug, Clone)]
pub struct Hardware {
    boot_rom: BootRom,
}

impl Hardware {
    pub fn new(boot_rom: BootRom) -> Self {
        Self { boot_rom }
    }

    pub fn tick(&mut self) {
        // TODO: Tick the sub-devices
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0..=0xFF => self.boot_rom[address as u8],
            _ => todo!("Indexing after the boot rom"),
        }
    }
}
