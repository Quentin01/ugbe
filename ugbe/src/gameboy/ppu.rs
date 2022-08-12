use std::fmt::Debug;

use self::oam::SpriteNo;

use super::components::{InterruptKind, InterruptLine};

mod color;
mod fetcher;
mod fifo;
mod oam;
mod registers;
pub mod screen;
mod tiling;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Mode {
    OAMScan {
        sprite_buffer: [Option<oam::Sprite>; 10],
        sprite_buffer_idx: usize,
        wy_match_ly: bool,
        sprite_no: oam::SpriteNo,
        current_sprite: Option<oam::Sprite>,
        win_ly: u8,
    },
    Drawing {
        sprite_buffer: [Option<oam::Sprite>; 10],
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
            sprite_no: SpriteNo::first(),
            current_sprite: None,
            win_ly: 0,
        }
    }
}

impl Mode {
    fn execute(
        &mut self,
        ppu_ctx: &mut Context,
        interrupt_line: &mut dyn InterruptLine,
    ) -> Option<screen::Event> {
        // If the LCD is OFF, we don't have to do anything
        if !ppu_ctx.lcdc.lcd_enabled() {
            return None;
        }

        match self {
            Self::OAMScan {
                sprite_buffer,
                sprite_buffer_idx,
                wy_match_ly,
                sprite_no,
                current_sprite,
                win_ly,
            } => {
                // Each check of sprite is done on two T-cycles, as we have 40 sprites in the OAM this mode uses 80 T-cycles.
                match current_sprite {
                    None => {
                        // During even T-cycles we will just fetch the sprite
                        let sprite = ppu_ctx.oam.sprite(*sprite_no);

                        if ppu_ctx.wy == ppu_ctx.ly {
                            *wy_match_ly = true;
                        }

                        *sprite_no = sprite_no.wrapping_inc();
                        *current_sprite = Some(sprite);

                        None
                    }
                    Some(sprite) => {
                        // During odd T-cycles we are checking if the sprite should be added to the sprite buffer
                        if *sprite_buffer_idx >= 10 {
                            if sprite.no() >= oam::SpriteNo::last() {
                                *self = Self::switch_from_oam_scan_to_drawing(
                                    ppu_ctx,
                                    *sprite_buffer,
                                    *wy_match_ly,
                                    *win_ly,
                                );

                                return None;
                            } else {
                                *current_sprite = None;

                                return None;
                            }
                        }

                        let sprite_height = ppu_ctx.lcdc.sprite_height();

                        let sprite_y_range_on_screen = if sprite.y() >= 16 {
                            let sprite_y_on_screen = sprite.y() - 16;
                            sprite_y_on_screen..(sprite_y_on_screen + sprite_height)
                        } else if sprite_height + sprite.y() > 16 {
                            0..(sprite_height + sprite.y() - 16)
                        } else {
                            0..0
                        };

                        if sprite_y_range_on_screen.contains(&ppu_ctx.ly) {
                            sprite_buffer[*sprite_buffer_idx] = Some(*sprite);
                            *sprite_buffer_idx += 1;
                        }

                        if sprite.no() >= oam::SpriteNo::last() {
                            *self = Self::switch_from_oam_scan_to_drawing(
                                ppu_ctx,
                                *sprite_buffer,
                                *wy_match_ly,
                                *win_ly,
                            );

                            None
                        } else {
                            *current_sprite = None;

                            None
                        }
                    }
                }
            }
            Self::Drawing {
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
            } => {
                *elapsed_cycles += 1;

                if sprite_fetcher.is_none() {
                    // Fetch the sprite that could be render starting at lx
                    // For that we need to disable the warning of clippy telling us that our loop is executed only once as we don't have labelled block yet
                    #[allow(clippy::never_loop)]
                    let sprite_to_render = 'fetch_sprite_to_render: loop {
                        if !ppu_ctx.lcdc.display_sprite() {
                            break 'fetch_sprite_to_render None;
                        }

                        for sprite_opt in sprite_buffer.iter_mut() {
                            match sprite_opt {
                                Some(sprite) => {
                                    let lx_start = if sprite.x() >= 8 {
                                        Some(sprite.x() - 8)
                                    } else if sprite.x() + 8 > 8 {
                                        Some(0)
                                    } else {
                                        None
                                    };

                                    if lx_start == Some(*lx) {
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
                            *sprite_fetcher = Some(fetcher::SpriteFetcher::new(sprite, ppu_ctx));
                        }
                        None => {}
                    }
                }

                if let Some(fetcher) = sprite_fetcher {
                    if fetcher.tick(ppu_ctx, *lx, sprite_fifo) {
                        *sprite_fetcher = None;
                    }

                    return None;
                }

                // Check if we need to start fetching the window
                if *scx_delay == 0
                    && ppu_ctx.lcdc.display_window()
                    && *wy_match_ly
                    && *lx >= ppu_ctx.wx - 7
                    && !*win_fetcher
                {
                    *bg_win_fetcher = fetcher::BackgroundWindowFetcher::new(
                        ppu_ctx.lcdc.window_tile_map(),
                        tiling::PixelPosition::new(0, *win_ly as usize),
                    );

                    bg_win_fifo.clear();

                    *win_fetcher = true;
                    *win_ly += 1;
                }

                bg_win_fetcher.tick(ppu_ctx, bg_win_fifo);

                if bg_win_fifo.len() > 0 {
                    if *scx_delay != 0 {
                        bg_win_fifo.pop();
                        *scx_delay -= 1;
                    } else {
                        let bg_pixel = bg_win_fifo.pop();

                        let pixel_color = if sprite_fifo.len() > 0 {
                            let sprite_pixel = sprite_fifo.pop();

                            if sprite_pixel.is_zero()
                                || (!sprite_pixel.over_bg_and_win() && !bg_pixel.is_zero())
                            {
                                bg_pixel.color()
                            } else {
                                sprite_pixel.color(ppu_ctx)
                            }
                        } else {
                            bg_pixel.color()
                        };

                        ppu_ctx
                            .screen
                            .set_pixel((*lx).into(), ppu_ctx.ly.into(), pixel_color);

                        *lx = lx.wrapping_add(1)
                    }
                }

                if *lx >= 160 {
                    *self = Self::switch_from_drawing_to_hblank(
                        ppu_ctx,
                        interrupt_line,
                        *elapsed_cycles,
                        *wy_match_ly,
                        *win_ly,
                    );

                    None
                } else {
                    None
                }
            }
            Self::HBlank {
                elapsed_cycles,
                wy_match_ly,
                win_ly,
            } => {
                *elapsed_cycles += 1;

                // A line is 456 T-cycles but elapsed_cycles started counting after the OAM scan which have a duration of 80 T-cycles
                if *elapsed_cycles >= 456 - 80 {
                    if ppu_ctx.ly == 143 {
                        *self = Self::switch_from_hblank_to_vblank(ppu_ctx, interrupt_line);

                        Some(screen::Event::VBlank)
                    } else {
                        *self = Self::switch_from_hblank_to_oam_scan(
                            ppu_ctx,
                            interrupt_line,
                            *wy_match_ly,
                            *win_ly,
                        );

                        None
                    }
                } else {
                    None
                }
            }
            Self::VBlank {
                elapsed_cycles_line,
                ly,
            } => {
                *elapsed_cycles_line += 1;

                if *elapsed_cycles_line == 456 {
                    if *ly == 153 {
                        *self = Self::switch_from_vblank_to_oam_scan(ppu_ctx, interrupt_line);

                        None
                    } else {
                        *ly += 1;
                        *elapsed_cycles_line = 0;

                        ppu_ctx.ly += 1;
                        ppu_ctx.check_lyc_compare(interrupt_line);

                        None
                    }
                } else {
                    if *elapsed_cycles_line == 4 && *ly == 153 {
                        // Simulate that the VBlank change LY to 0 after 4 T-cycles in the line 153
                        ppu_ctx.ly = 0;
                        ppu_ctx.check_lyc_compare(interrupt_line);
                    }

                    None
                }
            }
        }
    }

    fn switch_from_oam_scan_to_drawing(
        ppu_ctx: &mut Context,
        mut sprite_buffer: [Option<oam::Sprite>; 10],
        wy_match_ly: bool,
        win_ly: u8,
    ) -> Self {
        sprite_buffer.sort_by(|a, b| match a {
            Some(a) => match b {
                Some(b) => a.x().cmp(&b.x()),
                None => std::cmp::Ordering::Greater,
            },
            None => match b {
                Some(_) => std::cmp::Ordering::Less,
                None => std::cmp::Ordering::Equal,
            },
        });

        Self::Drawing {
            sprite_buffer,
            wy_match_ly,
            scx_delay: ppu_ctx.scx % 8,
            lx: 0,
            elapsed_cycles: 0,
            win_fetcher: false,
            bg_win_fetcher: fetcher::BackgroundWindowFetcher::new(
                ppu_ctx.lcdc.bg_tile_map(),
                tiling::PixelPosition::new(
                    ppu_ctx.scx as usize,
                    ppu_ctx.ly as usize + ppu_ctx.scy as usize,
                ),
            ),
            bg_win_fifo: fifo::Fifo::new(),
            sprite_fetcher: None,
            sprite_fifo: fifo::Fifo::new(),
            win_ly,
        }
    }

    fn switch_from_drawing_to_hblank(
        ppu_ctx: &mut Context,
        interrupt_line: &mut dyn InterruptLine,
        elapsed_cycles: usize,
        wy_match_ly: bool,
        win_ly: u8,
    ) -> Self {
        if ppu_ctx.stat.hblank_interrupt_enabled() {
            interrupt_line.request(InterruptKind::Stat);
        }

        Self::HBlank {
            elapsed_cycles,
            win_ly,
            wy_match_ly,
        }
    }

    fn switch_from_hblank_to_vblank(
        ppu_ctx: &mut Context,
        interrupt_line: &mut dyn InterruptLine,
    ) -> Self {
        interrupt_line.request(InterruptKind::VBlank);
        if ppu_ctx.stat.vblank_interrupt_enabled() {
            interrupt_line.request(InterruptKind::Stat);
        }
        if ppu_ctx.stat.oam_scanning_interrupt_enabled() {
            interrupt_line.request(InterruptKind::Stat);
        }

        if ppu_ctx.skip_frame {
            ppu_ctx.skip_frame = false;
        } else {
            ppu_ctx.screen.commit_frame()
        }

        ppu_ctx.ly += 1;
        ppu_ctx.check_lyc_compare(interrupt_line);

        Self::VBlank {
            elapsed_cycles_line: 0,
            ly: ppu_ctx.ly,
        }
    }

    fn switch_from_hblank_to_oam_scan(
        ppu_ctx: &mut Context,
        interrupt_line: &mut dyn InterruptLine,
        wy_match_ly: bool,
        win_ly: u8,
    ) -> Self {
        if ppu_ctx.stat.oam_scanning_interrupt_enabled() {
            interrupt_line.request(InterruptKind::Stat);
        }

        ppu_ctx.ly += 1;
        ppu_ctx.check_lyc_compare(interrupt_line);

        Self::OAMScan {
            sprite_buffer: [None; 10],
            sprite_buffer_idx: 0,
            wy_match_ly,
            sprite_no: oam::SpriteNo::first(),
            current_sprite: None,
            win_ly,
        }
    }

    fn switch_from_vblank_to_oam_scan(
        ppu_ctx: &mut Context,
        interrupt_line: &mut dyn InterruptLine,
    ) -> Self {
        if ppu_ctx.stat.oam_scanning_interrupt_enabled() {
            interrupt_line.request(InterruptKind::Stat);
        }

        Self::default()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Context {
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
    bgp: color::Palette,
    obp0: color::Palette,
    obp1: color::Palette,
    vram: [u8; 0x2000],
    oam: oam::Oam,
    screen: screen::Screen,
}

impl Context {
    pub fn new(screen_config: screen::Config) -> Self {
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
            vram: [0x0; 0x2000],
            oam: oam::Oam::new(),
            screen: screen::Screen::new(screen_config),
        }
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
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ppu {
    mode: Mode,
    ctx: Context,
    pending_lcd_event: Option<screen::Event>,
}

impl Debug for Ppu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LCDC:{:08b} STAT:{:08b} LY:{:02x}",
            self.read_lcdc(),
            self.read_stat(),
            self.ctx.ly,
        )
    }
}

impl Ppu {
    pub fn new(screen_config: screen::Config) -> Self {
        Self {
            mode: Mode::default(),
            ctx: Context::new(screen_config),
            pending_lcd_event: None,
        }
    }

    pub fn tick(&mut self, interrupt_line: &mut dyn InterruptLine) -> Option<screen::Event> {
        let screen_event = self.mode.execute(&mut self.ctx, interrupt_line);
        let lcd_event = self.pending_lcd_event.take();
        screen_event.or(lcd_event)
    }

    pub fn read_vram_byte(&self, address: u16) -> u8 {
        match self.mode {
            Mode::Drawing { .. } if self.ctx.lcdc.lcd_enabled() => 0xFF,
            _ => self.ctx.vram[address as usize],
        }
    }

    pub fn write_vram_byte(&mut self, address: u16, value: u8) {
        match self.mode {
            Mode::Drawing { .. } if self.ctx.lcdc.lcd_enabled() => {}
            _ => self.ctx.vram[address as usize] = value,
        }
    }

    pub fn read_oam_byte(&self, address: u16) -> u8 {
        match self.mode {
            Mode::OAMScan { .. } | Mode::Drawing { .. } if self.ctx.lcdc.lcd_enabled() => 0xFF,
            _ => self.ctx.oam.read_byte(address),
        }
    }

    pub fn write_oam_byte(&mut self, address: u16, value: u8) {
        match self.mode {
            Mode::OAMScan { .. } | Mode::Drawing { .. } if self.ctx.lcdc.lcd_enabled() => {}
            _ => self.ctx.oam.write_byte(address, value),
        }
    }

    pub fn read_lcdc(&self) -> u8 {
        self.ctx.lcdc.into()
    }

    pub fn write_lcdc(&mut self, value: u8) {
        let new_lcdc: registers::Lcdc = value.into();
        if new_lcdc != self.ctx.lcdc {
            if new_lcdc.lcd_enabled() != self.ctx.lcdc.lcd_enabled() {
                if new_lcdc.lcd_enabled() {
                    self.pending_lcd_event = Some(screen::Event::LCDOn);
                    self.ctx.skip_frame = true;

                    self.mode = Mode::default();
                } else {
                    self.pending_lcd_event = Some(screen::Event::LCDOff);
                    self.ctx.screen.off();

                    self.mode = Mode::HBlank {
                        elapsed_cycles: 0,
                        wy_match_ly: false,
                        win_ly: 0,
                    };

                    self.ctx.ly = 0;
                    self.ctx.lyc_compare = true;
                }
            }

            self.ctx.lcdc = value.into();
        }
    }

    pub fn read_stat(&self) -> u8 {
        let mode_bits = match self.mode {
            Mode::HBlank { .. } => 0b00,
            Mode::VBlank { .. } => 0b01,
            Mode::OAMScan { .. } => 0b10,
            Mode::Drawing { .. } => 0b11,
        };

        let coincidence_flag = if self.ctx.lyc_compare { 0b100 } else { 0b000 };

        mode_bits | coincidence_flag | u8::from(self.ctx.stat) | 0b10000000
    }

    pub fn write_stat(&mut self, value: u8) {
        // Remove some bits:
        //   bit 0-1: read only corresponding to the mode
        //   bit 2: read only corresponding to the coincidence flag
        //   bit 7: unused, always set to 1
        self.ctx.stat = (value & 0b01111000).into()
    }

    pub fn read_scy(&self) -> u8 {
        self.ctx.scy
    }

    pub fn write_scy(&mut self, value: u8) {
        self.ctx.scy = value
    }

    pub fn read_scx(&self) -> u8 {
        self.ctx.scx
    }

    pub fn write_scx(&mut self, value: u8) {
        self.ctx.scx = value
    }

    pub fn read_ly(&self) -> u8 {
        self.ctx.ly
    }

    pub fn write_ly(&mut self, _: u8) {
        self.ctx.ly = 0
    }

    pub fn read_lyc(&self) -> u8 {
        self.ctx.lyc
    }

    pub fn write_lyc(&mut self, value: u8) {
        self.ctx.lyc = value
    }

    pub fn read_bgp(&self) -> u8 {
        self.ctx.bgp.into()
    }

    pub fn write_bgp(&mut self, value: u8) {
        self.ctx.bgp = value.into()
    }

    pub fn read_obp0(&self) -> u8 {
        self.ctx.obp0.into()
    }

    pub fn write_obp0(&mut self, value: u8) {
        self.ctx.obp0 = value.into()
    }
    pub fn read_obp1(&self) -> u8 {
        self.ctx.obp1.into()
    }

    pub fn write_obp1(&mut self, value: u8) {
        self.ctx.obp1 = value.into()
    }

    pub fn read_wy(&self) -> u8 {
        self.ctx.wy
    }

    pub fn write_wy(&mut self, value: u8) {
        self.ctx.wy = value
    }

    pub fn read_wx(&self) -> u8 {
        self.ctx.wx
    }

    pub fn write_wx(&mut self, value: u8) {
        self.ctx.wx = value
    }

    pub fn screen(&self) -> &screen::Screen {
        &self.ctx.screen
    }
}
