mod bootrom;

pub use bootrom::BootRom;

#[derive(Debug, Clone)]
pub struct Hardware {
    boot_rom: BootRom,
    tmp_ram: [u8; 0x10000],
}

impl Hardware {
    pub fn new(boot_rom: BootRom) -> Self {
        Self { boot_rom, tmp_ram: [0; 0x10000] }
    }

    pub fn tick(&mut self) {
        // TODO: Tick the sub-devices
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // TODO: Disable boot rom depending on a IO register
            0x0..=0xFF => self.boot_rom[address as u8],
            // TODO: Handle memory correctly
            _ => self.tmp_ram[address as usize],
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        u16::from_le_bytes([self.read_byte(address), self.read_byte(address.wrapping_add(1))])
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // TODO: Disable boot rom depending on a IO register
            0x0..=0xFF => {},
            // TODO: Handle memory correctly
            _ => self.tmp_ram[address as usize] = value,
        }
    }
}
