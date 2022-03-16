use super::frame_sequencer::FrameSequencer;
use super::length_counter::LengthCounter;
use super::volume_envelope::{EnvelopeDirection, VolumeEnvelope};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NoiseVoice {
    enabled: bool,
    frame_sequencer: FrameSequencer,

    length_counter: LengthCounter<6>,
    length_counter_enabled: bool,

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
            frame_sequencer: FrameSequencer::new(),

            length_counter: LengthCounter::new(),
            length_counter_enabled: false,

            volume_envelope: VolumeEnvelope::new(),

            frequency_div: 0,
            frequency_div_shift: 0,
            frequency_timer: 0,

            counter_width: 0,
            lfsr: 0,
        }
    }

    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.frame_sequencer.tick();

        if self.length_counter_enabled {
            self.length_counter.tick(&self.frame_sequencer);
            if self.length_counter.value() == 0 {
                self.enabled = false;
            }
        }

        self.volume_envelope.tick(&self.frame_sequencer);

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

    fn trigger(&mut self) {
        self.enabled = true;

        self.frame_sequencer.trigger();

        if self.length_counter_enabled {
            self.length_counter.trigger();
        }

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

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn sample(&self, shift_to_avoid_precision: u8) -> i32 {
        if !self.enabled {
            return 0;
        }

        if self.lfsr & 0b1 == 0 {
            (self.volume_envelope.current() as i32) << shift_to_avoid_precision
        } else {
            (-(self.volume_envelope.current() as i32)) << shift_to_avoid_precision
        }
    }

    pub fn read_register_0(&self) -> u8 {
        0xFF
    }

    pub fn write_register_0(&mut self, _: u8) {}

    pub fn read_register_1(&self) -> u8 {
        0xFF
    }

    pub fn write_register_1(&mut self, value: u8) {
        self.length_counter.set_value(value & 0b111111);
    }

    pub fn read_register_2(&self) -> u8 {
        ((self.volume_envelope.initial() & 0b1111) << 4)
            | (((self.volume_envelope.direction() == EnvelopeDirection::Increase) as u8) << 3)
            | (self.volume_envelope.period() & 0b111)
    }

    pub fn write_register_2(&mut self, value: u8) {
        self.volume_envelope.set_initial((value >> 4) & 0b1111);
        self.volume_envelope
            .set_direction(match (value >> 3) & 0b1 == 1 {
                true => EnvelopeDirection::Increase,
                false => EnvelopeDirection::Decrease,
            });
        self.volume_envelope.set_period(value & 0b111);
    }

    pub fn read_register_3(&self) -> u8 {
        (self.frequency_div_shift & 0b1111) << 4
            | (self.counter_width & 0b1) << 3
            | (self.frequency_div & 0b111)
    }

    pub fn write_register_3(&mut self, value: u8) {
        self.frequency_div_shift = (value >> 4) & 0b1111;
        self.counter_width = (value >> 3) & 0b1;
        self.frequency_div = value & 0b111;
    }

    pub fn read_register_4(&self) -> u8 {
        0b10000000 | ((self.length_counter_enabled as u8) << 6) | 0b00111111
    }

    pub fn write_register_4(&mut self, value: u8) {
        self.length_counter_enabled = (value >> 6) & 0b1 == 1;

        if (value >> 7) & 0b1 == 1 {
            self.trigger();
        }
    }
}
