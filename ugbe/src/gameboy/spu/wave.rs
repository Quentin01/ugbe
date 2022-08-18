use super::frame_sequencer::FrameSequencer;
use super::length_counter::LengthCounter;
use super::sample::Voice as VoiceSample;
use super::Voice;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WaveVoice {
    enabled: bool,
    dac_enabled: bool,

    length_counter: LengthCounter<8>,

    volume_shift: u8,

    frequency: u16,
    frequency_timer: usize,

    ram_idx: u8,
    ram_recently_accessed: bool,
    ram_accessed_after_trigger: bool,
    ram: [u8; 16],

    cycles: usize,
    delay: usize,
}

impl WaveVoice {
    pub fn new() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,

            length_counter: LengthCounter::new(),

            volume_shift: 0,

            frequency: 0,
            frequency_timer: 2048,

            ram_idx: 0,
            ram_recently_accessed: false,
            ram_accessed_after_trigger: false,
            ram: [0; 16],

            cycles: 0,
            delay: 0,
        }
    }

    pub fn tick(&mut self, frame_sequencer: &FrameSequencer) {
        self.cycles += 1;

        self.ram_recently_accessed = false;

        self.length_counter.tick(frame_sequencer);
        if self.length_counter.enabled() && self.length_counter.value() == 0 {
            self.enabled = false;
        }

        if !self.enabled {
            return;
        }

        if self.delay > 0 {
            self.delay -= 1;
            return;
        }

        self.frequency_timer -= 1;

        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency as usize) * 2;

            self.ram_idx = (self.ram_idx + 1) % 32;
            self.ram_accessed_after_trigger = true;
        } else if self.frequency_timer == 2 {
            self.ram_recently_accessed = self.ram_accessed_after_trigger;
        }
    }

    fn trigger(&mut self, frame_sequencer: &FrameSequencer) {
        if !self.enabled {}

        if self.dac_enabled {
            self.enabled = true;
        }

        self.length_counter.trigger(frame_sequencer);

        self.frequency_timer = (2048 - self.frequency as usize) * 2;

        self.ram_idx = 0;
        self.ram_accessed_after_trigger = false;

        self.delay = 4;
    }

    pub fn reset(&mut self, frame_sequencer: &FrameSequencer) {
        self.enabled = false;
        self.dac_enabled = false;

        self.length_counter.enable(false, frame_sequencer);

        self.volume_shift = 0;

        self.frequency = 0;
        self.frequency_timer = 2048 * 2;

        self.ram_idx = 0;
    }

    pub fn read_register_0(&self) -> u8 {
        (self.dac_enabled as u8) << 7 | 0b0111_1111
    }

    pub fn write_register_0(&mut self, value: u8) {
        self.dac_enabled = (value >> 7) & 0b1 != 0;
        self.enabled = self.enabled && self.dac_enabled;
    }

    pub fn read_register_1(&self) -> u8 {
        0xFF
    }

    pub fn write_register_1(&mut self, value: u8) {
        self.length_counter.set_value(value)
    }

    pub fn read_register_2(&self) -> u8 {
        0b1000_0000 | ((self.volume_shift & 0b0011) << 5) | 0b0001_1111
    }

    pub fn write_register_2(&mut self, value: u8) {
        self.volume_shift = (value >> 5) & 0b0011;
    }

    pub fn read_register_3(&self) -> u8 {
        0xFF
    }

    pub fn write_register_3(&mut self, value: u8) {
        self.frequency = (self.frequency & 0xFF00) | (value as u16);
    }

    pub fn read_register_4(&self) -> u8 {
        0b1000_0000 | ((self.length_counter.enabled() as u8) << 6) | 0b0011_1111
    }

    pub fn write_register_4(&mut self, value: u8, frame_sequencer: &FrameSequencer) {
        self.length_counter
            .enable((value >> 6) & 0b1 != 0, frame_sequencer);
        self.frequency = (((value & 0b0111) as u16) << 8) | self.frequency & 0xFF;
        self.enabled &= self.length_counter.value() > 0;

        if (value >> 7) & 0b1 != 0 {
            self.trigger(frame_sequencer);
        }
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        if !self.enabled {
            self.ram[address as usize]
        } else {
            // In DMG mode we can't read from the RAM when the voice is enabled if we haven't access it recently
            if self.ram_recently_accessed {
                self.ram[(self.ram_idx / 2) as usize]
            } else {
                0xFF
            }
        }
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        if !self.enabled {
            self.ram[address as usize] = value;
        } else {
            // In DMG mode we can't write to the RAM when the voice is enabled if we haven't access it recently
            if self.ram_recently_accessed {
                self.ram[(self.ram_idx / 2) as usize] = value;
            }
        }
    }
}

impl Voice for WaveVoice {
    fn enabled(&self) -> bool {
        self.enabled
    }

    fn sample(&self) -> VoiceSample {
        let amp = if self.ram_idx % 2 == 0 {
            (self.ram[(self.ram_idx / 2) as usize] >> 4) & 0b1111
        } else {
            self.ram[(self.ram_idx / 2) as usize] & 0b1111
        };

        let volume_shift = match self.volume_shift {
            0b00 => 4,
            0b01 => 0,
            0b10 => 1,
            0b11 => 2,
            _ => unreachable!(),
        };

        VoiceSample::new(amp >> volume_shift)
    }
}
