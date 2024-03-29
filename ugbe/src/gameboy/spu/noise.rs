use super::frame_sequencer::FrameSequencer;
use super::length_counter::LengthCounter;
use super::sample::Voice as VoiceSample;
use super::volume_envelope::{EnvelopeDirection, VolumeEnvelope};
use super::Voice;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NoiseVoice {
    enabled: bool,
    dac_enabled: bool,

    length_counter: LengthCounter<6>,

    volume_envelope: VolumeEnvelope,

    frequency_div: u8,
    frequency_div_shift: u8,
    frequency_timer: usize,

    counter_width: u8,
    lfsr: u16,
}

impl NoiseVoice {
    pub fn new() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,

            length_counter: LengthCounter::new(),

            volume_envelope: VolumeEnvelope::new(),

            frequency_div: 0,
            frequency_div_shift: 0,
            frequency_timer: 0,

            counter_width: 0,
            lfsr: 0,
        }
    }

    pub fn tick(&mut self, frame_sequencer: &FrameSequencer) {
        self.length_counter.tick(frame_sequencer);
        if self.length_counter.enabled() && self.length_counter.value() == 0 {
            self.enabled = false;
        }

        if !self.enabled {
            return;
        }

        self.volume_envelope.tick(frame_sequencer);

        self.frequency_timer -= 1;
        if self.frequency_timer == 0 {
            self.frequency_timer = match self.frequency_div {
                0 => 8,
                1 => 16,
                2 => 32,
                3 => 48,
                4 => 64,
                5 => 80,
                6 => 96,
                7 => 112,
                _ => unreachable!(),
            } << self.frequency_div_shift;

            let lfsr_bit_0 = self.lfsr & 0b1;
            let lfsr_bit_1 = (self.lfsr >> 1) & 0b1;
            let new_bit = lfsr_bit_0 ^ lfsr_bit_1;

            self.lfsr >>= 1;
            self.lfsr &= !(1 << 14);
            self.lfsr |= new_bit << 14;

            if self.counter_width == 1 {
                self.lfsr &= !(1 << 6);
                self.lfsr |= new_bit << 6;
            }
        }
    }

    fn trigger(&mut self, frame_sequencer: &FrameSequencer) {
        if self.dac_enabled {
            self.enabled = true;
        }

        self.length_counter.trigger(frame_sequencer);

        self.volume_envelope.trigger();

        self.frequency_timer = match self.frequency_div {
            0 => 8,
            1 => 16,
            2 => 32,
            3 => 48,
            4 => 64,
            5 => 80,
            6 => 96,
            7 => 112,
            _ => unreachable!(),
        } << self.frequency_div_shift;

        self.lfsr = 0xFFFF;
    }

    pub fn reset(&mut self, frame_sequencer: &FrameSequencer) {
        self.enabled = false;
        self.dac_enabled = false;

        self.length_counter.enable(false, frame_sequencer);

        self.volume_envelope.reset();

        self.frequency_div = 0;
        self.frequency_div_shift = 0;
        self.frequency_timer = 0;

        self.counter_width = 0;
        self.lfsr = 0;
    }

    pub fn read_register_0(&self) -> u8 {
        0xFF
    }

    pub fn write_register_0(&mut self, _: u8) {}

    pub fn read_register_1(&self) -> u8 {
        0xFF
    }

    pub fn write_register_1(&mut self, value: u8) {
        self.length_counter.set_value(value & 0b0011_1111);
    }

    pub fn read_register_2(&self) -> u8 {
        ((self.volume_envelope.initial() & 0b1111) << 4)
            | (((self.volume_envelope.direction() == EnvelopeDirection::Increase) as u8) << 3)
            | (self.volume_envelope.period() & 0b0111)
    }

    pub fn write_register_2(&mut self, value: u8) {
        self.dac_enabled = (value >> 3) & 0b0001_1111 != 0;
        self.enabled = self.enabled && self.dac_enabled;

        self.volume_envelope.set_initial((value >> 4) & 0b1111);
        self.volume_envelope
            .set_direction(match (value >> 3) & 0b1 == 1 {
                true => EnvelopeDirection::Increase,
                false => EnvelopeDirection::Decrease,
            });
        self.volume_envelope.set_period(value & 0b0111);
    }

    pub fn read_register_3(&self) -> u8 {
        (self.frequency_div_shift & 0b1111) << 4
            | (self.counter_width & 0b1) << 3
            | (self.frequency_div & 0b0111)
    }

    pub fn write_register_3(&mut self, value: u8) {
        self.frequency_div_shift = (value >> 4) & 0b1111;
        self.counter_width = (value >> 3) & 0b1;
        self.frequency_div = value & 0b0111;
    }

    pub fn read_register_4(&self) -> u8 {
        0b1000_0000 | ((self.length_counter.enabled() as u8) << 6) | 0b0011_1111
    }

    pub fn write_register_4(&mut self, value: u8, frame_sequencer: &FrameSequencer) {
        self.length_counter
            .enable((value >> 6) & 0b1 != 0, frame_sequencer);
        self.enabled &= self.length_counter.value() > 0;

        if (value >> 7) & 0b1 != 0 {
            self.trigger(frame_sequencer);
        }
    }
}

impl Voice for NoiseVoice {
    fn enabled(&self) -> bool {
        self.enabled
    }

    fn sample(&self) -> VoiceSample {
        let amp = (self.lfsr & 0b1) as u8;
        VoiceSample::new(amp * self.volume_envelope.current())
    }
}
