use std::marker::PhantomData;

use super::super::super::registers::Registers;
use super::super::super::MemoryOperation;
use super::{Operand, OperandReadExecution, OperandReadExecutionState};

pub trait OperandImmediate: Operand {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReadImmediate<Op>
where
    Op: OperandImmediate,
    <Op as Operand>::Value: std::ops::BitOr<Output = <Op as Operand>::Value>
        + std::ops::Shl<usize, Output = <Op as Operand>::Value>
        + From<u8>,
{
    Start(PhantomData<Op>),
    Reading(usize, Op::Value),
    Complete(Op::Value),
}

impl<Op> OperandReadExecution<Op::Value> for ReadImmediate<Op>
where
    Op: OperandImmediate,
    <Op as Operand>::Value: std::ops::BitOr<Output = <Op as Operand>::Value>
        + std::ops::Shl<usize, Output = <Op as Operand>::Value>
        + From<u8>,
{
    fn next(
        &mut self,
        registers: &mut Registers,
        data_bus: u8,
    ) -> OperandReadExecutionState<Op::Value> {
        match std::mem::replace(self, Self::Complete(0.into())) {
            Self::Start(_) => {
                let pc = registers.pc();
                registers.set_pc(pc.wrapping_add(1));

                let _ = std::mem::replace(self, Self::Reading(0, 0.into()));
                OperandReadExecutionState::Yield(MemoryOperation::Read { address: pc })
            }
            Self::Reading(mut idx, mut value) => {
                value = value | (<Op as Operand>::Value::from(data_bus) << (idx * 8));
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
