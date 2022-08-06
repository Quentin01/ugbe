#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    Off,
    White,
    LightGray,
    DarkGray,
    Black,
}

#[derive(Debug, Clone)]
pub struct Screen {
    pixels: [Color; Self::WIDTH * Self::HEIGHT],
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            pixels: [Color::Off; Self::WIDTH * Self::HEIGHT],
        }
    }
}

impl Screen {
    pub const WIDTH: usize = 160;
    pub const HEIGHT: usize = 144;

    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        if x >= Self::WIDTH || y >= Self::HEIGHT {
            panic!(
                "Trying to retrieve a pixel with out-of-bound position: x={} / y={}",
                x, y
            );
        }

        let idx = y * Self::WIDTH + x;
        self.pixels[idx]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= Self::WIDTH || y >= Self::HEIGHT {
            panic!(
                "Trying to set a pixel with out-of-bound position: x={} / y={}",
                x, y
            );
        }

        let idx = y * Self::WIDTH + x;
        // TODO: Simulate LCD ghosting?
        self.pixels[idx] = color;
    }

    pub fn pixels(&self) -> &[Color; Self::WIDTH * Self::HEIGHT] {
        &self.pixels
    }

    pub fn off(&mut self) {
        self.pixels = [Color::Off; Self::WIDTH * Self::HEIGHT];
    }
}

pub trait Renderer {
    /// Called when the LCD screen is switch to on
    fn on(&mut self);

    /// Called when the LCD screen is switch to off
    fn off(&mut self);

    /// Called after the rendering of a frame
    fn vblank(&mut self, screen: &Screen);
}
