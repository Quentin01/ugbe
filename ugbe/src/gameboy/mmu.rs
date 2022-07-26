pub trait Mmu {
    fn read_byte(&self, address: u16) -> u8;

    fn read_word(&self, address: u16) -> u16 {
        u16::from_le_bytes([
            self.read_byte(address),
            self.read_byte(address.wrapping_add(1)),
        ])
    }

    fn write_byte(&mut self, address: u16, value: u8);
}
