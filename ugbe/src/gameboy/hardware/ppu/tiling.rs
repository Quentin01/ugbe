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
pub struct TilePosition{ pub x: u8, pub y: u8 }

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
        let tile_y =  (position.y as usize % Self::HEIGHT) as u16;

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
pub struct Tile<'a, const SIZE: usize> {
    data: &'a [u8; SIZE]
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

impl<'a, const SIZE: usize> Tile<'a, SIZE> {
    const ROW_COUNT: u8 = (SIZE / 2) as u8;
    
    pub fn get_low_row_data(&self, row: u8) -> TileLowRowData {
        let idx = ((row % Self::ROW_COUNT) * 2) as usize;
        self.data[idx].into()
    }

    pub fn get_high_row_data(&self, row: u8) -> TileHighRowData {
        let idx = ((row % Self::ROW_COUNT) * 2) as usize;
        self.data[idx + 1].into()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TileDataMap<const TILE_SIZE: usize> {
    method: TileDataMapMethod,
}

impl<const TILE_SIZE: usize> TileDataMap<TILE_SIZE> {
    pub fn new(method: TileDataMapMethod) -> Self {
        Self { method }
    }

    pub fn tile<'a>(&self, ppu: &'a super::Ppu, tile_no: &TileNo) -> Tile<'a, TILE_SIZE> {
        let tile_address = match self.method {
            TileDataMapMethod::Method8000 => {
                let base_address: u16 = 0x0; // 0x8000
                base_address + (tile_no.0 as u16) * (TILE_SIZE as u16)
            },
            TileDataMapMethod::Method8800 => {
                let base_address: u16 = 0x1000; // 0x9000
                ((base_address as i32) + (tile_no.0 as i8 as i32) * (TILE_SIZE as i32)) as u16
            },
        } as usize;
        let tile_address_end = tile_address + TILE_SIZE;

        Tile{ data: ppu.vram[tile_address..tile_address_end].try_into().expect("Error of size in the range") }
    }
}

pub fn tile_pixel_row(high_row_data: TileHighRowData, low_row_data: TileLowRowData) -> [super::ColorId; 8] {
    let mut pixel_row = [super::ColorId { msb: false, lsb: false }; 8];
    
    for i in 0..8 {
        pixel_row[7 - i] = super::ColorId {
            msb: (high_row_data.0 >> i) & 0x1 == 0x1,
            lsb: (low_row_data.0 >> i) & 0x1 == 0x1,
        };
    }

    pixel_row
}