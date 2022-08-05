#[derive(Debug, Copy, Clone)]
pub struct TileNo(u8);

impl From<u8> for TileNo {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<TileNo> for u8 {
    fn from(tile_no: TileNo) -> Self {
        tile_no.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TilePosition {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct TileMap {
    start: u16,
}

impl TileMap {
    const WIDTH: usize = 32;
    const HEIGHT: usize = 32;

    pub fn new(start: u16) -> Self {
        Self { start }
    }

    pub fn tile_number(&self, ppu: &super::Ppu, position: &TilePosition) -> TileNo {
        let tile_x = (position.x as usize % Self::WIDTH) as u16;
        let tile_y = (position.y as usize % Self::HEIGHT) as u16;

        let tile_offset = (Self::WIDTH as u16 * tile_y) + tile_x;
        let tile_addr = self.start + (tile_offset & 0x3FF);
        TileNo(ppu.vram[tile_addr as usize])
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TileDataMapMethod {
    Method8000,
    Method8800,
}

#[derive(Debug, Copy, Clone)]
pub struct Tile<'a> {
    data: &'a [u8],
}

#[derive(Debug, Copy, Clone)]
pub struct TileLowRowData(u8);

impl From<u8> for TileLowRowData {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<TileLowRowData> for u8 {
    fn from(tile_row_low_data: TileLowRowData) -> Self {
        tile_row_low_data.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TileHighRowData(u8);

impl From<u8> for TileHighRowData {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<TileHighRowData> for u8 {
    fn from(tile_row_high_data: TileHighRowData) -> Self {
        tile_row_high_data.0
    }
}

impl<'a> Tile<'a> {
    fn row_count(&self) -> u8 {
        (self.data.len() / 2) as u8
    }

    pub fn get_low_row_data(&self, row: u8) -> TileLowRowData {
        let idx = ((row % self.row_count()) * 2) as usize;
        self.data[idx].into()
    }

    pub fn get_high_row_data(&self, row: u8) -> TileHighRowData {
        let idx = ((row % self.row_count()) * 2) as usize;
        self.data[idx + 1].into()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TileDataMap {
    size: u8,
    method: TileDataMapMethod,
}

impl TileDataMap {
    const TILE_SIZE_IN_MEMORY: u8 = 16;

    pub fn new(method: TileDataMapMethod, size: u8) -> Self {
        assert!(size % Self::TILE_SIZE_IN_MEMORY == 0);
        Self { method, size }
    }

    pub fn tile<'a>(&self, ppu: &'a super::Ppu, tile_no: &TileNo) -> Tile<'a> {
        let bits_count_to_ignore = self.size / Self::TILE_SIZE_IN_MEMORY;
        let tile_mask = !((1 << (bits_count_to_ignore - 1)) - 1);
        let tile_no = tile_no.0 & tile_mask;

        let tile_address = match self.method {
            TileDataMapMethod::Method8000 => {
                let base_address: u16 = 0x0; // 0x8000
                base_address + (tile_no as u16) * (Self::TILE_SIZE_IN_MEMORY as u16)
            }
            TileDataMapMethod::Method8800 => {
                let base_address: u16 = 0x1000; // 0x9000
                ((base_address as i32)
                    + (tile_no as i8 as i32) * (Self::TILE_SIZE_IN_MEMORY as i32))
                    as u16
            }
        } as usize;

        let tile_address_end = tile_address + self.size as usize;

        Tile {
            data: &ppu.vram[tile_address..tile_address_end],
        }
    }
}

pub fn tile_pixel_row(
    high_row_data: TileHighRowData,
    low_row_data: TileLowRowData,
) -> [super::ColorId; 8] {
    let mut pixel_row = [super::ColorId {
        msb: false,
        lsb: false,
    }; 8];

    for i in 0..8 {
        pixel_row[7 - i] = super::ColorId {
            msb: (high_row_data.0 >> i) & 0x1 == 0x1,
            lsb: (low_row_data.0 >> i) & 0x1 == 0x1,
        };
    }

    pixel_row
}
