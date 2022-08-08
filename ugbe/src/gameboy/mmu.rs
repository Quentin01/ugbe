#[derive(Debug)]
pub struct Mmu {
    boot_rom_enabled: bool,
    tmp_ram: [u8; 0x10000],
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            boot_rom_enabled: true,
            tmp_ram: [0; 0x10000],
        }
    }
}

impl super::components::Mmu for Mmu {
    fn read_byte(&self, ctx: &super::components::MmuContext, address: u16) -> u8 {
        match address {
            0x0..=0xFF if self.boot_rom_enabled => ctx.boot_rom[address as u8],
            0x0..=0x3FFF => ctx.cartbridge.read_rom_bank_0(address - 0x0),
            0x4000..=0x7FFF => ctx.cartbridge.read_rom_bank_n(address - 0x4000),
            0x8000..=0x9FFF => ctx.ppu.read_vram_byte(address - 0x8000),
            0xA000..=0xBFFF => ctx.cartbridge.read_ram(address - 0xA000),
            0xC000..=0xCFFF => ctx.work_ram[address - 0xC000],
            0xE000..=0xEFFF => ctx.work_ram[address - 0xE000],
            0xFE00..=0xFE9F => ctx.ppu.read_oam_byte(address - 0xFE00),
            0xFF00 => 0xCF, // TODO: Joypad input
            0xFF04 => ctx.timer.read_div(),
            0xFF05 => ctx.timer.read_tima(),
            0xFF06 => ctx.timer.read_tma(),
            0xFF07 => ctx.timer.read_tac(),
            0xFF0F => ctx.interrupt.flags(),
            0xFF40 => ctx.ppu.read_lcdc(),
            0xFF41 => ctx.ppu.read_stat(),
            0xFF42 => ctx.ppu.read_scy(),
            0xFF43 => ctx.ppu.read_scx(),
            0xFF44 => ctx.ppu.read_ly(),
            0xFF45 => ctx.ppu.read_lyc(),
            0xFF46 => {
                // TODO: Properly do the DMA
                0xFF
            }
            0xFF47 => ctx.ppu.read_bgp(),
            0xFF48 => ctx.ppu.read_obp0(),
            0xFF49 => ctx.ppu.read_obp1(),
            0xFF4A => ctx.ppu.read_wy(),
            0xFF4B => ctx.ppu.read_wx(),
            0xFF50 => {
                if self.boot_rom_enabled {
                    0xFF
                } else {
                    0xFE
                }
            }
            0xFFFF => ctx.interrupt.enable(),
            // TODO: Handle missing memory mapping
            _ => self.tmp_ram[address as usize],
        }
    }

    fn write_byte(&mut self, ctx: &mut super::components::MmuContext, address: u16, value: u8) {
        if address == 0xFF01 {
            // Write to Serial transfer data SB
            print!("{}", value as char);
            // println!("Write to SB: {0x{:02x}}", value);
        } else if address == 0xFF02 {
            // Write to serial transfer control SC
            // println!("Write to SC: 0x{:02x}", value);
        }

        match address {
            0x0..=0xFF if self.boot_rom_enabled => {}
            0x0..=0x7FFF => ctx.cartbridge.write_rom(address, value),
            0x8000..=0x9FFF => ctx.ppu.write_vram_byte(address - 0x8000, value),
            0xA000..=0xBFFF => ctx.cartbridge.write_ram(address - 0xA000, value),
            0xC000..=0xCFFF => ctx.work_ram[address - 0xC000] = value,
            0xE000..=0xEFFF => ctx.work_ram[address - 0xE000] = value,
            0xFE00..=0xFE9F => ctx.ppu.write_oam_byte(address - 0xFE00, value),
            0xFF04 => ctx.timer.write_div(value),
            0xFF05 => ctx.timer.write_tima(value),
            0xFF06 => ctx.timer.write_tma(value),
            0xFF07 => ctx.timer.write_tac(value),
            0xFF0F => ctx.interrupt.set_flags(value),
            0xFF40 => ctx.ppu.write_lcdc(value),
            0xFF41 => ctx.ppu.write_stat(value),
            0xFF42 => ctx.ppu.write_scy(value),
            0xFF43 => ctx.ppu.write_scx(value),
            0xFF44 => ctx.ppu.write_ly(value),
            0xFF45 => ctx.ppu.write_lyc(value),
            0xFF46 => {
                // TODO: Properly do the DMA

                let src = (value as u16) * 0x100;
                for i in 0..=0x9f {
                    self.write_byte(ctx, 0xFE00 + i, self.read_byte(ctx, src + i));
                }
            }
            0xFF47 => ctx.ppu.write_bgp(value),
            0xFF48 => ctx.ppu.write_obp0(value),
            0xFF49 => ctx.ppu.write_obp1(value),
            0xFF4A => ctx.ppu.write_wy(value),
            0xFF4B => ctx.ppu.write_wx(value),
            0xFF50 => self.boot_rom_enabled = value & 0x1 == 0x0,
            0xFFFF => ctx.interrupt.set_enable(value),
            // TODO: Handle missing memory mapping
            _ => self.tmp_ram[address as usize] = value,
        }
    }
}
