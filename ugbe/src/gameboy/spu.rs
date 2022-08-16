mod frame_sequencer;
mod frequency_sweep;
mod length_counter;
mod noise;
mod sample;
mod square;
mod volume_envelope;
mod wave;

pub use sample::Output as OutputSample;
pub type SampleFrame = sample::Frame<OutputSample>;

pub const SAMPLE_RATE: usize = super::clock::FREQUENCY / 2;

trait Voice {
    fn enabled(&self) -> bool;
    fn sample(&self) -> sample::Voice;
}

type Voice1 = square::SquareWaveVoice<true>;
type Voice2 = square::SquareWaveVoice<false>;
type Voice3 = wave::WaveVoice;
type Voice4 = noise::NoiseVoice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spu {
    enabled: bool,
    left_volume: u8,
    right_volume: u8,

    voice1: Voice1,
    voice1_left_enabled: bool,
    voice1_right_enabled: bool,

    voice2: Voice2,
    voice2_left_enabled: bool,
    voice2_right_enabled: bool,

    voice3: Voice3,
    voice3_left_enabled: bool,
    voice3_right_enabled: bool,

    voice4: Voice4,
    voice4_left_enabled: bool,
    voice4_right_enabled: bool,
}

impl Default for Spu {
    fn default() -> Self {
        Self::new()
    }
}

impl Spu {
    pub fn new() -> Self {
        Self {
            enabled: false,

            left_volume: 0,
            right_volume: 0,

            voice1: square::SquareWaveVoice::new(),
            voice1_left_enabled: false,
            voice1_right_enabled: false,

            voice2: square::SquareWaveVoice::new(),
            voice2_left_enabled: false,
            voice2_right_enabled: false,

            voice3: wave::WaveVoice::new(),
            voice3_left_enabled: false,
            voice3_right_enabled: false,

            voice4: noise::NoiseVoice::new(),
            voice4_left_enabled: false,
            voice4_right_enabled: false,
        }
    }

    pub fn tick(&mut self) {
        if self.enabled {
            self.voice1.tick();
            self.voice2.tick();
            self.voice3.tick();
            self.voice4.tick();
        }
    }

    fn dac(voice: &impl Voice) -> sample::Dac {
        if !voice.enabled() {
            return sample::Dac::silence();
        }

        sample::Dac::from_voice(voice.sample())
    }

    fn mix(&self) -> sample::Frame<sample::Dac> {
        let mut left = sample::Dac::silence();
        let mut right = sample::Dac::silence();

        let voice1 = Self::dac(&self.voice1);
        let voice2 = Self::dac(&self.voice2);
        let voice3 = Self::dac(&self.voice3);
        let voice4 = Self::dac(&self.voice4);

        if self.voice1_left_enabled {
            left += voice1 / 4.0;
        }

        if self.voice1_right_enabled {
            right += voice1 / 4.0;
        }

        if self.voice2_left_enabled {
            left += voice2 / 4.0;
        }

        if self.voice2_right_enabled {
            right += voice2 / 4.0;
        }

        if self.voice3_left_enabled {
            left += voice3 / 4.0;
        }

        if self.voice3_right_enabled {
            right += voice3 / 4.0;
        }

        if self.voice4_left_enabled {
            left += voice4 / 4.0;
        }

        if self.voice4_right_enabled {
            right += voice4 / 4.0;
        }

        sample::Frame::new(left, right)
    }

    fn amplify(
        &self,
        sample_frame: sample::Frame<sample::Dac>,
    ) -> sample::Frame<sample::Amplified> {
        sample::Frame::new(
            sample::Amplified::from_dac(sample_frame.left(), self.left_volume),
            sample::Amplified::from_dac(sample_frame.right(), self.right_volume),
        )
    }

    pub fn sample_frame(&self) -> SampleFrame {
        if self.enabled {
            let sample_frame = self.amplify(self.mix());

            sample::Frame::new(
                OutputSample::from_amplified(sample_frame.left()),
                OutputSample::from_amplified(sample_frame.right()),
            )
        } else {
            SampleFrame::default()
        }
    }

    pub fn read_nr10(&self) -> u8 {
        self.voice1.read_register_0()
    }

    pub fn write_nr10(&mut self, value: u8) {
        self.voice1.write_register_0(value)
    }

    pub fn read_nr11(&self) -> u8 {
        self.voice1.read_register_1()
    }

    pub fn write_nr11(&mut self, value: u8) {
        self.voice1.write_register_1(value)
    }

    pub fn read_nr12(&self) -> u8 {
        self.voice1.read_register_2()
    }

    pub fn write_nr12(&mut self, value: u8) {
        self.voice1.write_register_2(value)
    }

    pub fn read_nr13(&self) -> u8 {
        self.voice1.read_register_3()
    }

    pub fn write_nr13(&mut self, value: u8) {
        self.voice1.write_register_3(value)
    }

    pub fn read_nr14(&self) -> u8 {
        self.voice1.read_register_4()
    }

    pub fn write_nr14(&mut self, value: u8) {
        self.voice1.write_register_4(value)
    }

    pub fn read_nr20(&self) -> u8 {
        self.voice2.read_register_0()
    }

    pub fn write_nr20(&mut self, value: u8) {
        self.voice2.write_register_0(value)
    }

    pub fn read_nr21(&self) -> u8 {
        self.voice2.read_register_1()
    }

    pub fn write_nr21(&mut self, value: u8) {
        self.voice2.write_register_1(value)
    }

