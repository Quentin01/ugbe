mod mbc;

pub struct Cartridge {
    cartridge: crate::cartridge::Cartridge,
    mbc: Box<dyn mbc::Mbc + Send + Sync + 'static>,
}

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cartbridge")
            .field("cartridge", &self.cartridge)
            .field("mbc", &self.mbc.str())
            .finish()
    }
}

impl Cartridge {
    pub fn read_rom_bank_0(&self, address: u16) -> u8 {
        self.mbc.read_rom_bank_0(self.cartridge.rom(), address)
    }

    pub fn read_rom_bank_n(&self, address: u16) -> u8 {
        self.mbc.read_rom_bank_n(self.cartridge.rom(), address)
    }

    pub fn write_rom(&mut self, address: u16, value: u8) {
        self.mbc.write_rom(self.cartridge.rom(), address, value)
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        match self.cartridge.ram() {
            Some(ram) => self.mbc.read_ram(ram, address),
            None => 0xFF,
        }
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        match self.cartridge.mut_ram() {
            Some(ram) => self.mbc.write_ram(ram, address, value),
            None => {}
        }
    }
}

impl From<crate::cartridge::Cartridge> for Cartridge {
    fn from(cartridge: crate::cartridge::Cartridge) -> Self {
        let mbc = match cartridge.header().kind {
            crate::cartridge::Kind::NoMBC { ram, battery } => mbc::new_none(ram, battery),
            crate::cartridge::Kind::MBC1 {
                ram,
                battery,
                multi_cart,
            } => mbc::new_mbc1(ram, battery, multi_cart),
            crate::cartridge::Kind::MBC5 {
                ram,
                battery,
                rumble,
            } => mbc::new_mbc5(ram, battery, rumble),

            kind => todo!("Not yet supported {}", kind),
        };

        Self { cartridge, mbc }
    }
}
