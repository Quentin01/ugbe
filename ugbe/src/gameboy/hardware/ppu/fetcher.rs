#[derive(Debug, Copy, Clone)]
pub enum FetcherState {
    GetTile {
        elapsed_cycle: usize,
    },
    GetTileDataLow {
        elapsed_cycle: usize,
        tile_no: super::tiling::TileNo,
    },
    GetTileDataHigh {
        elapsed_cycle: usize,
        tile_no: super::tiling::TileNo,
        tile_low_row_data: super::tiling::TileLowRowData,
    },
    Push {
        tile_low_row_data: super::tiling::TileLowRowData,
        tile_high_row_data: super::tiling::TileHighRowData,
    },
    Sleep,
}

#[derive(Debug, Copy, Clone)]
pub struct PixelPosition {
    x: u8,
    y: u8,
}

pub trait FetcherPixel {
    fn new(ppu: &super::Ppu, color_id: super::ColorId) -> Self;
}

pub trait Fetcher {
    type Pixel: FetcherPixel + Copy;

    fn default_state() -> FetcherState {
        FetcherState::GetTile { elapsed_cycle: 0 }
    }

    fn state(&mut self) -> &mut FetcherState;

    fn tile_map(&self, ppu: &super::Ppu) -> &super::tiling::TileMap;
    fn pixel_position(&mut self) -> &mut PixelPosition;

    fn tick(&mut self, ppu: &super::Ppu, fifo: &mut super::fifo::Fifo<Self::Pixel, 8>) {
        let mut new_state = match *self.state() {
            FetcherState::GetTile { elapsed_cycle } => {
                if elapsed_cycle == 1 {
                    let tile_position = super::tiling::TilePosition {
                        x: self.pixel_position().x / 8,
                        y: self.pixel_position().y / 8,
                    };
                    let tile_no = self.tile_map(ppu).tile_number(ppu, &tile_position);
                    FetcherState::GetTileDataLow {
                        elapsed_cycle: 0,
                        tile_no,
                    }
                } else {
                    FetcherState::GetTile {
                        elapsed_cycle: elapsed_cycle + 1,
                    }
                }
            }
            FetcherState::GetTileDataLow {
                elapsed_cycle,
                tile_no,
            } => {
                if elapsed_cycle == 1 {
                    let row = self.pixel_position().y % 8;
                    let tile_low_row_data = ppu
                        .lcdc
                        .bg_and_window_tile_data_map()
                        .tile(ppu, &tile_no)
                        .get_low_row_data(row);

                    FetcherState::GetTileDataHigh {
                        elapsed_cycle: 0,
                        tile_no,
                        tile_low_row_data,
                    }
                } else {
                    FetcherState::GetTileDataLow {
                        elapsed_cycle: elapsed_cycle + 1,
                        tile_no,
                    }
                }
            }
            FetcherState::GetTileDataHigh {
                elapsed_cycle,
                tile_no,
                tile_low_row_data,
            } => {
                if elapsed_cycle == 1 {
                    let row = self.pixel_position().y % 8;
                    let tile_high_row_data = ppu
                        .lcdc
                        .bg_and_window_tile_data_map()
                        .tile(ppu, &tile_no)
                        .get_high_row_data(row);

                    FetcherState::Push {
                        tile_low_row_data,
                        tile_high_row_data,
                    }
                } else {
                    FetcherState::GetTileDataHigh {
                        elapsed_cycle: elapsed_cycle + 1,
                        tile_no,
                        tile_low_row_data,
                    }
                }
            }
            FetcherState::Push {
                tile_low_row_data,
                tile_high_row_data,
            } => {
                if fifo.len() != 0 {
                    FetcherState::Push {
                        tile_low_row_data,
                        tile_high_row_data,
                    }
                } else {
                    let pixel_row =
                        super::tiling::tile_pixel_row(tile_high_row_data, tile_low_row_data);

                    for pixel_color_id in pixel_row.into_iter() {
                        fifo.push(Self::Pixel::new(ppu, pixel_color_id));
                    }

                    self.pixel_position().x += 8;
                    FetcherState::Sleep
                }
            }
            FetcherState::Sleep => FetcherState::GetTile { elapsed_cycle: 0 },
        };

        std::mem::swap(self.state(), &mut new_state);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BackgroundWindowPixel {
    color_id: super::ColorId,
    palette: super::Palette,
}

impl FetcherPixel for BackgroundWindowPixel {
    fn new(ppu: &super::Ppu, color_id: super::ColorId) -> Self {
        BackgroundWindowPixel {
            color_id,
            palette: ppu.bgp,
        }
    }
}

impl BackgroundWindowPixel {
    pub fn color(&self) -> super::screen::Color {
        self.palette[self.color_id]
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BackgroundWindowFetcher {
    tile_map: super::tiling::TileMap,
    position: PixelPosition,
    state: FetcherState,
}

impl BackgroundWindowFetcher {
    pub fn new(tile_map: super::tiling::TileMap, x: u8, y: u8) -> Self {
        Self {
            tile_map,
            position: PixelPosition { x, y },
            state: Self::default_state(),
        }
    }

    pub fn reset(&mut self) {
        self.state = Self::default_state();
    }
}

impl Fetcher for BackgroundWindowFetcher {
    type Pixel = BackgroundWindowPixel;

    fn tile_map(&self, _: &super::Ppu) -> &super::tiling::TileMap {
        &self.tile_map
    }

    fn pixel_position(&mut self) -> &mut PixelPosition {
        &mut self.position
    }

    fn state(&mut self) -> &mut FetcherState {
        &mut self.state
    }
}
