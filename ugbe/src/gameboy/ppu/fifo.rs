use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
        debug_assert!(
            self.size != SIZE,
            "Trying to push to a full FIFO (start={}, size={})",
            self.start,
            self.size
        );

        let idx = self.start.wrapping_add(self.size) % SIZE;
        self.data[idx] = Some(data);
        self.size += 1;
    }

    pub fn pop(&mut self) -> T {
        debug_assert!(
            self.size != 0,
            "Trying to pop from a FIFO without any more values to pop (start={}, size={})",
            self.start,
            self.size
        );

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
        self.data[index]
            .as_ref()
            .expect("Expected a data in the FIFO at this index")
    }
}

impl<T: Copy + Clone, const SIZE: usize> IndexMut<usize> for Fifo<T, SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index > self.size {
            panic!("Trying to access a value not yet pushed to the FIFO");
        }

        let index = self.start.wrapping_add(index) % SIZE;
        self.data[index]
            .as_mut()
            .expect("Expected a data in the FIFO at this index")
    }
}

#[cfg(test)]
mod tests {
    use super::Fifo;

    #[test]
    fn basic_fifo() {
        const VALUES: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

        let mut fifo = Fifo::<u8, 8>::new();

        for (idx, value) in VALUES.into_iter().enumerate() {
            fifo.push(value);

            assert_eq!(fifo.len(), idx + 1);
            assert_eq!(fifo[idx], value);
        }

        for (idx, value) in VALUES.into_iter().enumerate() {
            let popped_value = fifo.pop();

            assert_eq!(popped_value, value);
            assert_eq!(fifo.len(), 7 - idx);
        }
    }
}
