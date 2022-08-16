use std::borrow::Cow;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use super::super::super::registers::Registers;
use super::super::super::MemoryOperation;
use super::{
    Operand, OperandIn, OperandOut, OperandReadExecution, OperandReadExecutionState,
    OperandRegister, OperandWriteExecution, OperandWriteExecutionState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DecrementAddress {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IncrementAddress {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoneAddress {}

pub trait ValueToAddress<Value> {
    fn str() -> Cow<'static, str>;
    fn address(registers: &mut Registers, value: Value) -> u16;
}

impl<Op> ValueToAddress<u8> for (Op, u8, NoneAddress)
where
    Op: Operand<Value = u8>,
{
    fn str() -> Cow<'static, str> {
        format!("FF00+{}", Op::str()).into()
    }

    fn address(_: &mut Registers, value: u8) -> u16 {
        0xFF00 | (value as u16)
    }
}

impl<Op> ValueToAddress<u16> for (Op, u16, NoneAddress)
where
    Op: Operand<Value = u16>,
{
    fn str() -> Cow<'static, str> {
        Op::str()
    }

    fn address(_: &mut Registers, value: u16) -> u16 {
        value
    }
}

impl<Op> ValueToAddress<u16> for (Op, u16, IncrementAddress)
where
    Op: Operand<Value = u16> + OperandRegister,
{
    fn str() -> Cow<'static, str> {
        format!("{}+", Op::str()).into()
    }

    fn address(registers: &mut Registers, value: u16) -> u16 {
        <Op as OperandRegister>::write_register(registers, value.wrapping_add(1));
        value
    }
}

impl<Op> ValueToAddress<u16> for (Op, u16, DecrementAddress)
where
    Op: Operand<Value = u16> + OperandRegister,
{
    fn str() -> Cow<'static, str> {
        format!("{}-", Op::str()).into()
    }

    fn address(registers: &mut Registers, value: u16) -> u16 {
        <Op as OperandRegister>::write_register(registers, value.wrapping_sub(1));
        value
    }
}

pub trait EndianNumeric {
    type Array: Default
        + Index<usize, Output = u8>
        + IndexMut<usize, Output = u8>
        + Sized
        + Send
        + Sync
        + 'static;

    fn from_le_bytes(bytes: Self::Array) -> Self;
    fn to_le_bytes(value: Self) -> Self::Array;
}

impl EndianNumeric for u8 {
    type Array = [u8; std::mem::size_of::<Self>()];

    fn from_le_bytes(bytes: Self::Array) -> Self {
        bytes[0]
    }

    fn to_le_bytes(value: Self) -> Self::Array {
        [value; 1]
    }
}

impl EndianNumeric for u16 {
    type Array = [u8; std::mem::size_of::<Self>()];

    fn from_le_bytes(bytes: Self::Array) -> Self {
        u16::from_le_bytes(bytes)
    }

