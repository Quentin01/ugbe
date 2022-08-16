#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BackgroundWindowPixel {
    color_id: super::color::Id,
    palette: super::color::Palette,
}

impl BackgroundWindowPixel {
    fn new(ppu_ctx: &super::Context, color_id: super::color::Id) -> Self {
        BackgroundWindowPixel {
            color_id,
            palette: ppu_ctx.bgp,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.color_id == super::color::Id::ZERO
    }

    pub fn color(&self) -> super::color::Color {
        self.palette[self.color_id]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackgroundWindowFetcher {
    tile_map: super::tiling::TileMap,
    position: super::tiling::PixelPosition,
    state: FetcherState,
}

impl BackgroundWindowFetcher {
    pub fn new(
        tile_map: super::tiling::TileMap,
        starting_position: super::tiling::PixelPosition,
    ) -> Self {
        Self {
            tile_map,
            position: starting_position,
            state: FetcherState::GetTile { elapsed_cycle: 0 },
        }
    }

    pub fn reset(&mut self) {
        self.state = FetcherState::GetTile { elapsed_cycle: 0 };
    }

    pub fn tick(
        &mut self,
        ppu_ctx: &super::Context,
        fifo: &mut super::fifo::Fifo<BackgroundWindowPixel, 8>,
    ) {
        let mut new_state = match self.state {
            FetcherState::GetTile { elapsed_cycle } => {
                if elapsed_cycle == 1 {
                    let tile_position = self.position.into();

                    let tile_no = self.tile_map.tile_number(ppu_ctx, &tile_position);
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
                    let row = self.position.y() % 8;
                    let tile_low_row_data = ppu_ctx
                        .lcdc
                        .bg_and_window_tile_data_map()
                        .tile(ppu_ctx, &tile_no)
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
                    let row = self.position.y() % 8;
                    let tile_high_row_data = ppu_ctx
                        .lcdc
                        .bg_and_window_tile_data_map()
                        .tile(ppu_ctx, &tile_no)
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
                        if ppu_ctx.lcdc.display_bg() {
                            fifo.push(BackgroundWindowPixel::new(ppu_ctx, pixel_color_id));
                        } else {
                            fifo.push(BackgroundWindowPixel::new(ppu_ctx, super::color::Id::ZERO));
                        }
                    }

                    self.position.set_x(self.position.x() as usize + 8);
                    FetcherState::Sleep
                }
            }
            FetcherState::Sleep => FetcherState::GetTile { elapsed_cycle: 0 },
        };

        std::mem::swap(&mut self.state, &mut new_state);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SpritePixel {
    sprite: super::oam::Sprite,
    color_id: super::color::Id,
}

impl SpritePixel {
    fn new(sprite: super::oam::Sprite, color_id: super::color::Id) -> Self {
        SpritePixel { sprite, color_id }
    }

    pub fn is_zero(&self) -> bool {
        self.color_id == super::color::Id::ZERO
    }

    pub fn over_bg_and_win(&self) -> bool {
        self.sprite.over_bg_and_win()
    }

    pub fn color(&self, ppu_ctx: &super::Context) -> super::color::Color {
        self.sprite.palette(ppu_ctx)[self.color_id]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpriteFetcher {
    sprite: super::oam::Sprite,
    state: FetcherState,
    pixel_row: u8,
}

impl SpriteFetcher {
    pub fn new(sprite: super::oam::Sprite, ppu_ctx: &super::Context) -> Self {
        let pixel_row = if sprite.y_flip() {
            ppu_ctx.lcdc.sprite_height() - (ppu_ctx.ly + 16 - sprite.y()) - 1
        } else {
            ppu_ctx.ly + 16 - sprite.y()
        };

        Self {
            sprite,
            state: FetcherState::GetTile { elapsed_cycle: 0 },
            pixel_row,
        }
    }

    pub fn tick(
        &mut self,
        ppu_ctx: &super::Context,
        lx: u8,
        fifo: &mut super::fifo::Fifo<SpritePixel, 8>,
    ) -> bool {
        let mut new_state = match self.state {
            FetcherState::GetTile { elapsed_cycle } => {
                if elapsed_cycle == 1 {
                    let tile_no = self.sprite.tile_no();

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
                    let row = self.pixel_row;
                    let tile_low_row_data = ppu_ctx
                        .lcdc
                        .obj_tile_data_map()
                        .tile(ppu_ctx, &tile_no)
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
                    let row = self.pixel_row;
                    let tile_high_row_data = ppu_ctx
                        .lcdc
                        .obj_tile_data_map()
                        .tile(ppu_ctx, &tile_no)
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
                let pixel_row =
                    super::tiling::tile_pixel_row(tile_high_row_data, tile_low_row_data);

                let mut pixel_row_normal_it;
                let mut pixel_row_rev_it;

                let pixel_row_it: &mut dyn Iterator<Item = super::color::Id> =
                    if self.sprite.x_flip() {
                        let x_offset = (lx + 8 - self.sprite.x()) as usize;
                        pixel_row_rev_it = pixel_row.into_iter().skip(x_offset).rev();
                        &mut pixel_row_rev_it
                    } else {
                        let x_offset = (lx + 8 - self.sprite.x()) as usize;
                        pixel_row_normal_it = pixel_row.into_iter().skip(x_offset);
                        &mut pixel_row_normal_it
                    };

                for (idx, pixel_color_id) in pixel_row_it.enumerate() {
                    let pixel = SpritePixel::new(self.sprite, pixel_color_id);

                    if fifo.len() > idx {
                        let have_priority = {
                            let fifo_pixel = &fifo[idx];

                            if pixel.is_zero() {
                                false
                            } else if fifo_pixel.is_zero() {
                                true
                            } else {
                                // In non-CGB mode, the smaller the X coordinate, the higher the priority.
                                // When X coordinates are identical, the object located first in OAM has higher priority.
                                // In CGB mode, only the object’s location in OAM determines its priority. The earlier the object, the higher its priority.
                                match pixel.sprite.x().cmp(&fifo_pixel.sprite.x()) {
                                    std::cmp::Ordering::Less => true,
                                    std::cmp::Ordering::Equal => {
                                        pixel.sprite.no() < fifo_pixel.sprite.no()
                                    }
                                    std::cmp::Ordering::Greater => false,
                                }
                            }
                        };

                        if have_priority {
                            fifo[idx] = pixel;
                        }
                    } else {
                        fifo.push(pixel);
                    }
                }

                FetcherState::Sleep
            }
            FetcherState::Sleep => return true,
        };

        std::mem::swap(&mut self.state, &mut new_state);
        false
    }
}
