mod mbc;

pub struct Cartbridge {
    title: String,
    mbc: Box<dyn mbc::Mbc>,
    rom: Vec<u8>,
    ram: Option<Vec<u8>>,
}

impl std::fmt::Debug for Cartbridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cartbridge")
            .field("title", &self.title)
            .field("mbc", &self.mbc.str())
            .field("rom", &self.rom)
            .field("ram", &self.ram)
            .finish()
    }
}

impl Cartbridge {
    pub fn read_rom_bank_0(&self, address: u16) -> u8 {
        self.mbc.read_rom_bank_0(&self.rom, address)
    }

    pub fn read_rom_bank_n(&self, address: u16) -> u8 {
        self.mbc.read_rom_bank_n(&self.rom, address)
    }

    pub fn write_rom(&mut self, address: u16, value: u8) {
        self.mbc.write_rom(&self.rom, address, value)
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        match self.ram.as_ref() {
            Some(ram) => self.mbc.read_ram(ram, address),
            None => 0xFF,
        }
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        match self.ram.as_mut() {
            Some(ram) => self.mbc.write_ram(ram, address, value),
            None => {}
        }
    }
}

impl From<Vec<u8>> for Cartbridge {
    fn from(data: Vec<u8>) -> Self {
        let title = &data[0x134..=0x143];
        let title_size = title
            .iter()
            .position(|&c| c == b'\0')
            .unwrap_or(title.len());
        let title = String::from_utf8_lossy(&title[0..title_size]);
        println!("ROM title: {}", title);

        #[allow(clippy::identity_op)]
        let rom_size = match data[0x148] {
            0x00 => 32 * 1024,                    // 32KB
            0x01 => 64 * 1024,                    // 64KB
            0x02 => 128 * 1024,                   // 128KB
            0x03 => 256 * 1024,                   // 256KB
            0x04 => 512 * 1024,                   // 512KB
            0x05 => 1 * 1024 * 1024,              // 1MB
            0x06 => 2 * 1024 * 1024,              // 2MB
            0x07 => 4 * 1024 * 1024,              // 4MB
            0x08 => 8 * 1024 * 1024,              // 8MB
            0x52 => 1 * 1024 * 1024 + 128 * 1024, // 1.1MB
            0x53 => 1 * 1024 * 1024 + 256 * 1024, // 1.2MB
            0x54 => 1 * 1024 * 1024 + 512 * 1024, // 1.5MB
            o => todo!("Not supported ROM size ${:02x}", o),
        };

        if data.len() != rom_size {
            panic!("ROM file different that the internal reported ROM size");
        }

        let ram_size = match data[0x149] {
            0x00 => 0,          // No RAM
            0x01 => 2 * 1024,   // 2KB
            0x02 => 8 * 1024,   // 8KB
            0x03 => 32 * 1024,  // 32KB
            0x04 => 128 * 1024, // 128KB
            0x05 => 64 * 1024,  // 64KB
            o => todo!("No supported RAM size ${:02x}", o),
        };

        let mbc = match data[0x147] {
            0x00 => mbc::new_none(false, false),
            0x01 => mbc::new_mbc1(&data, false, false),
            0x02 => mbc::new_mbc1(&data, true, false),
            0x03 => mbc::new_mbc1(&data, true, true),
            0x0F => mbc::new_mbc3(false, false, true),
            0x10 => mbc::new_mbc3(true, true, true),
            0x11 => mbc::new_mbc3(false, false, false),
            0x12 => mbc::new_mbc3(true, false, false),
            0x13 => mbc::new_mbc3(true, true, false),
            0x19 => mbc::new_mbc5(false, false, false),
            0x1A => mbc::new_mbc5(true, false, false),
            0x1B => mbc::new_mbc5(true, true, false),
            0x1C => mbc::new_mbc5(false, false, true),
            0x1D => mbc::new_mbc5(true, false, true),
            0x1E => mbc::new_mbc5(true, true, true),
            o => todo!("Implement other MBC cartbridge type ${:02x}", o),
        };

        if mbc.has_ram() && ram_size == 0 {
            panic!("MBC asking for RAM but no RAM size in the header");
        } else if !mbc.has_ram() && ram_size > 0 {
            panic!("MBC not asking for RAM but RAM size in the header");
        }

        println!("MBC = {}", mbc.str());
        println!("ROM size = {}KB", rom_size / 1024);
        println!("RAM size = {}KB", ram_size / 1024);

        // TODO: If battery buffered, should try to load the save file

        Self {
            title: title.to_string(),
            mbc,
            rom: data,
            ram: match ram_size {
                0 => None,
                size => Some(vec![0; size]),
            },
        }
    }
}
