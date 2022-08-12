use super::frame_sequencer::FrameSequencer;
use super::length_counter::LengthCounter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WaveVoice {
    enabled: bool,
    frame_sequencer: FrameSequencer,

    length_counter: LengthCounter<8>,
    stop_after_length_counter: bool,

    volume_shift: u8,

    frequency: u16,
    cycle_count: usize,

    ram_idx: u8,
    ram: [u8; 16],
}

impl WaveVoice {
    pub fn new() -> Self {
        Self {
            enabled: false,
            frame_sequencer: FrameSequencer::new(),

            length_counter: LengthCounter::new(),
            stop_after_length_counter: false,

            volume_shift: 0,

            frequency: 0,
            cycle_count: 0,

            ram_idx: 0,
            ram: [0; 16],
        }
    }

    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.frame_sequencer.tick();

        self.length_counter.tick(&self.frame_sequencer);
        if self.length_counter.value() == 0 && self.stop_after_length_counter {
            self.enabled = false;
        }

        self.cycle_count += 1;
        let frequency_timer = (2048 - self.frequency as usize) * 4;
        if self.cycle_count > frequency_timer && frequency_timer > 0 {
            self.cycle_count %= frequency_timer;
            self.ram_idx = (self.ram_idx + 1) % 32;
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;

        self.length_counter.trigger();
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn sample(&self, shift_to_avoid_precision: u8) -> i32 {
        if !self.enabled {
            return 0;
        }

        let amp = if self.ram_idx % 2 == 0 {
            (self.ram[(self.ram_idx / 2) as usize] >> 4) & 0b1111
        } else {
            self.ram[(self.ram_idx / 2) as usize] & 0b1111
        };

        let volume_shift = match self.volume_shift {
            0b00 => 4 + shift_to_avoid_precision,
            0b01 => 0,
            0b10 => 1,
            0b11 => 2,
            _ => unreachable!(),
        };

        let sample = ((amp as i32) << shift_to_avoid_precision) >> volume_shift;
        sample - (0xF << (shift_to_avoid_precision - 1))
    }

    pub fn read_register_0(&self) -> u8 {
        (self.enabled as u8) << 7 | 0b1111111
    }

    pub fn write_register_0(&mut self, value: u8) {
        self.enabled = (value >> 7) & 0b1 == 1;
    }

    pub fn read_register_1(&self) -> u8 {
        0xFF
    }

    pub fn write_register_1(&mut self, value: u8) {
        self.length_counter.set_value(value)
    }

    pub fn read_register_2(&self) -> u8 {
        0b10000000 | ((self.volume_shift & 0b11) << 5) | 0b11111
    }

    pub fn write_register_2(&mut self, value: u8) {
        self.volume_shift = (value >> 5) & 0b11;
    }

    pub fn read_register_3(&self) -> u8 {
        (self.frequency & 0xFF) as u8
    }

    pub fn write_register_3(&mut self, value: u8) {
        self.frequency = (self.frequency & 0xFF00) | (value as u16);
    }

    pub fn read_register_4(&self) -> u8 {
        0b10000000
            | ((self.stop_after_length_counter as u8) << 6)
            | 0b00111000
            | (((self.frequency >> 8) as u8) & 0b111)
    }

    pub fn write_register_4(&mut self, value: u8) {
        self.stop_after_length_counter = (value >> 6) & 0b1 == 1;
        self.frequency = (((value & 0b111) as u16) << 8) | self.frequency & 0xFF;

        if (value >> 7) & 0b1 == 1 {
            self.trigger();
        }
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }
}
