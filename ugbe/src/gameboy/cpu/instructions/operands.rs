use paste::paste;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Kind {
    Memory,
    Immediate(usize),
    ImmediateMemory(usize),
    Register,
    Direct,
}

pub trait Operand {
    type Value;

    const KIND: Kind;
    const STR: &'static str;

    fn read(_: &mut super::super::Cpu) -> Self::Value {
        match Self::KIND {
            Kind::Register | Kind::Direct => panic!(
                "Not implemented `read` on {}",
                std::any::type_name::<Self>()
            ),
            _ => panic!("Invalid `read` on {}", std::any::type_name::<Self>()),
        }
    }

    fn write(_: &mut super::super::Cpu, _: Self::Value) {
        match Self::KIND {
            Kind::Register => panic!(
                "Not implemented `write` on {}",
                std::any::type_name::<Self>()
            ),
            _ => panic!("Invalid `write` on {}", std::any::type_name::<Self>()),
        }
    }

    fn write_msb(_: &mut super::super::Cpu, _: u8) {
        match Self::KIND {
            Kind::Register if std::mem::size_of::<Self::Value>() == 16 => panic!(
                "Not implemented `write_msb` on {}",
                std::any::type_name::<Self>()
            ),
            _ => panic!("Invalid `write_msb` on {}", std::any::type_name::<Self>()),
        }
    }

    fn write_lsb(_: &mut super::super::Cpu, _: u8) {
        match Self::KIND {
            Kind::Register if std::mem::size_of::<Self::Value>() == 16 => panic!(
                "Not implemented `write_lsb` on {}",
                std::any::type_name::<Self>()
            ),
            _ => panic!("Invalid `write_lsb` on {}", std::any::type_name::<Self>()),
        }
    }

    fn address(_: &mut super::super::Cpu) -> u16 {
        match Self::KIND {
            Kind::Memory => panic!(
                "Not implemented `address` on {}",
                std::any::type_name::<Self>()
            ),
            _ => panic!("Invalid `address` on {}", std::any::type_name::<Self>()),
        }
    }
}

macro_rules! define_operand_register_8 {
    ($name:ident) => {
        paste! {
            pub struct [<$name:upper>] {}

            impl Operand for [<$name:upper>] {
                type Value = u8;

                const KIND: Kind = Kind::Register;
                const STR: &'static str = stringify!([<$name:upper>]);

                fn read(cpu: &mut super::super::Cpu) -> Self::Value {
                    cpu.registers.[<$name:lower>]
                }

                fn write(cpu: &mut super::super::Cpu, value: Self::Value) {
                    cpu.registers.[<$name:lower>] = value;
                }
            }
        }
    };
}

define_operand_register_8!(A);
define_operand_register_8!(B);
define_operand_register_8!(C);
define_operand_register_8!(D);
define_operand_register_8!(E);
define_operand_register_8!(H);
define_operand_register_8!(L);

macro_rules! define_operand_register_16 {
    ($name:ident) => {
        paste! {
            pub struct [<$name:upper>] {}

            impl Operand for [<$name:upper>] {
                type Value = u16;

                const KIND: Kind = Kind::Register;
                const STR: &'static str = stringify!([<$name:upper>]);

                fn read(cpu: &mut super::super::Cpu) -> Self::Value {
                    cpu.registers.[<$name:lower>]()
                }

                fn write(cpu: &mut super::super::Cpu, value: Self::Value) {
                    cpu.registers.[<set_ $name:lower>](value);
                }

                fn write_msb(cpu: &mut super::super::Cpu, value: u8) {
                    cpu.registers.[<set_msb_ $name:lower>](value);
                }

                fn write_lsb(cpu: &mut super::super::Cpu, value: u8) {
                    cpu.registers.[<set_lsb_ $name:lower>](value);
                }
            }
        }
    };
}

define_operand_register_16!(AF);
define_operand_register_16!(BC);
define_operand_register_16!(DE);
define_operand_register_16!(HL);

pub struct SP {}

impl Operand for SP {
    type Value = u16;

    const KIND: Kind = Kind::Register;
    const STR: &'static str = "SP";

    fn read(cpu: &mut super::super::Cpu) -> Self::Value {
        cpu.registers.sp
    }

