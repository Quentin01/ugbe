use super::components::{InterruptKind, InterruptLine};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ButtonState {
    Down,
    Up,
}

#[derive(Debug)]
pub struct Joypad {
    buttons_state: [ButtonState; 8],

    data: u8,
    last_data: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            buttons_state: [ButtonState::Up; 8],
            data: 0xFF,
            last_data: 0xFF,
        }
    }

    fn use_direction_buttons(&self) -> bool {
        (self.data >> 4) & 0b1 == 0
    }

    fn use_action_buttons(&self) -> bool {
        (self.data >> 5) & 0b1 == 0
    }

    fn button_to_idx(button: Button) -> usize {
        match button {
            Button::A => 0,
            Button::B => 1,
            Button::Select => 2,
            Button::Start => 3,
            Button::Up => 4,
            Button::Right => 5,
            Button::Down => 6,
            Button::Left => 7,
        }
    }

    fn button_state(&self, button: Button) -> ButtonState {
        let idx = Joypad::button_to_idx(button);
        self.buttons_state[idx]
    }

    fn update_inputs(&mut self) {
        self.data = (self.data & 0xF0) | 0xF;

        if self.use_direction_buttons() {
            let nibble = {
                let mut nibble = 0;

                if let ButtonState::Up = self.button_state(Button::Right) {
                    nibble |= 0b1;
                }

                if let ButtonState::Up = self.button_state(Button::Left) {
                    nibble |= 0b10;
                }

                if let ButtonState::Up = self.button_state(Button::Up) {
                    nibble |= 0b100;
                }

                if let ButtonState::Up = self.button_state(Button::Down) {
                    nibble |= 0b1000;
                }

                nibble
            };

            self.data = (self.data & 0xF0) | nibble;
        }

        if self.use_action_buttons() {
            let nibble = {
                let mut nibble = 0;

                if let ButtonState::Up = self.button_state(Button::A) {
                    nibble |= 0b1;
                }

                if let ButtonState::Up = self.button_state(Button::B) {
                    nibble |= 0b10;
                }

                if let ButtonState::Up = self.button_state(Button::Select) {
                    nibble |= 0b100;
                }

                if let ButtonState::Up = self.button_state(Button::Start) {
                    nibble |= 0b1000;
                }

                nibble
            };

            self.data = (self.data & 0xF0) | nibble;
        }
    }

    pub(super) fn tick(&mut self, interrupt_line: &mut dyn InterruptLine) {
        if (!self.data & self.last_data & 0xF) != 0 {
            // If an input was 1 at the previous tick and switch to 0 for this tick
            interrupt_line.request(InterruptKind::Joypad);
        }

        self.last_data = self.data
    }

    pub fn keydown(&mut self, button: Button) {
        let idx = Joypad::button_to_idx(button);
        self.buttons_state[idx] = ButtonState::Down;

        self.update_inputs();
    }

    pub fn keyup(&mut self, button: Button) {
        let idx = Joypad::button_to_idx(button);
        self.buttons_state[idx] = ButtonState::Up;

        self.update_inputs();
    }

    pub(super) fn read_p1(&self) -> u8 {
        self.data
    }

    pub(super) fn write_p1(&mut self, value: u8) {
        self.data = (self.data & 0b11001111) | (value & 0b00110000);

        self.update_inputs();
    }
}
