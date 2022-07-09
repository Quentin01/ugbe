use std::ops::{Index, IndexMut};

#[derive(Debug, Copy, Clone)]
pub struct Fifo<T: Copy + Clone, const SIZE: usize = 10> {
    data: [Option<T>; SIZE],
    start: usize,
    size: usize,
}

impl<T: Copy + Clone, const SIZE: usize> Fifo<T, SIZE> {
    pub fn new() -> Self {
        Self {
            data: [None; SIZE],
            start: 0,
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn push(&mut self, data: T) {
        debug_assert!(self.size != SIZE, "Trying to push to a full FIFO");

        let idx = self.start.wrapping_add(self.size) % SIZE;
        self.data[idx] = Some(data);
        self.size += 1;
    }

    pub fn pop(&mut self) -> T {
        debug_assert!(self.size != 0, "Trying to pop from a FIFO without any more values to pop");

        let value = self.data[self.start]
            .take()
            .expect("Expected a data in the FIFO");
        self.start = self.start.wrapping_add(1) % SIZE;
        self.size -= 1;
        value
    }

    pub fn clear(&mut self) {
        while self.size > 0 {
            self.pop();
        }
    }
}

impl<T: Copy + Clone, const SIZE: usize> Index<usize> for Fifo<T, SIZE> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index > self.size {
            panic!("Trying to access a value not yet pushed to the FIFO");
        }

        let index = self.start.wrapping_add(index) % SIZE;
        self.data[index].as_ref().expect("Expected a data in the FIFO at this index")
    }
}

impl<T: Copy + Clone, const SIZE: usize> IndexMut<usize> for Fifo<T, SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index > self.size {
            panic!("Trying to access a value not yet pushed to the FIFO");
        }

        let index = self.start.wrapping_add(index) % SIZE;
        self.data[index].as_mut().expect("Expected a data in the FIFO at this index")
    }
}