use std::marker::PhantomData;

use super::super::super::registers::Registers;
use super::super::super::MemoryOperation;
use super::{Operand, OperandReadExecution, OperandReadExecutionState, OperandRegister};

pub trait ImmediateFromU8 {
    fn from_u8(value: u8) -> Self;
}

impl ImmediateFromU8 for u8 {
    fn from_u8(value: u8) -> Self {
        value
    }
}

impl ImmediateFromU8 for u16 {
    fn from_u8(value: u8) -> Self {
        value.into()
    }
}

impl ImmediateFromU8 for i8 {
    fn from_u8(value: u8) -> Self {
        value as i8
    }
}

pub trait OperandImmediate: Operand {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReadImmediate<Op>
where
    Op: OperandImmediate,
    <Op as Operand>::Value: std::ops::BitOr<Output = <Op as Operand>::Value>
        + std::ops::Shl<usize, Output = <Op as Operand>::Value>
        + ImmediateFromU8,
{
    Start(PhantomData<Op>),
    Reading(usize, Op::Value),
    Complete(Op::Value),
}

impl<Op> OperandReadExecution<Op::Value> for ReadImmediate<Op>
where
    Op: OperandImmediate + Send + Sync + 'static,
    <Op as Operand>::Value: std::ops::BitOr<Output = <Op as Operand>::Value>
        + std::ops::Shl<usize, Output = <Op as Operand>::Value>
        + ImmediateFromU8,
{
    fn next(
        &mut self,
        registers: &mut Registers,
        data_bus: u8,
    ) -> OperandReadExecutionState<Op::Value> {
        match std::mem::replace(self, Self::Complete(<Op as Operand>::Value::from_u8(0))) {
            Self::Start(_) => {
                let pc = registers.pc();
                registers.set_pc(pc.wrapping_add(1));

                let _ =
                    std::mem::replace(self, Self::Reading(0, <Op as Operand>::Value::from_u8(0)));
                OperandReadExecutionState::Yield(MemoryOperation::Read { address: pc })
            }
            Self::Reading(mut idx, mut value) => {
                value = value | (<Op as Operand>::Value::from_u8(data_bus) << (idx * 8));
                idx += 1;

                if idx >= std::mem::size_of::<Op::Value>() {
                    let _ = std::mem::replace(self, Self::Complete(value));
                    self.next(registers, data_bus)
                } else {
                    let pc = registers.pc();
                    registers.set_pc(pc.wrapping_add(1));

                    let _ = std::mem::replace(self, Self::Reading(idx, value));
                    OperandReadExecutionState::Yield(MemoryOperation::Read { address: pc })
                }
            }
            Self::Complete(value) => OperandReadExecutionState::Complete(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReadR16PlusOff8<Reg, Off>
where
    Reg: Operand<Value = u16> + OperandRegister,
    Off: Operand<Value = i8> + OperandImmediate,
{
    Start(PhantomData<(Reg, Off)>),
    Computing,
    Complete(u16),
}

impl<Reg, Off> OperandReadExecution<u16> for ReadR16PlusOff8<Reg, Off>
where
    Reg: Operand<Value = u16> + OperandRegister,
    Off: Operand<Value = i8> + OperandImmediate,
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> OperandReadExecutionState<u16> {
        match std::mem::replace(self, Self::Complete(0)) {
            Self::Start(_) => {
                let pc = registers.pc();
                registers.set_pc(pc.wrapping_add(1));

                let _ = std::mem::replace(self, Self::Computing);
                OperandReadExecutionState::Yield(MemoryOperation::Read { address: pc })
            }
            Self::Computing => {
                let offset = data_bus as i8 as i16 as u16;
                let reg_value = Reg::read_register(registers);
                let value = reg_value.wrapping_add(offset);

                let hf_mask = (1u16 << 3) | (1u16 << 3).wrapping_sub(1);
                let cf_mask = (1u16 << 7) | (1u16 << 7).wrapping_sub(1);

                registers.set_zf(false);
                registers.set_nf(false);
                registers.set_hf((reg_value & hf_mask) + (offset & hf_mask) > hf_mask);
                registers.set_cf((reg_value & cf_mask) + (offset & cf_mask) > cf_mask);

                let _ = std::mem::replace(self, Self::Complete(value));
                OperandReadExecutionState::Yield(MemoryOperation::None)
            }
            Self::Complete(value) => OperandReadExecutionState::Complete(value),
        }
    }
}
