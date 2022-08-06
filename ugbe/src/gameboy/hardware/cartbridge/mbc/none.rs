pub struct Mbc {
    ram: bool,
    battery_buffered_ram: bool,
}

impl Mbc {
    pub fn new(ram: bool, battery_buffered_ram: bool) -> Self {
        Self {
            ram,
            battery_buffered_ram,
        }
    }
}

impl super::Mbc for Mbc {
    fn has_ram(&self) -> bool {
        self.ram
    }

    fn ram_is_battery_buffered(&self) -> bool {
        self.battery_buffered_ram
    }

    fn has_rtc(&self) -> bool {
        false
    }

    fn has_rumble(&self) -> bool {
        false
    }

    fn read_rom_bank_0(&self, rom: &[u8], address: u16) -> u8 {
        let idx = address as usize;

        if idx < rom.len() {
            rom[idx as usize]
        } else {
            0xFF
        }
    }

    fn read_rom_bank_n(&self, rom: &[u8], address: u16) -> u8 {
        let idx = address as usize + 0x4000;

        if idx < rom.len() {
            rom[idx as usize]
        } else {
            0xFF
        }
    }

    fn write_rom(&mut self, _: &[u8], _: u16, _: u8) {}

    fn read_ram(&self, ram: &[u8], address: u16) -> u8 {
        let idx = address as usize;

        if idx < ram.len() {
            ram[idx as usize]
        } else {
            0xFF
        }
    }

    fn write_ram(&self, ram: &mut [u8], address: u16, value: u8) {
        let idx = address as usize;

        if idx < ram.len() {
            ram[idx as usize] = value
        }
    }

    fn str(&self) -> std::borrow::Cow<'static, str> {
        (if self.ram && self.battery_buffered_ram {
            "MBC None (Battery-buffered RAM)"
        } else if self.ram {
            "MBC None (RAM)"
        } else {
            "MBC None (No RAM)"
        })
        .into()
    }
}
