#[derive(Clone, Copy)]
pub enum Condition {
    NZ,
    Z,
    NC,
    C,
}

impl Condition {
    pub fn check(&self, cpu: &super::super::Cpu) -> bool {
        match self {
            Self::NZ => !cpu.registers.zf(),
            Self::Z => cpu.registers.zf(),
            Self::NC => !cpu.registers.cf(),
            Self::C => cpu.registers.cf(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Operations {
    Conditional {
        cond: Condition,
        ok: &'static [MachineCycle],
        not_ok: &'static [MachineCycle],
    },
    NotConditional(&'static [MachineCycle]),
}

pub trait Operation {
    fn execute(&self, cpu: &mut super::super::Cpu);
}

impl<T> Operation for T
where
    T: Fn(&mut super::super::Cpu),
{
    fn execute(&self, cpu: &mut super::super::Cpu) {
        (self)(cpu)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MemoryOperation {
    Read(u16),
    Write(u16, u8),
    PrefetchNextOp,
    PrefetchNextCbOp,
}

pub trait MemoryOperationGenerator {
    fn execute(&self, cpu: &mut super::super::Cpu) -> MemoryOperation;
}

impl<T> MemoryOperationGenerator for T
where
    T: Fn(&mut super::super::Cpu) -> MemoryOperation,
{
    fn execute(&self, cpu: &mut super::super::Cpu) -> MemoryOperation {
        (self)(cpu)
    }
}

#[derive(Copy, Clone)]
pub struct MachineCycle {
    operation: &'static dyn Operation,
    memory_operation_generator: Option<&'static dyn MemoryOperationGenerator>,
}

impl MachineCycle {
    pub const fn new(
        operation: &'static dyn Operation,
        memory_operation_generator: Option<&'static dyn MemoryOperationGenerator>,
    ) -> Self {
        Self {
            operation,
            memory_operation_generator,
        }
    }

    pub fn execute(&self, cpu: &mut super::super::Cpu, hardware: &mut super::super::Hardware) {
        self.operation.execute(cpu);

        if let Some(memory_operation_generator) = self.memory_operation_generator {
            match memory_operation_generator.execute(cpu) {
                MemoryOperation::Read(address) => cpu.data_bus = hardware.read_byte(address),
                MemoryOperation::Write(address, value) => hardware.write_byte(address, value),
                MemoryOperation::PrefetchNextOp => cpu.prefetch_next(hardware, false),
                MemoryOperation::PrefetchNextCbOp => cpu.prefetch_next(hardware, true),
            }
        }
    }
}
