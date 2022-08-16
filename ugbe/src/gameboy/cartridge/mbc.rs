use std::borrow::Cow;

mod mbc1;
mod mbc5;
mod none;

#[allow(clippy::upper_case_acronyms)]
pub trait MBC {
    fn has_ram(&self) -> bool;
    fn ram_is_battery_buffered(&self) -> bool;

    fn has_rtc(&self) -> bool;
    fn has_rumble(&self) -> bool;

    fn read_rom_bank_0(&self, rom: &[u8], address: u16) -> u8;
    fn read_rom_bank_n(&self, rom: &[u8], address: u16) -> u8;
    fn write_rom(&mut self, rom: &[u8], address: u16, value: u8);

    fn read_ram(&self, ram: &[u8], address: u16) -> u8;
    fn write_ram(&self, ram: &mut [u8], address: u16, value: u8);

    fn str(&self) -> Cow<'static, str>;
}

pub fn new_none(ram: bool, battery: bool) -> Box<dyn MBC + Send + Sync + 'static> {
    Box::new(none::MBC::new(ram, battery))
}

pub fn new_mbc1(
    ram: bool,
    battery: bool,
    multi_cart: bool,
) -> Box<dyn MBC + Send + Sync + 'static> {
    Box::new(mbc1::MBC::new(ram, battery, multi_cart))
}

pub fn new_mbc5(ram: bool, battery: bool, rumble: bool) -> Box<dyn MBC + Send + Sync + 'static> {
    Box::new(mbc5::MBC::new(ram, battery, rumble))
}
