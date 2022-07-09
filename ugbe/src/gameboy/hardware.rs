mod bootrom;
mod ppu;

pub use bootrom::BootRom;

#[derive(Debug, Clone)]
pub struct Hardware {
    boot_rom: BootRom,
    ppu: ppu::Ppu,
    tmp_ram: [u8; 0x10000],
}

impl Hardware {
    pub fn new(boot_rom: BootRom) -> Self {
        Self {
            boot_rom,
            ppu: ppu::Ppu::default(),
            tmp_ram: [0; 0x10000],
        }
    }

    pub fn tick(&mut self) {
        // TODO: Tick the sub-devices
        self.ppu.tick()
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // TODO: Disable boot rom depending on a IO register
            0x0..=0xFF => self.boot_rom[address as u8],
            0x8000..=0x9FFF => self.ppu.read_vram_byte(address - 0x8000),
            0xFE00..=0xFE9F => self.ppu.read_oam_byte(address - 0xFE00),
            0xFF40 => self.ppu.read_lcdc(),
            // TODO: Handle memory correctly
            _ => {
                println!("Warning: Unsupported read at ${:04x}", address);
                self.tmp_ram[address as usize]
            }
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        u16::from_le_bytes([
            self.read_byte(address),
            self.read_byte(address.wrapping_add(1)),
        ])
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // TODO: Disable boot rom depending on a IO register
            0x0..=0xFF => {}
            0x8000..=0x9FFF => self.ppu.write_vram_byte(address - 0x8000, value),
            0xFE00..=0xFE9F => self.ppu.write_oam_byte(address - 0xFE00, value),
            0xFF40 => self.ppu.write_lcdc(value),
            // TODO: Handle memory correctly
            _ => {
                println!(
                    "Warning: Unsupported write of ${:02x} at ${:04x}",
                    value, address
                );
                self.tmp_ram[address as usize] = value
            }
        }
    }
}
