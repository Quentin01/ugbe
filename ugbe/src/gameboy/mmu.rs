#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MMU {
    boot_rom_enabled: bool,
}

impl MMU {
    pub fn new() -> Self {
        Self {
            boot_rom_enabled: true,
        }
    }
}

impl super::components::Mmu for MMU {
    fn read_byte(&self, ctx: &super::components::MMUContext, address: u16) -> u8 {
        match address {
            0x0..=0xFF if self.boot_rom_enabled => ctx.boot_rom[address as u8],
            0x0..=0x3FFF => ctx.cartridge.read_rom_bank_0(address),
            0x4000..=0x7FFF => ctx.cartridge.read_rom_bank_n(address - 0x4000),
            0x8000..=0x9FFF => ctx.ppu.read_vram_byte(address - 0x8000),
            0xA000..=0xBFFF => ctx.cartridge.read_ram(address - 0xA000),
            0xC000..=0xCFFF => ctx.work_ram[address - 0xC000],
            0xD000..=0xDFFF => ctx.work_ram[address - 0xD000 + 0x1000], // Banked in CGB
            0xE000..=0xEFFF => ctx.work_ram[address - 0xE000],
            0xFE00..=0xFE9F => ctx.ppu.read_oam_byte(address - 0xFE00),
            0xFF00 => ctx.joypad.read_p1(),
            0xFF04 => ctx.timer.read_div(),
            0xFF05 => ctx.timer.read_tima(),
            0xFF06 => ctx.timer.read_tma(),
            0xFF07 => ctx.timer.read_tac(),
            0xFF0F => ctx.interrupt.flags(),
            0xFF10 => ctx.spu.read_nr10(),
            0xFF11 => ctx.spu.read_nr11(),
            0xFF12 => ctx.spu.read_nr12(),
            0xFF13 => ctx.spu.read_nr13(),
            0xFF14 => ctx.spu.read_nr14(),
            0xFF15 => ctx.spu.read_nr20(),
            0xFF16 => ctx.spu.read_nr21(),
            0xFF17 => ctx.spu.read_nr22(),
            0xFF18 => ctx.spu.read_nr23(),
            0xFF19 => ctx.spu.read_nr24(),
            0xFF1A => ctx.spu.read_nr30(),
            0xFF1B => ctx.spu.read_nr31(),
            0xFF1C => ctx.spu.read_nr32(),
            0xFF1D => ctx.spu.read_nr33(),
            0xFF1E => ctx.spu.read_nr34(),
            0xFF1F => ctx.spu.read_nr40(),
            0xFF20 => ctx.spu.read_nr41(),
            0xFF21 => ctx.spu.read_nr42(),
            0xFF22 => ctx.spu.read_nr43(),
            0xFF23 => ctx.spu.read_nr44(),
            0xFF24 => ctx.spu.read_nr50(),
            0xFF25 => ctx.spu.read_nr51(),
            0xFF26 => ctx.spu.read_nr52(),
            0xFF30..=0xFF3F => ctx.spu.read_wav_ram(address - 0xFF30),
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
            0xFF80..=0xFFFE => ctx.high_ram[address - 0xFF80],
            0xFFFF => ctx.interrupt.enable(),
            _ => {
                // TODO: Handle missing memory mapping
                0xFF
            }
        }
    }

    fn write_byte(&mut self, ctx: &mut super::components::MMUContext, address: u16, value: u8) {
        match address {
            0x0..=0xFF if self.boot_rom_enabled => {}
            0x0..=0x7FFF => ctx.cartridge.write_rom(address, value),
            0x8000..=0x9FFF => ctx.ppu.write_vram_byte(address - 0x8000, value),
            0xA000..=0xBFFF => ctx.cartridge.write_ram(address - 0xA000, value),
            0xC000..=0xCFFF => ctx.work_ram[address - 0xC000] = value,
            0xD000..=0xDFFF => ctx.work_ram[address - 0xD000 + 0x1000] = value, // Banked in CGB
            0xE000..=0xEFFF => ctx.work_ram[address - 0xE000] = value,
            0xFE00..=0xFE9F => ctx.ppu.write_oam_byte(address - 0xFE00, value),
            0xFF00 => ctx.joypad.write_p1(value),
            0xFF04 => ctx.timer.write_div(value),
            0xFF05 => ctx.timer.write_tima(value),
            0xFF06 => ctx.timer.write_tma(value),
            0xFF07 => ctx.timer.write_tac(value),
            0xFF0F => ctx.interrupt.set_flags(value),
            0xFF10 => ctx.spu.write_nr10(value),
            0xFF11 => ctx.spu.write_nr11(value),
            0xFF12 => ctx.spu.write_nr12(value),
            0xFF13 => ctx.spu.write_nr13(value),
            0xFF14 => ctx.spu.write_nr14(value),
            0xFF15 => ctx.spu.write_nr20(value),
            0xFF16 => ctx.spu.write_nr21(value),
            0xFF17 => ctx.spu.write_nr22(value),
            0xFF18 => ctx.spu.write_nr23(value),
            0xFF19 => ctx.spu.write_nr24(value),
            0xFF1A => ctx.spu.write_nr30(value),
            0xFF1B => ctx.spu.write_nr31(value),
            0xFF1C => ctx.spu.write_nr32(value),
            0xFF1D => ctx.spu.write_nr33(value),
            0xFF1E => ctx.spu.write_nr34(value),
            0xFF1F => ctx.spu.write_nr40(value),
            0xFF20 => ctx.spu.write_nr41(value),
            0xFF21 => ctx.spu.write_nr42(value),
            0xFF22 => ctx.spu.write_nr43(value),
            0xFF23 => ctx.spu.write_nr44(value),
            0xFF24 => ctx.spu.write_nr50(value),
            0xFF25 => ctx.spu.write_nr51(value),
            0xFF26 => ctx.spu.write_nr52(value),
            0xFF30..=0xFF3F => ctx.spu.write_wav_ram(address - 0xFF30, value),
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
                    let value = self.read_byte(ctx, src + i);
                    ctx.ppu.write_oam_byte(i, value);
                }
            }
            0xFF47 => ctx.ppu.write_bgp(value),
            0xFF48 => ctx.ppu.write_obp0(value),
            0xFF49 => ctx.ppu.write_obp1(value),
            0xFF4A => ctx.ppu.write_wy(value),
            0xFF4B => ctx.ppu.write_wx(value),
            0xFF50 => self.boot_rom_enabled = value & 0x1 == 0x0,
            0xFF80..=0xFFFE => ctx.high_ram[address - 0xFF80] = value,
            0xFFFF => ctx.interrupt.set_enable(value),
            _ => {
                // TODO: Handle missing memory mapping
            }
        }
    }
}
