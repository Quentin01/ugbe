use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct WorkRam<const SIZE: usize>([u8; SIZE]);

impl<const SIZE: usize> WorkRam<SIZE> {
    pub fn new() -> Self {
        Self([0; SIZE])
    }
}

impl<const SIZE: usize> Index<u16> for WorkRam<SIZE> {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<const SIZE: usize> IndexMut<u16> for WorkRam<SIZE> {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}
