#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MBC {
    ram: bool,
    battery: bool,

    multi_cart: bool,

    ram_enabled: bool,
    rom_bank_n: u8,
    ram_bank_n: u8,
    mode: bool,
}

impl MBC {
    pub fn new(ram: bool, battery: bool, multi_cart: bool) -> Self {
        Self {
            ram,
            battery,

            multi_cart,

            ram_enabled: false,
            rom_bank_n: 1,
            ram_bank_n: 0,
            mode: false,
        }
    }

    fn zero_bank_n(&self, rom: &[u8]) -> usize {
        if rom.len() < 1024 * 1024 {
            // ROM is < 1MB
            0
        } else if rom.len() == 1024 * 1024 {
            // ROM is 1MB
            if self.multi_cart {
                ((self.ram_bank_n & 0b0011) << 4) as usize
            } else {
                ((self.ram_bank_n & 0b0001) << 5) as usize
            }
        } else if rom.len() == 2 * 1024 * 1024 {
            // ROM is 2MB
            ((self.ram_bank_n & 0b0011) << 5) as usize
        } else {
            panic!("Invalid ROM size of {} for MBC1", rom.len());
        }
    }

    fn high_bank_n(&self, rom: &[u8]) -> usize {
        if rom.len() < 1024 * 1024 {
            // ROM is < 1MB
            self.rom_bank_n as usize
        } else if rom.len() == 1024 * 1024 {
            // ROM is 1MB
            if self.multi_cart {
                (((self.ram_bank_n & 0b0011) << 4) | (self.rom_bank_n & 0b1100_1111)) as usize
            } else {
                (((self.ram_bank_n & 0b0001) << 5) | (self.rom_bank_n & 0b1101_1111)) as usize
            }
        } else if rom.len() == 2 * 1024 * 1024 {
            // ROM is 2MB
            (((self.ram_bank_n & 0b0011) << 5) | (self.rom_bank_n & 0b1001_1111)) as usize
        } else {
            panic!("Invalid ROM size of {} for MBC1", rom.len());
        }
    }

    fn ram_idx(&self, ram: &[u8], address: u16) -> usize {
        if ram.len() == 2 * 1024 || ram.len() == 8 * 1024 {
            // RAM is 2KB or 8KB
            address as usize % ram.len()
        } else if ram.len() == 32 * 1024 {
            // RAM is 32KB
            if self.mode {
                let ram_bank_n = self.ram_bank_n as usize;
                address as usize + 0x2000 * ram_bank_n
            } else {
                address as usize
            }
        } else {
            panic!("Invalid RAM size of {} for MBC1", ram.len());
        }
    }
}

impl super::MBC for MBC {
    fn has_ram(&self) -> bool {
        self.ram
    }

    fn ram_is_battery_buffered(&self) -> bool {
        self.battery
    }

    fn has_rtc(&self) -> bool {
        false
    }

    fn has_rumble(&self) -> bool {
        false
    }

    fn read_rom_bank_0(&self, rom: &[u8], address: u16) -> u8 {
        let idx = if self.mode {
            let zero_bank_n = self.zero_bank_n(rom);
            address as usize + 0x4000 * zero_bank_n
        } else {
            address as usize
        };

        if idx < rom.len() {
            rom[idx as usize]
        } else {
            0xFF
        }
    }

    fn read_rom_bank_n(&self, rom: &[u8], address: u16) -> u8 {
        let high_bank_n = self.high_bank_n(rom);
        let idx = address as usize + 0x4000 * high_bank_n;

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
            0x2000..=0x3FFF => {
                let nb_banks = (rom.len() / (16 * 1024)) as u8;

                if value & 0b0001_1111 == 0 {
                    self.rom_bank_n = 1;
                } else {
                    self.rom_bank_n = value % nb_banks;
                }
            }
            0x4000..=0x5FFF => {
                self.ram_bank_n = value & 0b0011;
            }
            0x6000..=0x7FFF => self.mode = value & 0b001 != 0,
            _ => {}
        }
    }

    fn read_ram(&self, ram: &[u8], address: u16) -> u8 {
        let idx = self.ram_idx(ram, address);

        if self.ram_enabled {
            ram[idx as usize]
        } else {
            0xFF
        }
    }

    fn write_ram(&self, ram: &mut [u8], address: u16, value: u8) {
        let idx = self.ram_idx(ram, address);

        if self.ram_enabled {
            ram[idx as usize] = value
        }
    }

    fn str(&self) -> std::borrow::Cow<'static, str> {
        let mbc_name = if self.multi_cart { "MBC1M" } else { "MBC1" };

        let extras = if self.ram && self.battery {
            "Battery-buffered RAM"
        } else if self.ram {
            "RAM"
        } else {
            "No RAM"
        };

        format!("{} ({})", mbc_name, extras).into()
    }
}
