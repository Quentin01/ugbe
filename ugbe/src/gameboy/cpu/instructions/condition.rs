use super::super::registers::Registers;

pub trait Condition {
    const STR: &'static str;

    fn check(registers: &Registers) -> bool;
}

pub struct None {}

impl Condition for None {
    const STR: &'static str = "";

    fn check(_: &Registers) -> bool {
        true
    }
}

pub struct NZ {}

impl Condition for NZ {
    const STR: &'static str = "NZ";

    fn check(registers: &Registers) -> bool {
        !registers.zf()
    }
}

pub struct Z {}

impl Condition for Z {
    const STR: &'static str = "Z";

    fn check(registers: &Registers) -> bool {
        registers.zf()
    }
}

pub struct NC {}

impl Condition for NC {
    const STR: &'static str = "NC";

    fn check(registers: &Registers) -> bool {
        !registers.cf()
    }
}

pub struct C {}

impl Condition for C {
    const STR: &'static str = "C";

    fn check(registers: &Registers) -> bool {
        registers.cf()
    }
}
