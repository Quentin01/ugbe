pub mod bootrom;
pub mod cartbridge;
pub mod ppu;

#[derive(Debug)]
pub struct Hardware {
    boot_rom: bootrom::BootRom,
    boot_rom_enabled: bool,
    cartbridge: cartbridge::Cartbridge,
    ppu: ppu::Ppu,
    tmp_ram: [u8; 0x10000],
}

impl Hardware {
    pub fn new(
        boot_rom: bootrom::BootRom,
        cartbridge: cartbridge::Cartbridge,
        renderer: Option<Box<dyn ppu::screen::Renderer>>,
    ) -> Self {
        Self {
            boot_rom,
            boot_rom_enabled: true,
            cartbridge,
            ppu: ppu::Ppu::new(renderer),
            tmp_ram: [0; 0x10000],
        }
    }

    pub fn tick(&mut self) {
        // TODO: Tick the sub-devices
        self.ppu.tick()
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0..=0xFF if self.boot_rom_enabled => self.boot_rom[address as u8],
            0x0..=0x3FFF => self.cartbridge.read_rom_bank0(address - 0x0),
            0x4000..=0x7FFF => self.cartbridge.read_rom_bankN(address - 0x4000),
            0x8000..=0x9FFF => self.ppu.read_vram_byte(address - 0x8000),
            0xA000..=0xBFFF => self.cartbridge.read_ram(address - 0xA000),
            0xFE00..=0xFE9F => self.ppu.read_oam_byte(address - 0xFE00),
            0xFF40 => self.ppu.read_lcdc(),
            0xFF41 => self.ppu.read_stat(),
            0xFF42 => self.ppu.read_scy(),
            0xFF43 => self.ppu.read_scx(),
            0xFF44 => self.ppu.read_ly(),
            0xFF45 => self.ppu.read_lyc(),
            0xFF46 => todo!("DMA"),
            0xFF47 => self.ppu.read_bgp(),
            0xFF48 => self.ppu.read_obp0(),
            0xFF49 => self.ppu.read_obp1(),
            0xFF4A => self.ppu.read_wy(),
            0xFF4B => self.ppu.read_wx(),
            0xFF50 => {
                if self.boot_rom_enabled {
                    0xFF
                } else {
                    0xFE
                }
            },
            // TODO: Handle memory correctly
            _ => {
                // println!("Warning: Unsupported read at ${:04x}", address);
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
            0x0..=0xFF if self.boot_rom_enabled => {}, // Writing into the boot ROM
            0x0..=0x7FFF => {}, // Writing into the cartbridge ROM
            0x8000..=0x9FFF => self.ppu.write_vram_byte(address - 0x8000, value),
            0xA000..=0xBFFF => self.cartbridge.write_ram(address - 0xA000, value),
            0xFE00..=0xFE9F => self.ppu.write_oam_byte(address - 0xFE00, value),
            0xFF40 => self.ppu.write_lcdc(value),
            0xFF41 => self.ppu.write_stat(value),
            0xFF42 => self.ppu.write_scy(value),
            0xFF43 => self.ppu.write_scx(value),
            0xFF44 => {} // We can't write to LY
            0xFF45 => self.ppu.write_lyc(value),
            0xFF46 => todo!("DMA"),
            0xFF47 => self.ppu.write_bgp(value),
            0xFF48 => self.ppu.write_obp0(value),
            0xFF49 => self.ppu.write_obp1(value),
            0xFF4A => self.ppu.write_wy(value),
            0xFF4B => self.ppu.write_wx(value),
            0xFF50 => self.boot_rom_enabled = value & 0x1 == 0x1,
            // TODO: Handle memory correctly
            _ => {
                // println!(
                //     "Warning: Unsupported write of ${:02x} at ${:04x}",
                //     value, address
                // );
                self.tmp_ram[address as usize] = value
            }
        }
    }
}
