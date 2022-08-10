#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lcdc(u8);

impl From<u8> for Lcdc {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Lcdc> for u8 {
    fn from(lcdc: Lcdc) -> Self {
        lcdc.0
    }
}

impl Lcdc {
    const BG_DISPLAY_BIT_POS: u8 = 0;
    const SPRITE_DISPLAY_BIT_POS: u8 = 1;
    const SPRITE_SIZE_BIT_POS: u8 = 2;
    const BG_TILE_MAP_SELECT_BIT_POS: u8 = 3;
    const BG_AND_WINDOW_TILE_DATA_SELECT_BIT_POS: u8 = 4;
    const WINDOW_DISPLAY_BIT_POS: u8 = 5;
    const WINDOW_TILE_MAP_SELECT_BIT_POS: u8 = 6;
    const LCD_ENABLE_BIT_POS: u8 = 7;

    pub fn display_bg(&self) -> bool {
        ((self.0 >> Lcdc::BG_DISPLAY_BIT_POS) & 0x1) != 0
    }

    pub fn display_window(&self) -> bool {
        ((self.0 >> Lcdc::WINDOW_DISPLAY_BIT_POS) & 0x1) != 0
    }

    pub fn display_sprite(&self) -> bool {
        ((self.0 >> Lcdc::SPRITE_DISPLAY_BIT_POS) & 0x1) != 0
    }

    pub fn sprite_height(&self) -> u8 {
        match ((self.0 >> Lcdc::SPRITE_SIZE_BIT_POS) & 0x1) != 0 {
            true => 16,
            false => 8,
        }
    }

    pub fn window_tile_map(&self) -> super::tiling::TileMap {
        match ((self.0 >> Lcdc::WINDOW_TILE_MAP_SELECT_BIT_POS) & 0x1) != 0 {
            true => super::tiling::TileMap::new(0x1C00), // 9C00-9FFF
            false => super::tiling::TileMap::new(0x1800), // 9800-9BFF
        }
    }

    pub fn bg_tile_map(&self) -> super::tiling::TileMap {
        match ((self.0 >> Lcdc::BG_TILE_MAP_SELECT_BIT_POS) & 0x1) != 0 {
            true => super::tiling::TileMap::new(0x1C00), // 9C00-9FFF
            false => super::tiling::TileMap::new(0x1800), // 9800-9BFF
        }
    }

    pub fn bg_and_window_tile_data_map(&self) -> super::tiling::TileDataMap {
        match ((self.0 >> Lcdc::BG_AND_WINDOW_TILE_DATA_SELECT_BIT_POS) & 0x1) != 0 {
            true => {
                super::tiling::TileDataMap::new(super::tiling::TileDataMapMethod::Method8000, 16)
            }
            false => {
                super::tiling::TileDataMap::new(super::tiling::TileDataMapMethod::Method8800, 16)
            }
        }
    }

    pub fn obj_tile_data_map(&self) -> super::tiling::TileDataMap {
        super::tiling::TileDataMap::new(
            super::tiling::TileDataMapMethod::Method8000,
            self.sprite_height() * 2,
        )
    }

    pub fn lcd_enabled(&self) -> bool {
        ((self.0 >> Lcdc::LCD_ENABLE_BIT_POS) & 0x1) != 0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stat(u8);

impl From<u8> for Stat {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Stat> for u8 {
    fn from(stat: Stat) -> Self {
        stat.0
    }
}

impl Stat {
    const HBLANK_INTERRUPT_BIT_POS: u8 = 3;
    const VBLANK_INTERRUPT_BIT_POS: u8 = 4;
    const OAM_SCANNING_INTERRUPT_BIT_POS: u8 = 5;
    const LYC_INTERRUPT_BIT_POS: u8 = 6;

    pub fn hblank_interrupt_enabled(&self) -> bool {
        ((self.0 >> Stat::HBLANK_INTERRUPT_BIT_POS) & 0x1) != 0
    }

    pub fn vblank_interrupt_enabled(&self) -> bool {
        ((self.0 >> Stat::VBLANK_INTERRUPT_BIT_POS) & 0x1) != 0
    }

    pub fn oam_scanning_interrupt_enabled(&self) -> bool {
        ((self.0 >> Stat::OAM_SCANNING_INTERRUPT_BIT_POS) & 0x1) != 0
    }

    pub fn lyc_interrupt_enabled(&self) -> bool {
        ((self.0 >> Stat::LYC_INTERRUPT_BIT_POS) & 0x1) != 0
    }
}
