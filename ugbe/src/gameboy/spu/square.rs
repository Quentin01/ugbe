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
    dac_enabled: bool,

    length_counter: LengthCounter<6>,

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
            dac_enabled: false,

            length_counter: LengthCounter::new(),

            volume_envelope: VolumeEnvelope::new(),

            frequency_sweep: FrequencySweep::new(),
            frequency_timer: 2048 * 4,

            wave_pattern_duty: 0,
            duty_position: 0,
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

        if FREQUENCY_SWEEP {
            self.enabled &= self.frequency_sweep.tick(frame_sequencer);
        }

        self.frequency_timer -= 1;
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency_sweep.current() as usize) * 4;
            self.duty_position = (self.duty_position + 1) % 8;
        }
    }

    fn trigger(&mut self, frame_sequencer: &FrameSequencer) {
        if self.dac_enabled {
            self.enabled = true;
        }

        self.length_counter.trigger(frame_sequencer);

        self.volume_envelope.trigger();

        if FREQUENCY_SWEEP {
            self.enabled &= self.frequency_sweep.trigger();
        }

        self.frequency_timer = (2048 - self.frequency_sweep.current() as usize) * 4;
    }

    pub fn reset(&mut self, frame_sequencer: &FrameSequencer) {
        self.enabled = false;
        self.dac_enabled = false;

        self.length_counter.enable(false, frame_sequencer);

        self.volume_envelope.reset();

        self.frequency_sweep.reset();
        self.frequency_timer = 2048 * 4;

        self.wave_pattern_duty = 0;
        self.duty_position = 0;
    }

    pub fn read_register_0(&self) -> u8 {
        if !FREQUENCY_SWEEP {
            return 0xFF;
        }

        0b10000000
            | ((self.frequency_sweep.period() & 0b0111) << 4)
            | (((self.frequency_sweep.direction() == FrequencyDirection::Decrease) as u8) << 3)
            | (self.frequency_sweep.shift() & 0b0111)
    }

    pub fn write_register_0(&mut self, value: u8) {
        if !FREQUENCY_SWEEP {
            return;
        }

        self.frequency_sweep.set_period((value >> 4) & 0b0111);
        let previous_direction = self.frequency_sweep.direction();
        self.frequency_sweep
            .set_direction(match (value >> 3) & 0b1 == 1 {
                true => FrequencyDirection::Decrease,
                false => FrequencyDirection::Increase,
            });
        self.frequency_sweep.set_shift(value & 0b0111);

        if previous_direction == FrequencyDirection::Decrease
            && self.frequency_sweep.direction() == FrequencyDirection::Increase
            && self.frequency_sweep.did_decrease()
        {
            self.enabled = false;
        }
        self.frequency_sweep.reset_did_decrease();
    }

    pub fn read_register_1(&self) -> u8 {
        ((self.wave_pattern_duty & 0b0011) << 6) | 0b0011_1111
    }

    pub fn write_register_1(&mut self, value: u8) {
        self.wave_pattern_duty = (value >> 6) & 0b0011;
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
            .set_direction(match (value >> 3) & 0b1 != 0 {
                true => EnvelopeDirection::Increase,
                false => EnvelopeDirection::Decrease,
            });
        self.volume_envelope.set_period(value & 0b0111);
    }

    pub fn read_register_3(&self) -> u8 {
        0xFF
    }

    pub fn write_register_3(&mut self, value: u8) {
        self.frequency_sweep
            .set_current((self.frequency_sweep.current() & 0xFF00) | (value as u16));
    }

    pub fn read_register_4(&self) -> u8 {
        0b10000000 | ((self.length_counter.enabled() as u8) << 6) | 0b0011_1111
    }

    pub fn write_register_4(&mut self, value: u8, frame_sequencer: &FrameSequencer) {
        self.length_counter
            .enable((value >> 6) & 0b1 == 1, frame_sequencer);
        self.frequency_sweep
            .set_current((((value & 0b0111) as u16) << 8) | self.frequency_sweep.current() & 0xFF);
        self.enabled &= self.length_counter.value() > 0;

        if (value >> 7) & 0b1 != 0 {
            self.trigger(frame_sequencer);
        }
    }
}

impl<const FREQUENCY_SWEEP: bool> Voice for SquareWaveVoice<FREQUENCY_SWEEP> {
    fn enabled(&self) -> bool {
        self.enabled
    }

    fn sample(&self) -> VoiceSample {
        let duty = WAV_DUTY_TABLE[self.wave_pattern_duty as usize];
        let duty = (duty >> (7 - self.duty_position)) & 0b1 != 0;

        let amp = duty as u8;
        VoiceSample::new(amp * self.volume_envelope.current())
    }
}
