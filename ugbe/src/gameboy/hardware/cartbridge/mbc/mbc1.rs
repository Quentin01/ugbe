pub struct Mbc {
    ram: bool,
    battery_buffered_ram: bool,

    multi_cart: bool,

    ram_enabled: bool,
    rom_bank_n: u8,
    ram_bank_n: u8,
    mode: bool,
}

const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

impl Mbc {
    pub fn new(rom: &[u8], ram: bool, battery_buffered_ram: bool) -> Self {
        let multi_cart = if rom.len() != 1 * 1024 * 1024 {
            false
        } else {
            let nintendo_logo_count = (0..4)
                .map(|idx| {
                    let start = idx * 0x40000 + 0x0104;
                    let end = start + NINTENDO_LOGO.len();

                    &rom[start..end]
                })
                .filter(|&possible_logo| possible_logo == NINTENDO_LOGO)
                .count();

            // From mooneye: A multicart should have at least two games + a menu with valid logo data
            nintendo_logo_count >= 3
        };

        Self {
            ram,
            battery_buffered_ram,

            multi_cart,

            ram_enabled: false,
            rom_bank_n: 1,
            ram_bank_n: 0,
            mode: false,
        }
    }

    fn zero_bank_n(&self, rom: &[u8]) -> usize {
        if rom.len() < 1 * 1024 * 1024 {
            // ROM is < 1MB
            0
        } else if rom.len() == 1 * 1024 * 1024 {
            // ROM is 1MB
            if self.multi_cart {
                ((self.ram_bank_n & 0b11) << 4) as usize
            } else {
                ((self.ram_bank_n & 0b1) << 5) as usize
            }
        } else if rom.len() == 2 * 1024 * 1024 {
            // ROM is 2MB
            ((self.ram_bank_n & 0b11) << 5) as usize
        } else {
            panic!("Invalid ROM size of {} for MBC1", rom.len());
        }
    }

    fn high_bank_n(&self, rom: &[u8]) -> usize {
        if rom.len() < 1 * 1024 * 1024 {
            // ROM is < 1MB
            self.rom_bank_n as usize
        } else if rom.len() == 1 * 1024 * 1024 {
            // ROM is 1MB
            if self.multi_cart {
                (((self.ram_bank_n & 0b11) << 4) | (self.rom_bank_n & 0b11001111)) as usize
            } else {
                (((self.ram_bank_n & 0b1) << 5) | (self.rom_bank_n & 0b11011111)) as usize
            }
        } else if rom.len() == 2 * 1024 * 1024 {
            // ROM is 2MB
            (((self.ram_bank_n & 0b11) << 5) | (self.rom_bank_n & 0b10011111)) as usize
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

                if value & 0b11111 == 0 {
                    self.rom_bank_n = 1;
                } else {
                    self.rom_bank_n = value % nb_banks;
                }
            }
            0x4000..=0x5FFF => {
                self.ram_bank_n = value & 0b11;
            }
            0x6000..=0x7FFF => self.mode = value & 0b1 == 0b1,
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

        let extras = if self.ram && self.battery_buffered_ram {
            "Battery-buffered RAM"
        } else if self.ram {
            "RAM"
        } else {
            "No RAM"
        };

        format!("{} ({})", mbc_name, extras).into()
    }
}
