use std::borrow::Cow;

mod mbc1;
mod none;

pub trait Mbc {
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

pub fn new_none(ram: bool, battery_buffered_ram: bool) -> Box<dyn Mbc> {
    Box::new(none::Mbc::new(ram, battery_buffered_ram))
}

pub fn new_mbc1(rom: &[u8], ram: bool, battery_buffered_ram: bool) -> Box<dyn Mbc> {
    Box::new(mbc1::Mbc::new(rom, ram, battery_buffered_ram))
}

pub fn new_mbc3(ram: bool, battery_buffered_ram: bool, rtc: bool) -> Box<dyn Mbc> {
    todo!("New MBC3");
}

pub fn new_mbc5(ram: bool, battery_buffered_ram: bool, rumble: bool) -> Box<dyn Mbc> {
    todo!("New MBC5");
}
