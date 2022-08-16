use std::borrow::Cow;
use std::marker::PhantomData;

use crate::gameboy::cpu::MemoryOperation;

use super::super::super::registers::Registers;
use super::super::alu::{ALUOneOp, ALUOpResult, ALUTwoOp, AluBitOp};
use super::super::operands::{
    Operand, OperandIn, OperandOut, OperandReadExecution, OperandReadExecutionState,
    OperandWriteExecution, OperandWriteExecutionState,
};
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ALUOne<ALUOp, Op>
where
    ALUOp: ALUOneOp<<Op as Operand>::Value> + Send + Sync + 'static,
    Op: Operand + OperandIn + OperandOut + Send + Sync + 'static,
{
    phantom: PhantomData<(ALUOp, Op)>,
}

impl<ALUOp, Op> ALUOne<ALUOp, Op>
where
    ALUOp: ALUOneOp<<Op as Operand>::Value> + Send + Sync + 'static,
    Op: Operand + OperandIn + OperandOut + Send + Sync + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<ALUOp, Op> Instruction for ALUOne<ALUOp, Op>
where
    ALUOp: ALUOneOp<<Op as Operand>::Value> + Send + Sync + 'static,
    Op: Operand + OperandIn + OperandOut + Send + Sync + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        format!("{} {}", ALUOp::STR, Op::str()).into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(ALUExecution::<Op, Op, _, true>::Start(
            |_: Option<Op::Value>, value: Op::Value, registers: &mut Registers| {
                ALUOp::execute(value, registers.nf(), registers.hf(), registers.cf())
            },
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ALUTwo<ALUOp, Dst, Src>
where
    ALUOp: ALUTwoOp<<Dst as Operand>::Value, <Src as Operand>::Value> + Send + Sync + 'static,
    Src: Operand + OperandIn + Send + Sync + 'static,
    Dst: Operand + OperandIn + OperandOut + Send + Sync + 'static,
{
    phantom: PhantomData<(ALUOp, Src, Dst)>,
}

impl<ALUOp, Dst, Src> ALUTwo<ALUOp, Dst, Src>
where
    ALUOp: ALUTwoOp<<Dst as Operand>::Value, <Src as Operand>::Value> + Send + Sync + 'static,
    Src: Operand + OperandIn + Send + Sync + 'static,
    Dst: Operand + OperandIn + OperandOut + Send + Sync + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<ALUOp, Dst, Src> Instruction for ALUTwo<ALUOp, Dst, Src>
where
    ALUOp: ALUTwoOp<<Dst as Operand>::Value, <Src as Operand>::Value> + Send + Sync + 'static,
    Src: Operand + OperandIn + Send + Sync + 'static,
    Dst: Operand + OperandIn + OperandOut + Send + Sync + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        format!("{} {}, {}", ALUOp::STR, Dst::str(), Src::str()).into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(ALUExecution::<Dst, Src, _, false>::Start(
            |dst: Option<Dst::Value>, src: Src::Value, registers: &mut Registers| {
                ALUOp::execute(
                    dst.expect("As DST_IS_SRC is false, dst should be set"),
                    src,
                    registers.cf(),
                )
            },
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ALUBit<ALUOp, const BIT_POS: u8, Op>
where
    ALUOp: AluBitOp<BIT_POS> + Send + Sync + 'static,
    Op: Operand<Value = u8> + OperandIn + OperandOut + Send + Sync + 'static,
{
    phantom: PhantomData<(ALUOp, Op)>,
}

impl<ALUOp, const BIT_POS: u8, Op> ALUBit<ALUOp, BIT_POS, Op>
where
    ALUOp: AluBitOp<BIT_POS> + Send + Sync + 'static,
    Op: Operand<Value = u8> + OperandIn + OperandOut + Send + Sync + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<ALUOp, const BIT_POS: u8, Op> Instruction for ALUBit<ALUOp, BIT_POS, Op>
where
    ALUOp: AluBitOp<BIT_POS> + Send + Sync + 'static,
    Op: Operand<Value = u8> + OperandIn + OperandOut + Send + Sync + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        format!("{} {}, {}", ALUOp::STR, BIT_POS, Op::str()).into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(ALUExecution::<Op, Op, _, true>::Start(
            |_: Option<Op::Value>, value: Op::Value, _: &mut Registers| ALUOp::execute(value),
        ))
    }
}

enum ALUExecution<Dst, Src, ALUFn, const DST_IS_SRC: bool>
where
    Src: Operand + OperandIn + Send + Sync + 'static,
    Dst: Operand + OperandIn + OperandOut + Send + Sync + 'static,
    ALUFn: Fn(Option<Dst::Value>, Src::Value, &mut Registers) -> ALUOpResult<Dst::Value>
        + Send
        + 'static,
{
    Start(ALUFn),
    ReadingFromSrc {
        operand_read_value: Box<dyn OperandReadExecution<Src::Value> + 'static>,
        alu_fn: ALUFn,
    },
    ReadingFromDst {
        operand_read_value: Box<dyn OperandReadExecution<Dst::Value> + 'static>,
        alu_fn: ALUFn,
        src: Src::Value,
    },
    Do {
        alu_fn: ALUFn,
        dst: Option<Dst::Value>,
        src: Src::Value,
    },
    WritingToDst(Box<dyn OperandWriteExecution + 'static>),
    Wait(usize),
    Complete,
}

impl<Dst, Src, ALUFn, const DST_IS_SRC: bool> InstructionExecution
    for ALUExecution<Dst, Src, ALUFn, DST_IS_SRC>
where
    Src: Operand + OperandIn + Send + Sync + 'static,
    Dst: Operand + OperandIn + OperandOut + Send + Sync + 'static,
    ALUFn: Fn(Option<Dst::Value>, Src::Value, &mut Registers) -> ALUOpResult<Dst::Value>
        + Send
        + Sync
        + 'static,
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> InstructionExecutionState {
        // ALU operations on 8bits take no more clock cycle (e.g XOR A, A taking 1m cycles)
        // ALU operations on 16bits take one more clock cycle (e.g INC SP taking 2m cycles)
        // ALU operations between 16bits and 8bits take two more clock cycles (e.g ADD SP, i8 taking 4m cycles)
        let alu_extra_cycles =
            if std::mem::size_of::<Dst::Value>() == std::mem::size_of::<Src::Value>() {
                std::mem::size_of::<Dst::Value>() - 1
            } else {
                std::mem::size_of::<Dst::Value>()
            };

        match std::mem::replace(self, Self::Complete) {
            ALUExecution::Start(alu_fn) => {
                let _ = std::mem::replace(
                    self,
                    Self::ReadingFromSrc {
                        operand_read_value: Src::read_value(),
                        alu_fn,
                    },
                );
                self.next(registers, data_bus)
            }
            ALUExecution::ReadingFromSrc {
                mut operand_read_value,
                alu_fn,
            } => match operand_read_value.next(registers, data_bus) {
                OperandReadExecutionState::Yield(memory_operation) => {
                    let _ = std::mem::replace(
                        self,
                        Self::ReadingFromSrc {
                            operand_read_value,
                            alu_fn,
                        },
                    );
                    InstructionExecutionState::YieldMemoryOperation(memory_operation)
                }
                OperandReadExecutionState::Complete(value) => {
                    if DST_IS_SRC {
                        let _ = std::mem::replace(
                            self,
                            Self::Do {
                                alu_fn,
                                dst: None,
                                src: value,
                            },
                        );
                    } else {
                        let _ = std::mem::replace(
                            self,
                            Self::ReadingFromDst {
                                operand_read_value: Dst::read_value(),
                                alu_fn,
                                src: value,
                            },
                        );
                    }

                    self.next(registers, data_bus)
                }
            },
            ALUExecution::ReadingFromDst {
                mut operand_read_value,
                alu_fn,
                src,
            } => match operand_read_value.next(registers, data_bus) {
                OperandReadExecutionState::Yield(memory_operation) => {
                    let _ = std::mem::replace(
                        self,
                        Self::ReadingFromDst {
                            operand_read_value,
                            alu_fn,
                            src,
                        },
                    );
                    InstructionExecutionState::YieldMemoryOperation(memory_operation)
                }
                OperandReadExecutionState::Complete(value) => {
                    let _ = std::mem::replace(
                        self,
                        Self::Do {
                            alu_fn,
                            dst: Some(value),
                            src,
                        },
                    );

                    self.next(registers, data_bus)
                }
            },
            ALUExecution::Do { alu_fn, dst, src } => {
                let result = alu_fn(dst, src, registers);

                if let Some(zf) = result.zf {
                    registers.set_zf(zf);
                }

                if let Some(hf) = result.hf {
                    registers.set_hf(hf);
                }

                if let Some(nf) = result.nf {
                    registers.set_nf(nf);
                }

                if let Some(cf) = result.cf {
                    registers.set_cf(cf);
                }

                if let Some(value) = result.value {
                    let _ = std::mem::replace(self, Self::WritingToDst(Dst::write_value(value)));
                    self.next(registers, data_bus)
                } else {
                    let _ = std::mem::replace(self, Self::Wait(alu_extra_cycles));
                    self.next(registers, data_bus)
                }
            }
            ALUExecution::WritingToDst(mut operand_write_value) => {
                match operand_write_value.next(registers, data_bus) {
                    OperandWriteExecutionState::Yield(memory_operation) => {
                        let _ = std::mem::replace(self, Self::WritingToDst(operand_write_value));
                        InstructionExecutionState::YieldMemoryOperation(memory_operation)
                    }
                    OperandWriteExecutionState::Complete => {
                        let _ = std::mem::replace(self, Self::Wait(alu_extra_cycles));
                        self.next(registers, data_bus)
                    }
                }
            }
            ALUExecution::Wait(cycles) => {
                if cycles == 0 {
                    let _ = std::mem::replace(self, Self::Complete);
                    self.next(registers, data_bus)
                } else {
                    let _ = std::mem::replace(self, Self::Wait(cycles - 1));
                    InstructionExecutionState::YieldMemoryOperation(MemoryOperation::None)
                }
            }
            ALUExecution::Complete => InstructionExecutionState::Complete,
        }
    }
}
