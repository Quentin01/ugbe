use std::ops::Index;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id {
    msb: bool,
    lsb: bool,
}

impl Id {
    pub fn new(msb: bool, lsb: bool) -> Self {
        Self { msb, lsb }
    }

    pub const ZERO: Id = Id {
        msb: false,
        lsb: false,
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Palette(u8);

impl From<u8> for Palette {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Palette> for u8 {
    fn from(palette: Palette) -> Self {
        palette.0
    }
}

impl Index<Id> for Palette {
    type Output = Color;

    fn index(&self, index: Id) -> &Self::Output {
        let index = (index.msb as usize) << 1 | (index.lsb as usize);
        match (self.0 >> (index * 2)) & 0b11 {
            0b00 => &Color::Dmg(DMGColor::White),
            0b01 => &Color::Dmg(DMGColor::LightGray),
            0b10 => &Color::Dmg(DMGColor::DarkGray),
            0b11 => &Color::Dmg(DMGColor::Black),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DMGColor {
    White,
    LightGray,
    DarkGray,
    Black,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Color {
    Dmg(DMGColor),
    Off,
}
