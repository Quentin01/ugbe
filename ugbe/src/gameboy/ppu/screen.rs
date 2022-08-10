#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrameBlending {
    Interframe(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ColorPalette {
    off: Color,
    dmg_white: Color,
    dmg_light_gray: Color,
    dmg_dark_gray: Color,
    dmg_black: Color,
}

impl ColorPalette {
    pub fn new(
        off: Color,
        dmg_white: Color,
        dmg_light_gray: Color,
        dmg_dark_gray: Color,
        dmg_black: Color,
    ) -> ColorPalette {
        Self {
            off,
            dmg_white,
            dmg_light_gray,
            dmg_dark_gray,
            dmg_black,
        }
    }

    pub fn new_grayscale() -> Self {
        Self::new(
            Color::new(31, 31, 31),
            Color::new(31, 31, 31),
            Color::new(21, 21, 21),
            Color::new(11, 11, 11),
            Color::new(0, 0, 0),
        )
    }

    pub fn new_legacy() -> Self {
        Self::new(
            Color::from_rgb24(0x8b, 0x92, 0x26),
            Color::from_rgb24(0x7F, 0x86, 0x0F),
            Color::from_rgb24(0x57, 0x7C, 0x44),
            Color::from_rgb24(0x36, 0x5d, 0x48),
            Color::from_rgb24(0x2a, 0x45, 0x3b),
        )
    }

    fn color(&self, color: super::color::Color) -> Color {
        match color {
            super::color::Color::Dmg(dmg_color) => match dmg_color {
                super::color::DMGColor::White => self.dmg_white,
                super::color::DMGColor::LightGray => self.dmg_light_gray,
                super::color::DMGColor::DarkGray => self.dmg_dark_gray,
                super::color::DMGColor::Black => self.dmg_black,
            },
            super::color::Color::Off => self.off,
        }
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::new_legacy()
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    frame_blending: Option<FrameBlending>,
    color_palette: ColorPalette,
}

impl Config {
    pub fn new(frame_blending: Option<FrameBlending>, color_palette: ColorPalette) -> Self {
        Self {
            frame_blending,
            color_palette,
        }
    }

    pub fn set_frame_blending(&mut self, frame_blending: Option<FrameBlending>) {
        self.frame_blending = frame_blending;
    }

    pub fn set_color_palette(&mut self, color_palette: ColorPalette) {
        self.color_palette = color_palette;
    }
}

/// RGB555 color
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Color(u16);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u16 & 0x1F) << 10) | ((g as u16 & 0x1F) << 5) | (b as u16 & 0x1F))
    }

    pub fn from_rgb24(r: u8, g: u8, b: u8) -> Self {
        Self::new(
            ((r as u16 * 0x1F) / 0xFF) as u8,
            ((g as u16 * 0x1F) / 0xFF) as u8,
            ((b as u16 * 0x1F) / 0xFF) as u8,
        )
    }

    pub fn red(&self) -> u8 {
        ((self.0 >> 10) & 0x1F) as u8
    }

    pub fn green(&self) -> u8 {
        ((self.0 >> 5) & 0x1F) as u8
    }

    pub fn blue(&self) -> u8 {
        (self.0 & 0x1F) as u8
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Frame {
    pixels: [super::color::Color; Screen::WIDTH * Screen::HEIGHT],
}

impl Frame {
    fn new() -> Self {
        Self {
            pixels: [super::color::Color::Off; Screen::WIDTH * Screen::HEIGHT],
        }
    }

    fn off(&mut self) {
        self.pixels = [super::color::Color::Off; Screen::WIDTH * Screen::HEIGHT];
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Screen {
    config: Config,
    idx_frame: usize,
    frames: Vec<Frame>,
    frame_being_draw: Frame,
    pixels: [Color; Self::WIDTH * Self::HEIGHT],
}

impl Screen {
    pub const WIDTH: usize = 160;
    pub const HEIGHT: usize = 144;

    pub fn new(config: Config) -> Self {
        let required_frame_count = match config.frame_blending {
            Some(frame_blending) => match frame_blending {
                FrameBlending::Interframe(count) => count,
            },
            None => 1,
        };

        let mut screen = Self {
            config,
            idx_frame: 0,
            frames: vec![Frame::new(); required_frame_count],
            frame_being_draw: Frame::new(),
            pixels: [Color::new(0, 0, 0); Self::WIDTH * Self::HEIGHT],
        };

        screen.off();

        screen
    }

    pub(super) fn set_pixel(&mut self, x: usize, y: usize, color: super::color::Color) {
        debug_assert!(x < Screen::WIDTH && y < Screen::HEIGHT);
        self.frame_being_draw.pixels[y * Screen::WIDTH + x] = color;
    }

    pub(super) fn commit_frame(&mut self) {
        self.idx_frame = (self.idx_frame + 1) % self.frames.len();
        self.frames[self.idx_frame] = self.frame_being_draw;
        self.frame_being_draw = Frame::new();

        self.update_pixels()
    }

    pub fn update_pixels(&mut self) {
        match self.config.frame_blending {
            Some(frame_blending) => match frame_blending {
                FrameBlending::Interframe(frame_count) => {
                    for x in 0..Self::WIDTH {
                        for y in 0..Self::HEIGHT {
                            self.pixels[y * Screen::WIDTH + x] = {
                                let mut total_coeff = 0.0;

                                let mut red = 0.0;
                                let mut green = 0.0;
                                let mut blue = 0.0;

                                // Go from frame to frames from the older one first
                                for offset in (0..frame_count).rev() {
                                    let idx = (self.idx_frame + offset) % frame_count;

                                    // More recent frames have less influence on the frame
                                    let coeff = 1.0
                                        - ((frame_count - offset - 1) as f64
                                            * (1.0 / frame_count as f64));
                                    total_coeff += coeff;

                                    let color = self
                                        .config
                                        .color_palette
                                        .color(self.frames[idx].pixels[y * Self::WIDTH + x]);

                                    red += color.red() as f64 * coeff;
                                    green += color.green() as f64 * coeff;
                                    blue += color.blue() as f64 * coeff;
                                }

                                Color::new(
                                    (red / total_coeff) as u8,
                                    (green / total_coeff) as u8,
                                    (blue / total_coeff) as u8,
                                )
                            }
                        }
                    }
                }
            },
            None => {
                for x in 0..Self::WIDTH {
                    for y in 0..Self::HEIGHT {
                        self.pixels[y * Screen::WIDTH + x] = self
                            .config
                            .color_palette
                            .color(self.frames[0].pixels[y * Self::WIDTH + x]);
                    }
                }
            }
        }
    }

    pub fn pixels(&self) -> &[Color; Self::WIDTH * Self::HEIGHT] {
        &self.pixels
    }

    pub(super) fn off(&mut self) {
        for frame in &mut self.frames {
            frame.off();
        }

        self.pixels =
            [self.config.color_palette.color(super::color::Color::Off); Self::WIDTH * Self::HEIGHT];
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    VBlank,
    LCDOn,
    LCDOff,
}