    fn to_le_bytes(value: Self) -> Self::Array {
        value.to_le_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DerefOperand<Op, Value = u8, AddressOperation = NoneAddress>
where
    Op: Operand + OperandIn + Send + Sync + 'static,
    Value: Copy + EndianNumeric + Send + Sync + 'static,
    AddressOperation: Send + Sync + 'static,
    (Op, <Op as Operand>::Value, AddressOperation): ValueToAddress<<Op as Operand>::Value>,
{
    phantom: PhantomData<(Op, Value, AddressOperation)>,
}

impl<Op, Value, AddressOperation> Operand for DerefOperand<Op, Value, AddressOperation>
where
    Op: Operand + OperandIn + Send + Sync + 'static,
    Value: Copy + EndianNumeric + Send + Sync + 'static,
    AddressOperation: Send + Sync + 'static,
    (Op, <Op as Operand>::Value, AddressOperation): ValueToAddress<<Op as Operand>::Value>,
{
    type Value = Value;

    fn str() -> Cow<'static, str> {
        format!("({})", <(Op, Op::Value, AddressOperation)>::str()).into()
    }
}

enum ReadDerefOperand<Op, Value = u8, AddressOperation = NoneAddress>
where
    Op: Operand + OperandIn + Send + Sync + 'static,
    Value: Copy + EndianNumeric + Send + Sync + 'static,
    AddressOperation: Send + Sync + 'static,
    (Op, <Op as Operand>::Value, AddressOperation): ValueToAddress<<Op as Operand>::Value>,
{
    Start(PhantomData<(Op, Value, AddressOperation)>),
    WaitingForAddress(Box<dyn OperandReadExecution<Op::Value> + 'static>),
    Dereferencing(u16, Value::Array, usize),
    Complete(Value),
}

impl<Op, Value, AddressOperation>
    OperandReadExecution<<DerefOperand<Op, Value, AddressOperation> as Operand>::Value>
    for ReadDerefOperand<Op, Value, AddressOperation>
where
    Op: Operand + OperandIn + Send + Sync + 'static,
    Value: Copy + EndianNumeric + Send + Sync + 'static,
    AddressOperation: Send + Sync + 'static,
    (Op, <Op as Operand>::Value, AddressOperation): ValueToAddress<<Op as Operand>::Value>,
{
    fn next(
        &mut self,
        registers: &mut Registers,
        data_bus: u8,
    ) -> OperandReadExecutionState<<DerefOperand<Op, Value, AddressOperation> as Operand>::Value>
    {
        match std::mem::replace(self, Self::Start(PhantomData)) {
            Self::Start(_) => {
                let _ = std::mem::replace(self, Self::WaitingForAddress(Op::read_value()));
                self.next(registers, data_bus)
            }
            Self::WaitingForAddress(mut operand_read_value) => {
                match operand_read_value.next(registers, data_bus) {
                    OperandReadExecutionState::Yield(memory_operation) => {
                        let _ =
                            std::mem::replace(self, Self::WaitingForAddress(operand_read_value));
                        OperandReadExecutionState::Yield(memory_operation)
                    }
                    OperandReadExecutionState::Complete(address) => {
                        let address =
                            <(Op, Op::Value, AddressOperation)>::address(registers, address);

                        let _ = std::mem::replace(
                            self,
                            Self::Dereferencing(address, Value::Array::default(), 0),
                        );
                        OperandReadExecutionState::Yield(MemoryOperation::Read { address })
                    }
                }
            }
            Self::Dereferencing(address, mut le_bytes, idx) => {
                le_bytes[idx] = data_bus;

                if idx >= std::mem::size_of::<Value>() - 1 {
                    let _ = std::mem::replace(self, Self::Complete(Value::from_le_bytes(le_bytes)));
                    self.next(registers, data_bus)
                } else {
                    let address = address.wrapping_add(1);

                    let _ =
                        std::mem::replace(self, Self::Dereferencing(address, le_bytes, idx + 1));
                    OperandReadExecutionState::Yield(MemoryOperation::Read { address })
                }
            }
            Self::Complete(value) => OperandReadExecutionState::Complete(value),
        }
    }
}

impl<Op, Value, AddressOperation> OperandIn for DerefOperand<Op, Value, AddressOperation>
where
    Op: Operand + OperandIn + Send + Sync + 'static,
    Value: Copy + EndianNumeric + Send + Sync + 'static,
    AddressOperation: Send + Sync + 'static,
    (Op, <Op as Operand>::Value, AddressOperation): ValueToAddress<<Op as Operand>::Value>,
{
    fn read_value() -> Box<dyn OperandReadExecution<Self::Value> + 'static> {
        Box::new(ReadDerefOperand::<Op, Value, AddressOperation>::Start(
            PhantomData,
        ))
    }
}

enum WriteDerefOperand<Op, Value = u8, AddressOperation = NoneAddress>
where
    Op: Operand + OperandIn + Send + Sync + 'static,
    Value: Copy + EndianNumeric + Send + Sync + 'static,
    AddressOperation: Send + Sync + 'static,
    (Op, <Op as Operand>::Value, AddressOperation): ValueToAddress<<Op as Operand>::Value>,
{
    Start(PhantomData<(Op, Value, AddressOperation)>, Value),
    WaitingForAddress(Box<dyn OperandReadExecution<Op::Value> + 'static>, Value),
    Dereferencing(u16, Value::Array, usize),
    Complete,
}

impl<Op, Value, AddressOperation> OperandWriteExecution
    for WriteDerefOperand<Op, Value, AddressOperation>
where
    Op: Operand + OperandIn + Send + Sync + 'static,
    Value: Copy + EndianNumeric + Send + Sync + 'static,
    AddressOperation: Send + Sync + 'static,
    (Op, <Op as Operand>::Value, AddressOperation): ValueToAddress<<Op as Operand>::Value>,
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> OperandWriteExecutionState {
        match std::mem::replace(self, Self::Complete) {
            Self::Start(_, value) => {
                let _ = std::mem::replace(self, Self::WaitingForAddress(Op::read_value(), value));
                self.next(registers, data_bus)
            }
            Self::WaitingForAddress(mut operand_read_value, value) => {
                match operand_read_value.next(registers, data_bus) {
                    OperandReadExecutionState::Yield(memory_operation) => {
                        let _ = std::mem::replace(
                            self,
                            Self::WaitingForAddress(operand_read_value, value),
                        );
                        OperandWriteExecutionState::Yield(memory_operation)
                    }
                    OperandReadExecutionState::Complete(address) => {
                        let _ = std::mem::replace(
                            self,
                            Self::Dereferencing(
                                <(Op, Op::Value, AddressOperation)>::address(registers, address),
                                Value::to_le_bytes(value),
                                0,
                            ),
                        );
                        self.next(registers, data_bus)
                    }
                }
            }
            Self::Dereferencing(address, le_bytes, idx) => {
                if idx >= std::mem::size_of::<Value>() {
                    let _ = std::mem::replace(self, Self::Complete);
                    self.next(registers, data_bus)
                } else {
                    let value = le_bytes[idx];

                    let _ = std::mem::replace(
                        self,
                        Self::Dereferencing(address.wrapping_add(1), le_bytes, idx + 1),
                    );
                    OperandWriteExecutionState::Yield(MemoryOperation::Write { address, value })
                }
            }
            Self::Complete => OperandWriteExecutionState::Complete,
        }
    }
}

impl<Op, Value, AddressOperation> OperandOut for DerefOperand<Op, Value, AddressOperation>
where
    Op: Operand + OperandIn + Send + Sync + 'static,
    Value: Copy + EndianNumeric + Send + Sync + 'static,
    AddressOperation: Send + Sync + 'static,
    (Op, <Op as Operand>::Value, AddressOperation): ValueToAddress<<Op as Operand>::Value>,
{
    fn write_value(value: Self::Value) -> Box<dyn OperandWriteExecution + 'static> {
        Box::new(WriteDerefOperand::<Op, Value, AddressOperation>::Start(
            PhantomData,
            value,
        ))
    }
}

pub type DerefOperandToU16<Op> = DerefOperand<Op, u16, NoneAddress>;
pub type DerefIncOperand<Op, Value = u8> = DerefOperand<Op, Value, IncrementAddress>;
pub type DerefDecOperand<Op, Value = u8> = DerefOperand<Op, Value, DecrementAddress>;
