pub mod bootrom;
pub mod cartbridge;
pub mod interrupt;
pub mod ppu;
pub mod wram;

#[derive(Debug)]
pub struct Hardware {
    boot_rom: bootrom::BootRom,
    boot_rom_enabled: bool,
    cartbridge: cartbridge::Cartbridge,
    ppu: ppu::Ppu,
    interrupt: interrupt::Interrupt,
    work_ram: wram::WorkRam<0x1000>,
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
            interrupt: interrupt::Interrupt::new(),
            work_ram: wram::WorkRam::new(),
            tmp_ram: [0; 0x10000],
        }
    }

    pub fn tick(&mut self) {
        // TODO: Tick the sub-devices
        self.ppu.tick(&mut self.interrupt)
    }
}

impl super::mmu::Mmu for Hardware {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0..=0xFF if self.boot_rom_enabled => self.boot_rom[address as u8],
            0x0..=0x3FFF => self.cartbridge.read_rom_bank0(address - 0x0),
            0x4000..=0x7FFF => self.cartbridge.read_rom_bankN(address - 0x4000),
            0x8000..=0x9FFF => self.ppu.read_vram_byte(address - 0x8000),
            0xA000..=0xBFFF => self.cartbridge.read_ram(address - 0xA000),
            0xC000..=0xCFFF => self.work_ram[address - 0xC000],
            0xE000..=0xEFFF => self.work_ram[address - 0xE000],
            0xFE00..=0xFE9F => self.ppu.read_oam_byte(address - 0xFE00),
            0xFF0F => self.interrupt.flags(),
            0xFF40 => self.ppu.read_lcdc(),
            0xFF41 => self.ppu.read_stat(),
            0xFF42 => self.ppu.read_scy(),
            0xFF43 => self.ppu.read_scx(),
            0xFF44 => self.ppu.read_ly(),
            0xFF45 => self.ppu.read_lyc(),
            0xFF46 => {
                println!("DMA read");
                0xFF
            }
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
            }
            0xFFFF => self.interrupt.enable(),
            // TODO: Handle memory correctly
            _ => {
                // println!("Warning: Unsupported read at ${:04x}", address);
                self.tmp_ram[address as usize]
            }
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0..=0xFF if self.boot_rom_enabled => {} // Writing into the boot ROM
            0x0..=0x7FFF => {}                        // Writing into the cartbridge ROM
            0x8000..=0x9FFF => self.ppu.write_vram_byte(address - 0x8000, value),
            0xA000..=0xBFFF => self.cartbridge.write_ram(address - 0xA000, value),
            0xC000..=0xCFFF => self.work_ram[address - 0xC000] = value,
            0xE000..=0xEFFF => self.work_ram[address - 0xE000] = value,
            0xFE00..=0xFE9F => self.ppu.write_oam_byte(address - 0xFE00, value),
            0xFF0F => self.interrupt.set_flags(value),
            0xFF40 => self.ppu.write_lcdc(value),
            0xFF41 => self.ppu.write_stat(value),
            0xFF42 => self.ppu.write_scy(value),
            0xFF43 => self.ppu.write_scx(value),
            0xFF44 => {} // We can't write to LY
            0xFF45 => self.ppu.write_lyc(value),
            0xFF46 => {
                let src = (value as u16) * 0x100;

                // TODO: Properly do the DMA
                for i in 0..=0x9f {
                    self.write_byte(0xFE00 + i, self.read_byte(src + i));
                }
            }
            0xFF47 => self.ppu.write_bgp(value),
            0xFF48 => self.ppu.write_obp0(value),
            0xFF49 => self.ppu.write_obp1(value),
            0xFF4A => self.ppu.write_wy(value),
            0xFF4B => self.ppu.write_wx(value),
            0xFF50 => self.boot_rom_enabled = value & 0x1 == 0x0,
            0xFFFF => self.interrupt.set_enable(value),
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

impl super::interrupt::Line for Hardware {
    fn highest_priority(&self) -> Option<super::interrupt::Kind> {
        self.interrupt.highest_priority()
    }

    fn ack(&mut self, kind: super::interrupt::Kind) {
        self.interrupt.ack(kind)
    }

    fn request(&mut self, kind: super::interrupt::Kind) {
        self.interrupt.request(kind)
    }

    fn flags_not_empty(&self) -> bool {
        self.interrupt.flags_not_empty()
    }
}
