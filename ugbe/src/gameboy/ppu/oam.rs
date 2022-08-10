const SPRITE_COUNT: usize = 40;
const SPRITE_SIZE: usize = 4;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpriteNo(u8);

impl SpriteNo {
    pub fn first() -> Self {
        Self(0)
    }

    pub fn wrapping_inc(&self) -> Self {
        Self((self.0 + 1) % (SPRITE_COUNT as u8))
    }

    pub fn last() -> Self {
        Self((SPRITE_COUNT - 1) as u8)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Sprite {
    no: SpriteNo,
    y: u8,
    x: u8,
    tile_no: super::tiling::TileNo,
    attr: u8,
}

impl Sprite {
    pub fn no(&self) -> SpriteNo {
        self.no
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn tile_no(&self) -> super::tiling::TileNo {
        self.tile_no
    }

    pub fn palette(&self, ppu: &super::Ppu) -> super::color::Palette {
        if self.attr & (1 << 4) == 0 {
            ppu.obp0
        } else {
            ppu.obp1
        }
    }

    pub fn x_flip(&self) -> bool {
        self.attr & (1 << 5) != 0
    }

    pub fn y_flip(&self) -> bool {
        self.attr & (1 << 6) != 0
    }

    pub fn over_bg_and_win(&self) -> bool {
        self.attr & (1 << 7) == 0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Oam {
    data: [u8; SPRITE_COUNT * SPRITE_SIZE],
}

impl Oam {
    pub fn new() -> Self {
        Self {
            data: [0; SPRITE_COUNT * SPRITE_SIZE],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }

    pub fn sprite(&self, no: SpriteNo) -> Sprite {
        let idx_in_oam = ((no.0 % 40) * 4) as usize;

        Sprite {
            no,
            y: self.data[idx_in_oam],
            x: self.data[idx_in_oam + 1],
            tile_no: self.data[idx_in_oam + 2].into(),
            attr: self.data[idx_in_oam + 3],
        }
    }
}
