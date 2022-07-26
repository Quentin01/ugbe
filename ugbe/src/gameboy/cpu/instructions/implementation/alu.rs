use std::borrow::Cow;
use std::marker::PhantomData;

use crate::gameboy::cpu::instructions::alu;
use crate::gameboy::cpu::{registers, MemoryOperation};

use super::super::super::registers::Registers;
use super::super::alu::{AluAOp, AluBitOp, AluOneOp, AluOpResult, AluTwoOp};
use super::super::operands::{
    Operand, OperandIn, OperandOut, OperandReadExecution, OperandReadExecutionState,
    OperandWriteExecution, OperandWriteExecutionState,
};
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

pub struct AluOne<AluOp, Op>
where
    AluOp: AluOneOp<<Op as Operand>::Value> + 'static,
    Op: Operand + OperandIn + OperandOut + 'static,
{
    phantom: PhantomData<(AluOp, Op)>,
}

impl<AluOp, Op> AluOne<AluOp, Op>
where
    AluOp: AluOneOp<<Op as Operand>::Value> + 'static,
    Op: Operand + OperandIn + OperandOut + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<AluOp, Op> Instruction for AluOne<AluOp, Op>
where
    AluOp: AluOneOp<<Op as Operand>::Value> + 'static,
    Op: Operand + OperandIn + OperandOut + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        format!("{} {}", AluOp::STR, Op::str()).into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(AluExecution::<Op, Op, _, true>::Start(
            |_: Option<Op::Value>, value: Op::Value, registers: &mut Registers| {
                AluOp::execute(value, registers.cf())
            },
        ))
    }
}

pub struct AluTwo<AluOp, Dst, Src>
where
    AluOp: AluTwoOp<<Dst as Operand>::Value, <Src as Operand>::Value> + 'static,
    Src: Operand + OperandIn + 'static,
    Dst: Operand + OperandIn + OperandOut + 'static,
{
    phantom: PhantomData<(AluOp, Src, Dst)>,
}

impl<AluOp, Dst, Src> AluTwo<AluOp, Dst, Src>
where
    AluOp: AluTwoOp<<Dst as Operand>::Value, <Src as Operand>::Value> + 'static,
    Src: Operand + OperandIn + 'static,
    Dst: Operand + OperandIn + OperandOut + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<AluOp, Dst, Src> Instruction for AluTwo<AluOp, Dst, Src>
where
    AluOp: AluTwoOp<<Dst as Operand>::Value, <Src as Operand>::Value> + 'static,
    Src: Operand + OperandIn + 'static,
    Dst: Operand + OperandIn + OperandOut + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        format!("{} {}, {}", AluOp::STR, Dst::str(), Src::str()).into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(AluExecution::<Dst, Src, _, false>::Start(
            |dst: Option<Dst::Value>, src: Src::Value, registers: &mut Registers| {
                AluOp::execute(
                    dst.expect("As DstIsSrc is false, dst should be set"),
                    src,
                    registers.cf(),
                )
            },
        ))
    }
}

pub struct AluBit<AluOp, const BitPos: u8, Op>
where
    AluOp: AluBitOp<BitPos> + 'static,
    Op: Operand<Value = u8> + OperandIn + OperandOut + 'static,
{
    phantom: PhantomData<(AluOp, Op)>,
}

impl<AluOp, const BitPos: u8, Op> AluBit<AluOp, BitPos, Op>
where
    AluOp: AluBitOp<BitPos> + 'static,
    Op: Operand<Value = u8> + OperandIn + OperandOut + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<AluOp, const BitPos: u8, Op> Instruction for AluBit<AluOp, BitPos, Op>
where
    AluOp: AluBitOp<BitPos> + 'static,
    Op: Operand<Value = u8> + OperandIn + OperandOut + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        format!("{} {}, {}", AluOp::STR, BitPos, Op::str()).into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(AluExecution::<Op, Op, _, true>::Start(
            |_: Option<Op::Value>, value: Op::Value, _: &mut Registers| AluOp::execute(value),
        ))
    }
}

enum AluExecution<Dst, Src, AluFn, const DstIsSrc: bool>
where
    Src: Operand + OperandIn + 'static,
    Dst: Operand + OperandIn + OperandOut + 'static,
    AluFn: Fn(Option<Dst::Value>, Src::Value, &mut Registers) -> AluOpResult<Dst::Value> + 'static,
{
    Start(AluFn),
    ReadingFromSrc {
        operand_read_value: Box<dyn OperandReadExecution<Src::Value>>,
        alu_fn: AluFn,
    },
    ReadingFromDst {
        operand_read_value: Box<dyn OperandReadExecution<Dst::Value>>,
        alu_fn: AluFn,
        src: Src::Value,
    },
    Do {
        alu_fn: AluFn,
        dst: Option<Dst::Value>,
        src: Src::Value,
    },
    WritingToDst(Box<dyn OperandWriteExecution>),
    Wait(usize),
    Complete,
}

impl<Dst, Src, AluFn, const DstIsSrc: bool> InstructionExecution
    for AluExecution<Dst, Src, AluFn, DstIsSrc>
where
    Src: Operand + OperandIn + 'static,
    Dst: Operand + OperandIn + OperandOut + 'static,
    AluFn: Fn(Option<Dst::Value>, Src::Value, &mut Registers) -> AluOpResult<Dst::Value> + 'static,
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
            AluExecution::Start(alu_fn) => {
                let _ = std::mem::replace(
                    self,
                    Self::ReadingFromSrc {
                        operand_read_value: Src::read_value(),
                        alu_fn,
                    },
                );
                self.next(registers, data_bus)
            }
            AluExecution::ReadingFromSrc {
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
                    InstructionExecutionState::Yield(memory_operation)
                }
                OperandReadExecutionState::Complete(value) => {
                    if DstIsSrc {
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
            AluExecution::ReadingFromDst {
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
                    InstructionExecutionState::Yield(memory_operation)
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
            AluExecution::Do { alu_fn, dst, src } => {
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
            AluExecution::WritingToDst(mut operand_write_value) => {
                match operand_write_value.next(registers, data_bus) {
                    OperandWriteExecutionState::Yield(memory_operation) => {
                        let _ = std::mem::replace(self, Self::WritingToDst(operand_write_value));
                        InstructionExecutionState::Yield(memory_operation)
                    }
                    OperandWriteExecutionState::Complete => {
                        let _ = std::mem::replace(self, Self::Wait(alu_extra_cycles));
                        self.next(registers, data_bus)
                    }
                }
            }
            AluExecution::Wait(cycles) => {
                if cycles == 0 {
                    let _ = std::mem::replace(self, Self::Complete);
                    self.next(registers, data_bus)
                } else {
                    let _ = std::mem::replace(self, Self::Wait(cycles - 1));
                    InstructionExecutionState::Yield(MemoryOperation::None)
                }
            }
            AluExecution::Complete => InstructionExecutionState::Complete,
        }
    }
}