    pub fn read_nr22(&self) -> u8 {
        self.voice2.read_register_2()
    }

    pub fn write_nr22(&mut self, value: u8) {
        self.voice2.write_register_2(value)
    }

    pub fn read_nr23(&self) -> u8 {
        self.voice2.read_register_3()
    }

    pub fn write_nr23(&mut self, value: u8) {
        self.voice2.write_register_3(value)
    }

    pub fn read_nr24(&self) -> u8 {
        self.voice2.read_register_4()
    }

    pub fn write_nr24(&mut self, value: u8) {
        self.voice2.write_register_4(value)
    }

    pub fn read_nr30(&self) -> u8 {
        self.voice3.read_register_0()
    }

    pub fn write_nr30(&mut self, value: u8) {
        self.voice3.write_register_0(value)
    }

    pub fn read_nr31(&self) -> u8 {
        self.voice3.read_register_1()
    }

    pub fn write_nr31(&mut self, value: u8) {
        self.voice3.write_register_1(value)
    }

    pub fn read_nr32(&self) -> u8 {
        self.voice3.read_register_2()
    }

    pub fn write_nr32(&mut self, value: u8) {
        self.voice3.write_register_2(value)
    }

    pub fn read_nr33(&self) -> u8 {
        self.voice3.read_register_3()
    }

    pub fn write_nr33(&mut self, value: u8) {
        self.voice3.write_register_3(value)
    }

    pub fn read_nr34(&self) -> u8 {
        self.voice3.read_register_4()
    }

    pub fn write_nr34(&mut self, value: u8) {
        self.voice3.write_register_4(value)
    }

    pub fn read_wav_ram(&self, address: u16) -> u8 {
        self.voice3.read_ram(address)
    }

    pub fn write_wav_ram(&mut self, address: u16, value: u8) {
        self.voice3.write_ram(address, value)
    }

    pub fn read_nr40(&self) -> u8 {
        self.voice4.read_register_0()
    }

    pub fn write_nr40(&mut self, value: u8) {
        self.voice4.write_register_0(value)
    }

    pub fn read_nr41(&self) -> u8 {
        self.voice4.read_register_1()
    }

    pub fn write_nr41(&mut self, value: u8) {
        self.voice4.write_register_1(value)
    }

    pub fn read_nr42(&self) -> u8 {
        self.voice4.read_register_2()
    }

    pub fn write_nr42(&mut self, value: u8) {
        self.voice4.write_register_2(value)
    }

    pub fn read_nr43(&self) -> u8 {
        self.voice4.read_register_3()
    }

    pub fn write_nr43(&mut self, value: u8) {
        self.voice4.write_register_3(value)
    }

    pub fn read_nr44(&self) -> u8 {
        self.voice4.read_register_4()
    }

    pub fn write_nr44(&mut self, value: u8) {
        self.voice4.write_register_4(value)
    }

    pub fn read_nr50(&self) -> u8 {
        ((self.left_volume & 0b111) << 4) | (self.right_volume & 0b111)
    }

    pub fn write_nr50(&mut self, value: u8) {
        self.left_volume = (value >> 4) & 0b111;
        self.right_volume = value & 0b111;
    }

    pub fn read_nr51(&self) -> u8 {
        ((self.voice4_left_enabled as u8) << 7)
            | ((self.voice3_left_enabled as u8) << 6)
            | ((self.voice2_left_enabled as u8) << 5)
            | ((self.voice1_left_enabled as u8) << 4)
            | ((self.voice4_right_enabled as u8) << 3)
            | ((self.voice3_right_enabled as u8) << 2)
            | ((self.voice2_right_enabled as u8) << 1)
            | (self.voice1_right_enabled as u8)
    }

    pub fn write_nr51(&mut self, value: u8) {
        self.voice4_left_enabled = (value >> 7) & 0b1 == 1;
        self.voice3_left_enabled = (value >> 6) & 0b1 == 1;
        self.voice2_left_enabled = (value >> 5) & 0b1 == 1;
        self.voice1_left_enabled = (value >> 4) & 0b1 == 1;

        self.voice4_right_enabled = (value >> 3) & 0b1 == 1;
        self.voice3_right_enabled = (value >> 2) & 0b1 == 1;
        self.voice2_right_enabled = (value >> 1) & 0b1 == 1;
        self.voice1_right_enabled = value & 0b1 == 1;
    }

    pub fn read_nr52(&self) -> u8 {
        ((self.enabled as u8) << 7)
            | 0b1110000
            | ((self.voice4.enabled() as u8) << 3)
            | ((self.voice3.enabled() as u8) << 2)
            | ((self.voice2.enabled() as u8) << 1)
            | (self.voice1.enabled() as u8)
    }

    pub fn write_nr52(&mut self, value: u8) {
        self.enabled = (value >> 7) & 0b1 == 1;

        if !self.enabled {
            self.voice1 = square::SquareWaveVoice::new();
            self.voice2 = square::SquareWaveVoice::new();
            self.voice3 = wave::WaveVoice::new();
            self.voice4 = noise::NoiseVoice::new();

            self.left_volume = 0;
            self.right_volume = 0;

            self.voice4_left_enabled = false;
            self.voice3_left_enabled = false;
            self.voice2_left_enabled = false;
            self.voice1_left_enabled = false;

            self.voice4_right_enabled = false;
            self.voice3_right_enabled = false;
            self.voice2_right_enabled = false;
            self.voice1_right_enabled = false;
        }
    }
}
