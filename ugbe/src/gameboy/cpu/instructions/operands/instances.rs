use paste::paste;

use std::borrow::Cow;
use std::marker::PhantomData;

use super::super::super::registers::Registers;
use super::{
    DerefDecOperand, DerefIncOperand, DerefOperand, DerefOperandToU16, Operand, OperandImmediate,
    OperandIn, OperandOut, OperandReadExecution, OperandRegister, OperandWriteExecution,
    ReadImmediate, ReadR16PlusOff8, ReadRegister, WriteRegister,
};

macro_rules! define_immediate {
    ($size:literal) => {
        paste! {
            #[allow(dead_code)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct [<Imm $size>] {}

            impl Operand for [<Imm $size>] {
                type Value = [<u $size>];

                fn str() -> Cow<'static, str> {
                    stringify!([<u $size>]).into()
                }
            }

            impl OperandImmediate for [<Imm $size>] {}

            impl OperandIn for [<Imm $size>]
            {
                fn read_value() -> Box<dyn OperandReadExecution<Self::Value> + 'static> {
                    Box::new(ReadImmediate::<Self>::Start(PhantomData))
                }
            }

            #[allow(dead_code)]
            pub type [<DerefImm $size>] = DerefOperand<[<Imm $size>]>;

            #[allow(dead_code)]
            pub type [<DerefImm $size ToU16>] = DerefOperandToU16<[<Imm $size>]>;
        }
    };
}

define_immediate!(8);
define_immediate!(16);

macro_rules! define_offset {
    ($size:literal) => {
        paste! {
            #[allow(dead_code)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct [<Off $size>] {}

            impl Operand for [<Off $size>] {
                type Value = [<i $size>];

                fn str() -> Cow<'static, str> {
                    stringify!([<i $size>]).into()
                }
            }

            impl OperandImmediate for [<Off $size>] {}

            impl OperandIn for [<Off $size>]
            {
                fn read_value() -> Box<dyn OperandReadExecution<Self::Value> + 'static> {
                    Box::new(ReadImmediate::<Self>::Start(PhantomData))
                }
            }
        }
    };
}

define_offset!(8);

macro_rules! define_register_8 {
    ($reg:ident) => {
        paste! {
            #[allow(dead_code)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct [<$reg:upper>] {}

            impl Operand for [<$reg:upper>] {
                type Value = u8;

                fn str() -> Cow<'static, str> {
                    stringify!([<$reg:upper>]).into()
                }
            }

            impl OperandRegister for [<$reg:upper>] {
                fn read_register(registers: &mut Registers) -> <Self as Operand>::Value {
                    registers.[<$reg:lower>]()
                }

                fn write_register(registers: &mut Registers, value: <Self as Operand>::Value) {
                    registers.[<set_ $reg:lower>](value);
                }
            }

            impl OperandIn for [<$reg:upper>]
            {
                fn read_value() -> Box<dyn OperandReadExecution<Self::Value> + 'static> {
                    Box::new(ReadRegister::<Self>::new())
                }
            }

            impl OperandOut for [<$reg:upper>]
            {
                fn write_value(value: Self::Value) -> Box<dyn OperandWriteExecution + 'static> {
                    Box::new(WriteRegister::<Self>::new(value))
                }
            }

            #[allow(dead_code)]
            pub type [<Deref $reg:upper>] = DerefOperand<[<$reg:upper>]>;
        }
    };
}

define_register_8!(A);
define_register_8!(B);
define_register_8!(C);
define_register_8!(D);
define_register_8!(E);
define_register_8!(H);
define_register_8!(L);

macro_rules! define_register_16 {
    ($reg:ident) => {
        paste! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct [<$reg:upper>] {}

            impl Operand for [<$reg:upper>] {
                type Value = u16;

                fn str() -> Cow<'static, str> {
                    stringify!([<$reg:upper>]).into()
                }
            }

            impl OperandRegister for [<$reg:upper>] {
                fn read_register(registers: &mut Registers) -> <Self as Operand>::Value {
                    registers.[<$reg:lower>]()
                }

                fn write_register(registers: &mut Registers, value: <Self as Operand>::Value) {
                    registers.[<set_ $reg:lower>](value);
                }
            }

            impl OperandIn for [<$reg:upper>]
            {
                fn read_value() -> Box<dyn OperandReadExecution<Self::Value> + 'static> {
                    Box::new(ReadRegister::<Self>::new())
                }
            }

            impl OperandOut for [<$reg:upper>]
            {
                fn write_value(value: Self::Value) -> Box<dyn OperandWriteExecution + 'static> {
                    Box::new(WriteRegister::<Self>::new(value))
                }
            }

            #[allow(dead_code)]
            pub type [<Deref $reg:upper>] = DerefOperand<[<$reg:upper>]>;

            #[allow(dead_code)]
            pub type [<DerefInc $reg:upper>] = DerefIncOperand<[<$reg:upper>]>;

            #[allow(dead_code)]
            pub type [<DerefDec $reg:upper>] = DerefDecOperand<[<$reg:upper>]>;
        }
    };
}

define_register_16!(AF);
define_register_16!(BC);
define_register_16!(DE);
define_register_16!(HL);
define_register_16!(SP);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SPPlusOff8 {}

impl Operand for SPPlusOff8 {
    type Value = u16;

    fn str() -> Cow<'static, str> {
        "SP+i8".into()
    }
}

impl OperandIn for SPPlusOff8 {
    fn read_value() -> Box<dyn OperandReadExecution<Self::Value> + 'static> {
        Box::new(ReadR16PlusOff8::<SP, Off8>::Start(PhantomData))
    }
}
