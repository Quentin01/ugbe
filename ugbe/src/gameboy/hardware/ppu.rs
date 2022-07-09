#[derive(Debug, Copy, Clone)]
pub struct AddressRange {
    start: u16,
    end: u16,
}

#[derive(Debug, Copy, Clone)]
pub struct Sprite {
    idx_in_oam: usize,
    y: u8,
    x: u8,
    tile_idx: u8,
    attr: u8,
}

impl Ord for Sprite {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.x.cmp(&other.x) {
            std::cmp::Ordering::Equal => self.idx_in_oam.cmp(&other.idx_in_oam),
            other => other,
        }
    }
}

impl PartialOrd for Sprite {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Sprite {
    fn eq(&self, other: &Self) -> bool {
        (self.x, self.idx_in_oam) == (other.x, other.idx_in_oam)
    }
}

impl Eq for Sprite {}

#[derive(Debug, Copy, Clone)]
pub struct Lcdc(u8);

impl From<u8> for Lcdc {
    fn from(value: u8) -> Self {
        Lcdc(value)
    }
}

impl Into<u8> for Lcdc {
    fn into(self) -> u8 {
        self.0
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

    /// Returns the non-inclusive range of memory in the VRAM containing the Window Tile Map
    pub fn window_tile_map_range(&self) -> AddressRange {
        match ((self.0 >> Lcdc::WINDOW_TILE_MAP_SELECT_BIT_POS) & 0x1) != 0 {
            true => AddressRange {
                start: 0x1C00,
                end: 0x2000,
            }, // 9C00-9FFF
            false => AddressRange {
                start: 0x1800,
                end: 0x1C00,
            }, // 9800-9BFF
        }
    }

    /// Returns the non-inclusive range of memory in the VRAM containing the BG Tile Map
    pub fn bg_tile_map_range(&self) -> AddressRange {
        match ((self.0 >> Lcdc::BG_TILE_MAP_SELECT_BIT_POS) & 0x1) != 0 {
            true => AddressRange {
                start: 0x1C00,
                end: 0x2000,
            }, // 9C00-9FFF
            false => AddressRange {
                start: 0x1800,
                end: 0x1C00,
            }, // 9800-9BFF
        }
    }

    /// Returns the non-inclusive range of memory in the VRAM containing the BG & Window Tile Data
    pub fn bg_and_window_tile_data_range(&self) -> AddressRange {
        match ((self.0 >> Lcdc::BG_AND_WINDOW_TILE_DATA_SELECT_BIT_POS) & 0x1) != 0 {
            true => AddressRange {
                start: 0x0,
                end: 0x1000,
            }, // 8000-8FFF
            false => AddressRange {
                start: 0x800,
                end: 0x1800,
            }, // 8800-97FF
        }
    }

    pub fn lcd_enabled(&self) -> bool {
        ((self.0 >> Lcdc::LCD_ENABLE_BIT_POS) & 0x1) != 0
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    // TODO: Add data to the modes so that we know what we are currently doing
    OAMScan {
        sprite_buffer: [Option<Sprite>; 10],
        sprite_buffer_idx: usize,
        sprite_idx: usize,
        current_sprite: Option<Sprite>,
    },
    Drawing {
        sprite_buffer: [Option<Sprite>; 10],
        lx: u8,
        elapsed_cycles: usize,
    },
    HBlank {
        elapsed_cycles: usize,
    },
    VBlank,
}

impl Mode {
    fn execute(mut self, ppu: &mut Ppu) -> Self {
        // If the LCD is OFF, we don't have to do anything
        if !ppu.lcdc.lcd_enabled() {
            return self;
        }

        match self {
            // TODO: Do the ticking of the PPU
            Mode::OAMScan {
                mut sprite_buffer,
                mut sprite_buffer_idx,
                sprite_idx,
                current_sprite,
            } => {
                // Each check of sprite is done on two T-cycles, as we have 40 sprites in the OAM this mode uses 80 T-cycles.
                match current_sprite {
                    None => {
                        // During even T-cycles we will just fetch the sprite
                        let idx_in_oam = sprite_idx * 4;

                        let sprite = Sprite {
                            idx_in_oam: idx_in_oam,
                            y: ppu.oam[idx_in_oam],
                            x: ppu.oam[idx_in_oam + 1],
                            tile_idx: ppu.oam[idx_in_oam + 2],
                            attr: ppu.oam[idx_in_oam + 3],
                        };

                        Mode::OAMScan {
                            sprite_buffer,
                            sprite_buffer_idx,
                            sprite_idx: sprite_idx + 1,
                            current_sprite: Some(sprite),
                        }
                    }
                    Some(current_sprite) => {
                        // During odd T-cycles we are checking if the sprite should be added to the sprite buffer
                        if sprite_buffer_idx >= 10 {
                            if sprite_idx >= 40 {
                                return Mode::switch_from_oam_scan_to_drawing(ppu, sprite_buffer);
                            } else {
                                return Mode::OAMScan {
                                    sprite_buffer,
                                    sprite_buffer_idx,
                                    sprite_idx,
                                    current_sprite: None,
                                };
                            }
                        }

                        let sprite_height = ppu.lcdc.sprite_height();

                        if (current_sprite.y..=(current_sprite.y + sprite_height))
                            .contains(&(ppu.ly + 16))
                        {
                            sprite_buffer[sprite_buffer_idx] = Some(current_sprite);
                            sprite_buffer_idx += 1;
                        }

                        if sprite_idx >= 40 {
                            Mode::switch_from_oam_scan_to_drawing(ppu, sprite_buffer)
                        } else {
                            Mode::OAMScan {
                                sprite_buffer,
                                sprite_buffer_idx,
                                sprite_idx,
                                current_sprite: None,
                            }
                        }
                    }
                }
            }
            Mode::Drawing {
                mut sprite_buffer,
                lx,
                mut elapsed_cycles,
            } => {
                elapsed_cycles += 1;

                // TODO: If we have a sprite fetcher already, use it

                // Fetch the sprite that can be drawn on lx
                // Use a loop instead of a label on block as it isn't stabilized yet
                // For that we need to disable the warning of clippy telling us that our loop is executed only once
                #[allow(clippy::never_loop)] let sprite_to_render = 'fetch_sprite_to_render: loop {
                    if !ppu.lcdc.display_sprite() {
                        break 'fetch_sprite_to_render None;
                    }

                    for sprite_opt in sprite_buffer.iter_mut() {
                        match sprite_opt {
                            Some(sprite) => {
                                if sprite.x <= lx + 8 {
                                    break 'fetch_sprite_to_render sprite_opt.take();
                                }
                            }
                            None => continue,
                        }
                    }

                    break 'fetch_sprite_to_render None;
                };

                match sprite_to_render {
                    Some(_) => todo!("Implement sprite fetcher that will return early"),
                    None => {},
                }

                // TODO: Check if we need to start fetching the window
                // TODO: If we have a background or a window fetcher, use it
                // TODO: If our FIFO isn't empty, shift out a pixel

                if lx >= 160 {
                    Mode::switch_from_drawing_to_hblank(ppu, elapsed_cycles)
                } else {
                    Mode::Drawing {
                        sprite_buffer,
                        lx,
                        elapsed_cycles,
                    }
                }
            }
            Mode::HBlank { elapsed_cycles} => {
                if elapsed_cycles >= 456 {
                    todo!("Exiting HBlank mode")
                } else {
                    Mode::HBlank { elapsed_cycles: elapsed_cycles + 1 }
                }
            },
            Mode::VBlank => todo!("VBlank"),
        }
    }

    fn switch_from_oam_scan_to_drawing(ppu: &mut Ppu, mut sprite_buffer: [Option<Sprite>; 10]) -> Mode {
        // TODO: Interrupts?
        sprite_buffer.sort();

        Mode::Drawing {
            sprite_buffer,
            lx: 0,
            elapsed_cycles: 0,
        }
    }

    fn switch_from_drawing_to_hblank(ppu: &mut Ppu, elapsed_cycles: usize) -> Mode {
        // TODO: Interrupts?
        Mode::HBlank { elapsed_cycles }
    }
}

#[derive(Debug, Clone)]
pub struct Ppu {
    lcdc: Lcdc,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    wy: u8,
    wx: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    mode: Mode,
    vram: [u8; 0x2000],
    oam: [u8; 0x100],
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            lcdc: 0.into(),
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            wy: 0,
            wx: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            mode: Mode::OAMScan {
                sprite_buffer: [None; 10],
                sprite_idx: 0,
                sprite_buffer_idx: 0,
                current_sprite: None,
            },
            vram: [0x0; 0x2000],
            oam: [0x0; 0x100],
        }
    }
}

impl Ppu {
    pub fn tick(&mut self) {
        self.mode = self.mode.execute(self);
    }

    pub fn read_vram_byte(&self, address: u16) -> u8 {
        // TODO: Should check that the VRAM is accessible
        self.vram[address as usize]
    }

    pub fn write_vram_byte(&mut self, address: u16, value: u8) {
        // TODO: Should check that the VRAM is accessible
        self.vram[address as usize] = value;
    }

    pub fn read_oam_byte(&self, address: u16) -> u8 {
        // TODO: Should check that the OAM is accessible
        self.oam[address as usize]
    }

    pub fn write_oam_byte(&mut self, address: u16, value: u8) {
        // TODO: Should check that the OAM is accessible
        self.oam[address as usize] = value;
    }

    pub fn read_lcdc(&self) -> u8 {
        self.lcdc.into()
    }

    pub fn write_lcdc(&mut self, value: u8) {
        // TODO: Do some logic depending on the new LCDC compared to the previous one
        //       e.g the user turned off the LCD
        self.lcdc = value.into();
    }
}
