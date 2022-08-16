use super::super::registers::Registers;

pub trait Condition {
    const STR: &'static str;

    fn is_none() -> bool {
        false
    }

    fn check(registers: &Registers) -> bool;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct None {}

impl Condition for None {
    const STR: &'static str = "";

    fn is_none() -> bool {
        true
    }

    fn check(_: &Registers) -> bool {
        true
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NZ {}

impl Condition for NZ {
    const STR: &'static str = "NZ";

    fn check(registers: &Registers) -> bool {
        !registers.zf()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Z {}

impl Condition for Z {
    const STR: &'static str = "Z";

    fn check(registers: &Registers) -> bool {
        registers.zf()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NC {}

impl Condition for NC {
    const STR: &'static str = "NC";

    fn check(registers: &Registers) -> bool {
        !registers.cf()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct C {}

impl Condition for C {
    const STR: &'static str = "C";

    fn check(registers: &Registers) -> bool {
        registers.cf()
    }
}
