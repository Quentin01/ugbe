use std::{fmt::Debug, ops::Index};

use super::super::interrupt::Kind as InterruptKind;
use super::super::interrupt::Line as InterruptLine;

mod fetcher;
mod fifo;
mod registers;
pub mod screen;
mod tiling;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ColorId {
    msb: bool,
    lsb: bool,
}

impl ColorId {
    const ZERO: ColorId = ColorId {
        msb: false,
        lsb: false,
    };
}

#[derive(Debug, Copy, Clone)]
pub struct Sprite {
    idx_in_oam: usize,
    y: u8,
    x: u8,
    tile_no: tiling::TileNo,
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

impl Sprite {
    fn palette(&self, ppu: &Ppu) -> Palette {
        if self.attr & (1 << 4) == 0 {
            ppu.obp0
        } else {
            ppu.obp1
        }
    }

    fn x_flip(&self) -> bool {
        self.attr & (1 << 5) != 0
    }

    fn y_flip(&self) -> bool {
        self.attr & (1 << 6) != 0
    }

    fn over_bg_and_win(&self) -> bool {
        self.attr & (1 << 7) == 0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

impl Index<ColorId> for Palette {
    type Output = screen::Color;

    fn index(&self, index: ColorId) -> &Self::Output {
        let index = (index.msb as usize) << 1 | (index.lsb as usize);
        match (self.0 >> (index * 2)) & 0b11 {
            0 => &screen::Color::White,
            1 => &screen::Color::LightGray,
            2 => &screen::Color::DarkGray,
            3 => &screen::Color::Black,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    OAMScan {
        sprite_buffer: [Option<Sprite>; 10],
        sprite_buffer_idx: usize,
        wy_match_ly: bool,
        sprite_idx: usize,
        current_sprite: Option<Sprite>,
        win_ly: u8,
    },
    Drawing {
        sprite_buffer: [Option<Sprite>; 10],
        wy_match_ly: bool,
        scx_delay: u8,
        lx: u8,
        elapsed_cycles: usize,
        win_fetcher: bool,
        bg_win_fetcher: fetcher::BackgroundWindowFetcher,
        bg_win_fifo: fifo::Fifo<fetcher::BackgroundWindowPixel, 8>,
        sprite_fetcher: Option<fetcher::SpriteFetcher>,
        sprite_fifo: fifo::Fifo<fetcher::SpritePixel, 8>,
        win_ly: u8,
    },
    HBlank {
        elapsed_cycles: usize,
        wy_match_ly: bool,
        win_ly: u8,
    },
    VBlank {
        elapsed_cycles_line: usize,
        ly: u8,
    },
}

impl Default for Mode {
    fn default() -> Self {
        Self::OAMScan {
            sprite_buffer: [None; 10],
            sprite_buffer_idx: 0,
            wy_match_ly: false,
            sprite_idx: 0,
            current_sprite: None,
            win_ly: 0,
        }
    }
}

impl Mode {
    fn execute(
        self,
        ppu: &mut Ppu,
        interrupt_line: &mut dyn InterruptLine,
    ) -> (Self, Option<screen::Event>) {
        // If the LCD is OFF, we don't have to do anything
        if !ppu.lcdc.lcd_enabled() {
            return (self, None);
        }

        match self {
            Mode::OAMScan {
                mut sprite_buffer,
                mut sprite_buffer_idx,
                wy_match_ly,
                sprite_idx,
                current_sprite,
                win_ly,
            } => {
                // Each check of sprite is done on two T-cycles, as we have 40 sprites in the OAM this mode uses 80 T-cycles.
                match current_sprite {
                    None => {
                        // During even T-cycles we will just fetch the sprite
                        let idx_in_oam = sprite_idx * 4;

                        let sprite = Sprite {
                            idx_in_oam,
                            y: ppu.oam[idx_in_oam],
                            x: ppu.oam[idx_in_oam + 1],
                            tile_no: ppu.oam[idx_in_oam + 2].into(),
                            attr: ppu.oam[idx_in_oam + 3],
                        };

                        (
                            Mode::OAMScan {
                                sprite_buffer,
                                sprite_buffer_idx,
                                wy_match_ly: if ppu.wy == ppu.ly { true } else { wy_match_ly },
                                sprite_idx: sprite_idx + 1,
                                current_sprite: Some(sprite),
                                win_ly,
                            },
                            None,
                        )
                    }
                    Some(current_sprite) => {
                        // During odd T-cycles we are checking if the sprite should be added to the sprite buffer
                        if sprite_buffer_idx >= 10 {
                            if sprite_idx >= 40 {
                                return (
                                    Mode::switch_from_oam_scan_to_drawing(
                                        ppu,
                                        sprite_buffer,
                                        wy_match_ly,
                                        win_ly,
                                    ),
                                    None,
                                );
                            } else {
                                return (
                                    Mode::OAMScan {
                                        sprite_buffer,
                                        sprite_buffer_idx,
                                        wy_match_ly,
                                        sprite_idx,
                                        current_sprite: None,
                                        win_ly,
                                    },
                                    None,
                                );
                            }
                        }

                        let sprite_height = ppu.lcdc.sprite_height();

                        let sprite_y_range_on_screen = if current_sprite.y >= 16 {
                            let sprite_y_on_screen = current_sprite.y - 16;
                            sprite_y_on_screen..(sprite_y_on_screen + sprite_height)
                        } else if sprite_height + current_sprite.y > 16 {
                            0..(sprite_height + current_sprite.y - 16)
                        } else {
                            0..0
                        };

                        if sprite_y_range_on_screen.contains(&ppu.ly) {
                            sprite_buffer[sprite_buffer_idx] = Some(current_sprite);
                            sprite_buffer_idx += 1;
                        }

                        if sprite_idx >= 40 {
                            (
                                Mode::switch_from_oam_scan_to_drawing(
                                    ppu,
                                    sprite_buffer,
                                    wy_match_ly,
                                    win_ly,
                                ),
                                None,
                            )
                        } else {
                            (
                                Mode::OAMScan {
                                    sprite_buffer,
                                    sprite_buffer_idx,
                                    wy_match_ly,
                                    sprite_idx,
                                    current_sprite: None,
                                    win_ly,
                                },
                                None,
                            )
                        }
                    }
                }
            }
            Mode::Drawing {
                mut sprite_buffer,
                wy_match_ly,
                mut scx_delay,
                mut lx,
                mut elapsed_cycles,
                mut win_fetcher,
                mut bg_win_fetcher,
                mut bg_win_fifo,
                mut sprite_fetcher,
                mut sprite_fifo,
                mut win_ly,
            } => {
                elapsed_cycles += 1;

                if sprite_fetcher.is_none() {
                    // Fetch the sprite that could be render starting at lx
                    // For that we need to disable the warning of clippy telling us that our loop is executed only once as we don't have labelled block yet
                    #[allow(clippy::never_loop)]
                    let sprite_to_render = 'fetch_sprite_to_render: loop {
                        if !ppu.lcdc.display_sprite() {
                            break 'fetch_sprite_to_render None;
                        }

                        for sprite_opt in sprite_buffer.iter_mut() {
                            match sprite_opt {
                                Some(sprite) => {
                                    let sprite_x_range_on_screen = if sprite.x >= 8 {
                                        let sprite_x_on_screen = sprite.x - 8;
                                        sprite_x_on_screen..(sprite_x_on_screen + 8)
                                    } else if sprite.x + 8 > 8 {
                                        0..sprite.x
                                    } else {
                                        0..0
                                    };

                                    if sprite_x_range_on_screen.contains(&lx) {
                                        break 'fetch_sprite_to_render sprite_opt.take();
                                    }
                                }
                                None => {
                                    continue;
                                }
                            }
                        }

                        break 'fetch_sprite_to_render None;
                    };

                    match sprite_to_render {
                        Some(sprite) => {
                            bg_win_fetcher.reset();
                            sprite_fetcher = Some(fetcher::SpriteFetcher::new(sprite, ppu));
                        }
                        None => {}
                    }
                }

                if let Some(fetcher) = &mut sprite_fetcher {
                    if fetcher.tick(ppu, lx, &mut sprite_fifo) {
                        sprite_fetcher = None;
                    }

                    return (
                        Mode::Drawing {
                            sprite_buffer,
                            wy_match_ly,
                            scx_delay,
                            lx,
                            elapsed_cycles,
                            win_fetcher,
                            bg_win_fetcher,
                            bg_win_fifo,
                            sprite_fetcher,
                            sprite_fifo,
                            win_ly,
                        },
                        None,
                    );
                }

                // Check if we need to start fetching the window
                if ppu.lcdc.display_window() && wy_match_ly && lx == ppu.wx - 7 && !win_fetcher {
                    bg_win_fetcher = fetcher::BackgroundWindowFetcher::new(
                        ppu.lcdc.window_tile_map(),
                        tiling::PixelPosition::new(0, win_ly as usize),
                    );

                    bg_win_fifo.clear();

                    win_fetcher = true;
                    win_ly += 1;
                }

                bg_win_fetcher.tick(ppu, &mut bg_win_fifo);

                if bg_win_fifo.len() > 0 {
                    if scx_delay != 0 {
                        bg_win_fifo.pop();

                        if sprite_fifo.len() > 0 {
                            sprite_fifo.pop();
                        }

                        scx_delay -= 1;
                    } else {
                        let bg_pixel = bg_win_fifo.pop();

                        let pixel_color = if sprite_fifo.len() > 0 {
                            let sprite_pixel = sprite_fifo.pop();

                            if sprite_pixel.is_zero()
                                || (!sprite_pixel.over_bg_and_win() && !bg_pixel.is_zero())
                            {
                                bg_pixel.color()
                            } else {
                                sprite_pixel.color()
                            }
                        } else {
                            bg_pixel.color()
                        };

                        if !ppu.skip_frame {
                            ppu.screen.set_pixel(lx.into(), ppu.ly.into(), pixel_color);
                        }

                        lx = lx.wrapping_add(1)
                    }
                }

                if lx >= 160 {
                    (
                        Mode::switch_from_drawing_to_hblank(
                            ppu,
                            interrupt_line,
                            elapsed_cycles,
                            wy_match_ly,
                            win_ly,
                        ),
                        None,
                    )
                } else {
                    (
                        Mode::Drawing {
                            sprite_buffer,
                            wy_match_ly,
                            scx_delay,
                            lx,
                            elapsed_cycles,
                            win_fetcher,
                            bg_win_fetcher,
                            bg_win_fifo,
                            sprite_fetcher,
                            sprite_fifo,
                            win_ly,
                        },
                        None,
                    )
                }
            }
            Mode::HBlank {
                mut elapsed_cycles,
                wy_match_ly,
                win_ly,
            } => {
                elapsed_cycles += 1;

                // A line is 456 T-cycles but elapsed_cycles started counting after the OAM scan which have a duration of 80 T-cycles
                if elapsed_cycles >= 456 - 80 {
                    if ppu.ly == 143 {
                        (
                            Self::switch_from_hblank_to_vblank(ppu, interrupt_line),
                            Some(screen::Event::VBlank),
                        )
                    } else {
                        (
                            Self::switch_from_hblank_to_oam_scan(
                                ppu,
                                interrupt_line,
                                wy_match_ly,
                                win_ly,
                            ),
                            None,
                        )
                    }
                } else {
                    (
                        Mode::HBlank {
                            elapsed_cycles,
                            wy_match_ly,
                            win_ly,
                        },
                        None,
                    )
                }
            }
            Mode::VBlank {
                mut elapsed_cycles_line,
                mut ly,
            } => {
                elapsed_cycles_line += 1;

                if elapsed_cycles_line == 456 {
                    if ly == 153 {
                        (
                            Self::switch_from_vblank_to_oam_scan(ppu, interrupt_line),
                            None,
                        )
                    } else {
                        ly += 1;
                        elapsed_cycles_line = 0;

                        ppu.ly += 1;
                        ppu.check_lyc_compare(interrupt_line);

                        (
                            Mode::VBlank {
                                elapsed_cycles_line,
                                ly,
                            },
                            None,
                        )
                    }
                } else {
                    if elapsed_cycles_line == 4 && ly == 153 {
                        // Simulate that the VBlank change LY to 0 after 4 T-cycles in the line 153
                        ppu.ly = 0;
                        ppu.check_lyc_compare(interrupt_line);
                    }

                    (
                        Mode::VBlank {
                            elapsed_cycles_line,
                            ly,
                        },
                        None,
                    )
                }
            }
        }
    }

    fn switch_from_oam_scan_to_drawing(
        ppu: &mut Ppu,
        mut sprite_buffer: [Option<Sprite>; 10],
        wy_match_ly: bool,
        win_ly: u8,
    ) -> Mode {
        sprite_buffer.sort();

        Mode::Drawing {
            sprite_buffer,
            wy_match_ly,
            scx_delay: ppu.scx % 8,
            lx: 0,
            elapsed_cycles: 0,
            win_fetcher: false,
            bg_win_fetcher: fetcher::BackgroundWindowFetcher::new(
                ppu.lcdc.bg_tile_map(),
                tiling::PixelPosition::new(ppu.scx as usize, ppu.ly as usize + ppu.scy as usize),
            ),
            bg_win_fifo: fifo::Fifo::new(),
            sprite_fetcher: None,
            sprite_fifo: fifo::Fifo::new(),
            win_ly,
        }
    }

    fn switch_from_drawing_to_hblank(
        ppu: &mut Ppu,
        interrupt_line: &mut dyn InterruptLine,
        elapsed_cycles: usize,
        wy_match_ly: bool,
        win_ly: u8,
    ) -> Mode {
        if ppu.stat.hblank_interrupt_enabled() {
            interrupt_line.request(InterruptKind::Stat);
        }

        Mode::HBlank {
            elapsed_cycles,
            win_ly,
            wy_match_ly,
        }
    }

    fn switch_from_hblank_to_vblank(ppu: &mut Ppu, interrupt_line: &mut dyn InterruptLine) -> Mode {
        interrupt_line.request(InterruptKind::VBlank);
        if ppu.stat.vblank_interrupt_enabled() {
            interrupt_line.request(InterruptKind::Stat);
        }
        if ppu.stat.oam_scanning_interrupt_enabled() {
            interrupt_line.request(InterruptKind::Stat);
        }

        ppu.skip_frame = false;

        ppu.ly += 1;
        ppu.check_lyc_compare(interrupt_line);

        Mode::VBlank {
            elapsed_cycles_line: 0,
            ly: ppu.ly,
        }
    }

    fn switch_from_hblank_to_oam_scan(
        ppu: &mut Ppu,
        interrupt_line: &mut dyn InterruptLine,
        wy_match_ly: bool,
        win_ly: u8,
    ) -> Mode {
        if ppu.stat.oam_scanning_interrupt_enabled() {
            interrupt_line.request(InterruptKind::Stat);
        }

        ppu.ly += 1;
        ppu.check_lyc_compare(interrupt_line);

        Self::OAMScan {
            sprite_buffer: [None; 10],
            sprite_buffer_idx: 0,
            wy_match_ly,
            sprite_idx: 0,
            current_sprite: None,
            win_ly,
        }
    }

    fn switch_from_vblank_to_oam_scan(
        ppu: &mut Ppu,
        interrupt_line: &mut dyn InterruptLine,
    ) -> Mode {
        if ppu.stat.oam_scanning_interrupt_enabled() {
            interrupt_line.request(InterruptKind::Stat);
        }

        Self::default()
    }
}

pub struct Ppu {
    skip_frame: bool,
    lcdc: registers::Lcdc,
    lyc_compare: bool,
    stat: registers::Stat,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    wy: u8,
    wx: u8,
    bgp: Palette,
    obp0: Palette,
    obp1: Palette,
    mode: Mode,
    vram: [u8; 0x2000],
    oam: [u8; 0x100],
    screen: screen::Screen,
    ldc_event: Option<screen::Event>,
}

impl Debug for Ppu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LCDC:{:08b} STAT:{:08b} LY:{:02x}",
            self.read_lcdc(),
            self.read_stat(),
            self.ly,
        )
    }
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            skip_frame: false,
            lcdc: 0.into(),
            lyc_compare: false,
            stat: 0.into(),
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            wy: 0,
            wx: 0,
            bgp: 0.into(),
            obp0: 0.into(),
            obp1: 0.into(),
            mode: Mode::default(),
            vram: [0x0; 0x2000],
            oam: [0x0; 0x100],
            screen: screen::Screen::default(),
            ldc_event: None,
        }
    }

    pub fn tick(&mut self, interrupt_line: &mut dyn InterruptLine) -> Option<screen::Event> {
        let (new_mode, screen_event) = self.mode.execute(self, interrupt_line);
        self.mode = new_mode;
        let lcd_event = self.ldc_event.take();
        screen_event.or(lcd_event)
    }

    pub fn check_lyc_compare(&mut self, interrupt_line: &mut dyn InterruptLine) {
        if self.ly != self.lyc {
            self.lyc_compare = false;
        } else {
            self.lyc_compare = true;

            if self.stat.lyc_interrupt_enabled() {
                interrupt_line.request(InterruptKind::Stat);
            }
        }
    }

    pub fn read_vram_byte(&self, address: u16) -> u8 {
        match self.mode {
            Mode::Drawing { .. } if self.lcdc.lcd_enabled() => 0xFF,
            _ => self.vram[address as usize],
        }
    }

    pub fn write_vram_byte(&mut self, address: u16, value: u8) {
        match self.mode {
            Mode::Drawing { .. } if self.lcdc.lcd_enabled() => {}
            _ => self.vram[address as usize] = value,
        }
    }

    pub fn read_oam_byte(&self, address: u16) -> u8 {
        match self.mode {
            Mode::OAMScan { .. } | Mode::Drawing { .. } if self.lcdc.lcd_enabled() => 0xFF,
            _ => self.oam[address as usize],
        }
    }

    pub fn write_oam_byte(&mut self, address: u16, value: u8) {
        match self.mode {
            Mode::OAMScan { .. } | Mode::Drawing { .. } if self.lcdc.lcd_enabled() => {}
            _ => self.oam[address as usize] = value,
        }
    }

    pub fn read_lcdc(&self) -> u8 {
        self.lcdc.into()
    }

    pub fn write_lcdc(&mut self, value: u8) {
        let new_lcdc: registers::Lcdc = value.into();
        if new_lcdc != self.lcdc {
            if new_lcdc.lcd_enabled() != self.lcdc.lcd_enabled() {
                if new_lcdc.lcd_enabled() {
                    self.ldc_event = Some(screen::Event::LCDOn);
                    self.skip_frame = true;

                    self.mode = Mode::default();
                } else {
                    self.ldc_event = Some(screen::Event::LCDOff);
                    self.screen.off();

                    self.mode = Mode::HBlank {
                        elapsed_cycles: 0,
                        wy_match_ly: false,
                        win_ly: 0,
                    };

                    self.ly = 0;
                    self.lyc_compare = true;
                }
            }

            self.lcdc = value.into();
        }
    }

    pub fn read_stat(&self) -> u8 {
        let mode_bits = match self.mode {
            Mode::HBlank { .. } => 0b00,
            Mode::VBlank { .. } => 0b01,
            Mode::OAMScan { .. } => 0b10,
            Mode::Drawing { .. } => 0b11,
        };

        let coincidence_flag = if self.lyc_compare { 0b100 } else { 0b000 };

        mode_bits | coincidence_flag | u8::from(self.stat) | 0b10000000
    }

    pub fn write_stat(&mut self, value: u8) {
        // Remove some bits:
        //   bit 0-1: read only corresponding to the mode
        //   bit 2: read only corresponding to the coincidence flag
        //   bit 7: unused, always set to 1
        self.stat = (value & 0b01111000).into()
    }

    pub fn read_scy(&self) -> u8 {
        self.scy
    }

    pub fn write_scy(&mut self, value: u8) {
        self.scy = value
    }

    pub fn read_scx(&self) -> u8 {
        self.scx
    }

    pub fn write_scx(&mut self, value: u8) {
        self.scx = value
    }

    pub fn read_ly(&self) -> u8 {
        self.ly
    }

    pub fn write_ly(&mut self, _: u8) {
        self.ly = 0
    }

    pub fn read_lyc(&self) -> u8 {
        self.lyc
    }

    pub fn write_lyc(&mut self, value: u8) {
        self.lyc = value
    }

    pub fn read_bgp(&self) -> u8 {
        self.bgp.into()
    }

    pub fn write_bgp(&mut self, value: u8) {
        self.bgp = value.into()
    }

    pub fn read_obp0(&self) -> u8 {
        self.obp0.into()
    }

    pub fn write_obp0(&mut self, value: u8) {
        self.obp0 = value.into()
    }
    pub fn read_obp1(&self) -> u8 {
        self.obp1.into()
    }

    pub fn write_obp1(&mut self, value: u8) {
        self.obp1 = value.into()
    }

    pub fn read_wy(&self) -> u8 {
        self.wy
    }

    pub fn write_wy(&mut self, value: u8) {
        self.wy = value
    }

    pub fn read_wx(&self) -> u8 {
        self.wx
    }

    pub fn write_wx(&mut self, value: u8) {
        self.wx = value
    }

    pub fn screen(&self) -> &screen::Screen {
        &self.screen
    }
}
