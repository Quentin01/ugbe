use super::frame_sequencer::FrameSequencer;
use super::frequency_sweep::{FrequencyDirection, FrequencySweep};
use super::length_counter::LengthCounter;
use super::sample::Voice as VoiceSample;
use super::volume_envelope::{EnvelopeDirection, VolumeEnvelope};
use super::Voice;

const WAV_DUTY_TABLE: [u8; 4] = [0b00000001, 0b00000011, 0b00001111, 0b11111100];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SquareWaveVoice<const FREQUENCY_SWEEP: bool> {
    enabled: bool,
    frame_sequencer: FrameSequencer,

    length_counter: LengthCounter<6>,
    length_counter_enabled: bool,

    volume_envelope: VolumeEnvelope,

    frequency_sweep: FrequencySweep,
    frequency_timer: usize,

    wave_pattern_duty: u8,
    duty_position: u8,
}

impl<const FREQUENCY_SWEEP: bool> SquareWaveVoice<FREQUENCY_SWEEP> {
    pub fn new() -> Self {
        Self {
            enabled: false,
            frame_sequencer: FrameSequencer::new(),

            length_counter: LengthCounter::new(),
            length_counter_enabled: false,

            volume_envelope: VolumeEnvelope::new(),

            frequency_sweep: FrequencySweep::new(),
            frequency_timer: 2048 * 4,

            wave_pattern_duty: 0,
            duty_position: 0,
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

        if FREQUENCY_SWEEP {
            self.frequency_sweep.tick(&self.frame_sequencer);
        }

        self.frequency_timer -= 1;
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency_sweep.current() as usize) * 4;
            self.duty_position = (self.duty_position + 1) % 8;
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;

        self.frame_sequencer.trigger();

        if self.length_counter_enabled {
            self.length_counter.trigger();
        }

        self.volume_envelope.trigger();

        if FREQUENCY_SWEEP {
            self.frequency_sweep.trigger();
        }

        self.frequency_timer = (2048 - self.frequency_sweep.current() as usize) * 4;
    }

    pub fn read_register_0(&self) -> u8 {
        if FREQUENCY_SWEEP {
            return 0xFF;
        }

        0b10000000
            | ((self.frequency_sweep.period() & 0b111) << 4)
            | (((self.frequency_sweep.direction() == FrequencyDirection::Decrease) as u8) << 3)
            | (self.frequency_sweep.shift() & 0b111)
    }

    pub fn write_register_0(&mut self, value: u8) {
        if !FREQUENCY_SWEEP {
            return;
        }

        self.frequency_sweep.set_period((value >> 4) & 0b111);
        self.frequency_sweep
            .set_direction(match (value >> 3) & 0b1 == 1 {
                true => FrequencyDirection::Decrease,
                false => FrequencyDirection::Increase,
            });
        self.frequency_sweep.set_shift(value & 0b111);
    }

    pub fn read_register_1(&self) -> u8 {
        ((self.wave_pattern_duty & 0b11) << 6) | 0b111111
    }

    pub fn write_register_1(&mut self, value: u8) {
        self.wave_pattern_duty = (value >> 6) & 0b11;
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
        (self.frequency_sweep.current() & 0xFF) as u8
    }

    pub fn write_register_3(&mut self, value: u8) {
        self.frequency_sweep
            .set_current((self.frequency_sweep.current() & 0xFF00) | (value as u16));
    }

    pub fn read_register_4(&self) -> u8 {
        0b10000000
            | ((self.length_counter_enabled as u8) << 6)
            | 0b00111000
            | (((self.frequency_sweep.current() >> 8) as u8) & 0b111)
    }

    pub fn write_register_4(&mut self, value: u8) {
        self.length_counter_enabled = (value >> 6) & 0b1 == 1;
        self.frequency_sweep
            .set_current((((value & 0b111) as u16) << 8) | self.frequency_sweep.current() & 0xFF);

        if (value >> 7) & 0b1 == 1 {
            self.trigger();
        }
    }
}

impl<const FREQUENCY_SWEEP: bool> Voice for SquareWaveVoice<FREQUENCY_SWEEP> {
    fn enabled(&self) -> bool {
        self.enabled
    }

    fn sample(&self) -> VoiceSample {
        let duty = WAV_DUTY_TABLE[self.wave_pattern_duty as usize];
        let duty = (duty >> (7 - self.duty_position)) & 0b1 == 1;

        let amp = duty as u8;
        VoiceSample::new(amp * self.volume_envelope.current())
    }
}
