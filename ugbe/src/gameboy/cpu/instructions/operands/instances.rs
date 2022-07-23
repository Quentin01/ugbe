use paste::paste;

use std::borrow::Cow;
use std::marker::PhantomData;

use super::super::super::registers::Registers;
use super::{
    DerefDecOperand, DerefIncOperand, DerefOperand, DerefOperandToU16, Operand, OperandImmediate,
    OperandIn, OperandOut, OperandReadExecution, OperandRegister, OperandWriteExecution,
    ReadImmediate, ReadRegister, WriteRegister,
};

macro_rules! define_immediate {
    ($size:literal) => {
        paste! {
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
                fn read_value() -> Box<dyn OperandReadExecution<Self::Value>> {
                    Box::new(ReadImmediate::<Self>::Start(PhantomData))
                }
            }

            pub type [<DerefImm $size>] = DerefOperand<[<Imm $size>]>;
            pub type [<DerefImm $size ToU16>] = DerefOperandToU16<[<Imm $size>]>;
        }
    };
}

define_immediate!(8);
define_immediate!(16);

macro_rules! define_offset {
    ($size:literal) => {
        paste! {
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
                fn read_value() -> Box<dyn OperandReadExecution<Self::Value>> {
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
                fn read_value() -> Box<dyn OperandReadExecution<Self::Value>> {
                    Box::new(ReadRegister::<Self>::new())
                }
            }

            impl OperandOut for [<$reg:upper>]
            {
                fn write_value(value: Self::Value) -> Box<dyn OperandWriteExecution> {
                    Box::new(WriteRegister::<Self>::new(value))
                }
            }

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
                fn read_value() -> Box<dyn OperandReadExecution<Self::Value>> {
                    Box::new(ReadRegister::<Self>::new())
                }
            }

            impl OperandOut for [<$reg:upper>]
            {
                fn write_value(value: Self::Value) -> Box<dyn OperandWriteExecution> {
                    Box::new(WriteRegister::<Self>::new(value))
                }
            }

            pub type [<Deref $reg:upper>] = DerefOperand<[<$reg:upper>]>;
            pub type [<DerefInc $reg:upper>] = DerefIncOperand<[<$reg:upper>]>;
            pub type [<DerefDec $reg:upper>] = DerefDecOperand<[<$reg:upper>]>;
        }
    };
}

define_register_16!(AF);
define_register_16!(BC);
define_register_16!(DE);
define_register_16!(HL);
define_register_16!(SP);
