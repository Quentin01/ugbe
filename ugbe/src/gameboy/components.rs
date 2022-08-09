#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InterruptKind {
    VBlank = 1 << 0,
    Stat = 1 << 1,
    Timer = 1 << 2,
    Serial = 1 << 3,
    Joypad = 1 << 4,
}

pub trait InterruptLine {
    fn highest_priority(&self) -> Option<InterruptKind>;
    fn ack(&mut self, kind: InterruptKind);
    fn request(&mut self, kind: InterruptKind);
    fn flags_not_empty(&self) -> bool;
}

#[derive(Debug)]
pub struct MmuContext<'components> {
    pub joypad: &'components mut super::joypad::Joypad,
    pub ppu: &'components mut super::ppu::Ppu,
    pub timer: &'components mut super::timer::Timer,
    pub interrupt: &'components mut super::interrupt::Interrupt,
    pub boot_rom: &'components mut crate::bootrom::BootRom,
    pub cartridge: &'components mut super::cartridge::Cartridge,
    pub work_ram: &'components mut super::wram::WorkRam<0x1000>,
}

pub trait Mmu {
    fn read_byte(&self, ctx: &MmuContext, address: u16) -> u8;

    fn read_word(&self, ctx: &MmuContext, address: u16) -> u16 {
        u16::from_le_bytes([
            self.read_byte(ctx, address),
            self.read_byte(ctx, address.wrapping_add(1)),
        ])
    }

    fn write_byte(&mut self, ctx: &mut MmuContext, address: u16, value: u8);
}
