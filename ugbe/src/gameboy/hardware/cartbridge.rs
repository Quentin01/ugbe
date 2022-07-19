#[derive(Debug, Clone)]
pub enum MBC {
    None,
}

#[derive(Debug, Clone)]
pub struct Cartbridge {
    title: String,
    mbc: MBC,
    rom: Vec<u8>,
    ram: Option<Vec<u8>>,
    rom_bank0_offset: u16,
    rom_bankN_offset: u16,
}

impl Cartbridge {
    pub fn read_rom_bank0(&self, address: u16) -> u8 {
        self.rom[(address + self.rom_bank0_offset) as usize]
    }

    pub fn read_rom_bankN(&self, address: u16) -> u8 {
        self.rom[(address + self.rom_bankN_offset) as usize]
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        match self.ram.as_ref() {
            Some(ram) => ram[address as usize],
            None => 0xFF,
        }
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        match self.ram.as_mut() {
            Some(ram) => ram[address as usize] = value,
            None => {}
        }
    }
}

impl From<Vec<u8>> for Cartbridge {
    fn from(data: Vec<u8>) -> Self {
        let title = &data[0x134..=0x143];
        let title_size = title.iter().position(|&c| c == b'\0').unwrap_or(title.len());
        let title =
            String::from_utf8(title[0..title_size].to_vec()).expect("Fail parsing cartbridge title");
        println!("ROM title: {}", title);

        let mbc = match data[0x147] {
            0x00 => MBC::None,
            0x01 => MBC::None, // TO REMOVE: MBC1 but fake it to None for blargg tests
            o => todo!("Implement other MBC cartbridge type ${:02x}", o),
        };

        let rom_size = match data[0x148] {
            0x00 => 32 * 1024, // 32KB
            o => todo!("Implement other ROM size ${:02x}", o),
        };

        if data.len() != rom_size {
            panic!("ROM file different that the internal reported ROM size");
        }

        let ram_size = match data[0x149] {
            0x00 => 0, // No RAM
            o => todo!("Implement other RAM size ${:02x}", o),
        };

        // TODO: Save data support
        Self {
            title,
            mbc,
            rom: data,
            ram: match ram_size {
                0 => None,
                size => Some(vec![0; size]),
            },
            rom_bank0_offset: 0x0,
            rom_bankN_offset: 0x4000,
        }
    }
}
