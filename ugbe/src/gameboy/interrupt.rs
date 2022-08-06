#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    VBlank = 1 << 0,
    Stat = 1 << 1,
    Timer = 1 << 2,
    Serial = 1 << 3,
    Joypad = 1 << 4,
}

pub trait Line {
    fn highest_priority(&self) -> Option<Kind>;
    fn ack(&mut self, kind: Kind);
    fn request(&mut self, kind: Kind);
    fn flags_not_empty(&self) -> bool;
}
