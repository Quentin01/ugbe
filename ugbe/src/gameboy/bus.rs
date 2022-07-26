use super::mmu::Mmu;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryOperation {
    None,
    Read { address: u16 },
    Write { address: u16, value: u8 },
}

pub struct Bus {
    data: u8,
}

impl Bus {
    pub fn new() -> Self {
        Self { data: 0 }
    }

    pub fn data(&self) -> u8 {
        self.data
    }

    pub(super) fn tick(&mut self, memory_operation: MemoryOperation, mmu: &mut impl Mmu) {
        match memory_operation {
            MemoryOperation::None => {}
            MemoryOperation::Read { address } => self.data = mmu.read_byte(address),
            MemoryOperation::Write { address, value } => mmu.write_byte(address, value),
        }
    }
}
