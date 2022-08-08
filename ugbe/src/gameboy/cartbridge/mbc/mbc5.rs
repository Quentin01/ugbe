pub struct Mbc {
    ram: bool,
    battery_buffered_ram: bool,
    rumble: bool,

    ram_enabled: bool,
    rom_bank_n: u16,
    ram_bank_n: u8,

    rumble_enabled: bool,
}

impl Mbc {
    pub fn new(ram: bool, battery_buffered_ram: bool, rumble: bool) -> Self {
        Self {
            ram,
            battery_buffered_ram,
            rumble,

            ram_enabled: false,
            rom_bank_n: 1,
            ram_bank_n: 0,

            rumble_enabled: false,
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
        self.rumble
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
        let rom_bank_n = self.rom_bank_n as usize;
        let idx = address as usize + 0x4000 * rom_bank_n;

        if idx < rom.len() {
            rom[idx as usize]
        } else {
            0xFF
        }
    }

    fn write_rom(&mut self, rom: &[u8], address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = value & 0xF == 0xA;
            }
            0x2000..=0x2FFF => {
                let nb_banks = (rom.len() / (16 * 1024)) as u16;

                let [msb, _] = self.rom_bank_n.to_be_bytes();
                self.rom_bank_n = u16::from_be_bytes([msb, value]) % nb_banks;
            }
            0x3000..=0x3FFF => {
                let nb_banks = (rom.len() / (16 * 1024)) as u16;

                let [_, lsb] = self.rom_bank_n.to_be_bytes();
                self.rom_bank_n = u16::from_be_bytes([value & 0b1, lsb]) % nb_banks;
            }
            0x4000..=0x5FFF => {
                if self.rumble {
                    self.ram_bank_n = value & 0b0111;
                    self.rumble_enabled = (value >> 3) & 0b1 == 1;
                } else {
                    self.ram_bank_n = value & 0b1111;
                }
            }
            _ => {}
        }
    }

    fn read_ram(&self, ram: &[u8], address: u16) -> u8 {
        let ram_bank_n = self.ram_bank_n as usize;
        let idx = address as usize + 0x2000 * ram_bank_n;

        if idx < ram.len() {
            ram[idx as usize]
        } else {
            0xFF
        }
    }

    fn write_ram(&self, ram: &mut [u8], address: u16, value: u8) {
        let ram_bank_n = self.ram_bank_n as usize;
        let idx = address as usize + 0x2000 * ram_bank_n;

        if idx < ram.len() {
            ram[idx as usize] = value
        }
    }

    fn str(&self) -> std::borrow::Cow<'static, str> {
        (if self.ram && self.battery_buffered_ram {
            "MBC5 (Battery-buffered RAM)"
        } else if self.ram {
            "MBC5 (RAM)"
        } else {
            "MBC5 (No RAM)"
        })
        .into()
    }
}
