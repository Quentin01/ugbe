#[derive(Default, Debug, Clone, Copy)]
pub struct Frame<TYPE>
where
    TYPE: Default + std::fmt::Debug + Clone + Copy,
{
    left: TYPE,
    right: TYPE,
}

impl<TYPE> Frame<TYPE>
where
    TYPE: Default + std::fmt::Debug + Clone + Copy,
{
    pub fn new(left: TYPE, right: TYPE) -> Self {
        Self { left, right }
    }

    pub fn left(&self) -> TYPE {
        self.left
    }

    pub fn right(&self) -> TYPE {
        self.right
    }
}

/// Sample represented by a u8 in the range [0; 15]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Voice(u8);

impl Voice {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 15;

    pub fn new(value: u8) -> Self {
        debug_assert!((Self::MIN..=Self::MAX).contains(&value));
        Self(value)
    }
}

/// Sample represented by a f64 in the range [-1.0; 1.0]
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Dac(f64);

impl Dac {
    pub const MIN: f64 = -1.0;
    pub const MAX: f64 = 1.0;

    pub fn silence() -> Self {
        Self(0.0)
    }

    pub fn from_voice(voice_sample: Voice) -> Self {
        let value = (voice_sample.0 as f64 / 7.5) - 1.0;

        debug_assert!((Self::MIN..=Self::MAX).contains(&value));
        Self(value)
    }
}

impl std::ops::Div<f64> for Dac {
    type Output = Dac;

    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl std::ops::AddAssign for Dac {
    fn add_assign(&mut self, rhs: Self) {
        let value = self.0 + rhs.0;
        debug_assert!((Self::MIN..=Self::MAX).contains(&value));

        self.0 = value;
    }
}

/// Sample represented by a f64 in the range [-128.0; 128.0]
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Amplified(f64);

impl Amplified {
    pub const MIN: f64 = -128.0;
    pub const MAX: f64 = 128.0;

    pub fn new(value: f64) -> Self {
        debug_assert!((Self::MIN..=Self::MAX).contains(&value));
        Self(value)
    }

    pub fn from_dac(dac_sample: Dac, volume: u8) -> Self {
        debug_assert!((0..=7).contains(&volume));

        let value = dac_sample.0 * 2f64.powi(volume as i32);

        debug_assert!((Self::MIN..=Self::MAX).contains(&value));
        Self(value)
    }
}

/// Sample represented by a i8 in the range [-128; 127]
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Output(i8);

impl Output {
    pub const MIN: i8 = -128;
    pub const MAX: i8 = 127;

    pub fn new(value: i8) -> Self {
        debug_assert!((Self::MIN..=Self::MAX).contains(&value));
        Self(value)
    }

    pub fn from_amplified(amplified_sample: Amplified) -> Self {
        let slope = (Self::MAX as f64 - Self::MIN as f64) / (Amplified::MAX - Amplified::MIN);
        let value = ((i8::MIN as f64) + slope * (amplified_sample.0 - Amplified::MIN)) as i8;

        debug_assert!((Self::MIN..=Self::MAX).contains(&value));
        Self(value)
    }

    pub fn value(&self) -> i8 {
        self.0
    }
}