    fn write(cpu: &mut super::super::Cpu, value: Self::Value) {
        cpu.registers.sp = value;
    }

    fn write_msb(cpu: &mut super::super::Cpu, value: u8) {
        cpu.registers.sp = (cpu.registers.sp & 0x00FF) | ((value as u16) << 8);
    }

    fn write_lsb(cpu: &mut super::super::Cpu, value: u8) {
        cpu.registers.sp = (cpu.registers.sp & 0xFF00) | (value as u16);
    }
}

macro_rules! define_operand_deref_register_16 {
    ($name:ident) => {
        paste! {
            pub struct [<Deref $name:upper>] {}

            impl Operand for [<Deref $name:upper>] {
                type Value = u8;

                const KIND: Kind = Kind::Memory;
                const STR: &'static str = concat!("(", stringify!([<$name:upper>]), ")");

                fn address(cpu: &mut super::super::Cpu) -> u16 {
                    cpu.registers.[<$name:lower>]()
                }
            }
        }
    };
}

define_operand_deref_register_16!(BC);
define_operand_deref_register_16!(DE);
define_operand_deref_register_16!(HL);

pub struct DerefIncHL {}

impl Operand for DerefIncHL {
    type Value = u8;

    const KIND: Kind = Kind::Memory;
    const STR: &'static str = "(HL+)";

    fn address(cpu: &mut super::super::Cpu) -> u16 {
        let hl = cpu.registers.hl();
        cpu.registers.set_hl(hl.wrapping_add(1));
        hl
    }
}

pub struct DerefDecHL {}

impl Operand for DerefDecHL {
    type Value = u8;

    const KIND: Kind = Kind::Memory;
    const STR: &'static str = "(HL+)";

    fn address(cpu: &mut super::super::Cpu) -> u16 {
        let hl = cpu.registers.hl();
        cpu.registers.set_hl(hl.wrapping_sub(1));
        hl
    }
}

pub struct DerefHighC {}

impl Operand for DerefHighC {
    type Value = u8;

    const KIND: Kind = Kind::Memory;
    const STR: &'static str = "(FF00+C)";

    fn address(cpu: &mut super::super::Cpu) -> u16 {
        (cpu.registers.c as u16) | 0xFF00
    }
}

pub struct Imm8 {}

impl Operand for Imm8 {
    type Value = u8;

    const KIND: Kind = Kind::Immediate(8);
    const STR: &'static str = "{u8}";
}

pub struct Off8 {}

impl Operand for Off8 {
    type Value = i8;

    const KIND: Kind = Kind::Immediate(8);
    const STR: &'static str = "{i8}";
}

pub struct DerefHighImm8 {}

impl Operand for DerefHighImm8 {
    type Value = u8;

    const KIND: Kind = Kind::ImmediateMemory(8);
    const STR: &'static str = "(FF00+{u8})";

    fn address(cpu: &mut super::super::Cpu) -> u16 {
        (cpu.data_bus as u16) | 0xFF00
    }
}

pub struct Imm16 {}

impl Operand for Imm16 {
    type Value = u16;

    const KIND: Kind = Kind::Immediate(16);
    const STR: &'static str = "{u16}";
}

pub struct DerefImm16ToU8 {}

impl Operand for DerefImm16ToU8 {
    type Value = u8;

    const KIND: Kind = Kind::ImmediateMemory(16);
    const STR: &'static str = "({u16})";
}

pub struct DerefImm16ToU16 {}

impl Operand for DerefImm16ToU16 {
    type Value = u16;

    const KIND: Kind = Kind::ImmediateMemory(16);
    const STR: &'static str = "({u16})";
}

pub struct Direct<const VALUE: u8> {}

macro_rules! impl_direct {
    ($value:literal) => {
        impl Operand for Direct<$value> {
            type Value = u8;

            const KIND: Kind = Kind::Direct;
            const STR: &'static str = stringify!($value);

            fn read(_: &mut super::super::Cpu) -> Self::Value {
                $value
            }
        }
    };
}

impl_direct!(0);
impl_direct!(1);
impl_direct!(2);
impl_direct!(3);
impl_direct!(4);
impl_direct!(5);
impl_direct!(6);
impl_direct!(7);
