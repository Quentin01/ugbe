mod bus;
mod cartridge;
pub mod clock;
mod components;
mod cpu;
mod interrupt;
pub mod joypad;
mod mmu;
mod ppu;
mod timer;
mod wram;

pub use ppu::screen;

pub struct GameboyBuilder {
    boot_rom: crate::bootrom::BootRom,
    cartridge: crate::cartridge::Cartridge,
}

impl GameboyBuilder {
    // TODO: Remove the requirements of the boot rom and allow to construct a Gameboy without it
    // TODO: Check that the cartridge is ok (for example if CGB is required we won't be able to boot the game
    pub fn new(boot_rom: crate::bootrom::BootRom, cartridge: crate::cartridge::Cartridge) -> Self {
        Self {
            boot_rom,
            cartridge,
        }
    }

    pub fn build(self) -> Gameboy {
        Gameboy {
            mmu: mmu::Mmu::new(),
            boot_rom: self.boot_rom,
            cartridge: self.cartridge.into(),
            joypad: joypad::Joypad::new(),
            ppu: ppu::Ppu::new(),
            cpu: cpu::Cpu::new(),
            bus: bus::Bus::new(),
            interrupt: interrupt::Interrupt::new(),
            work_ram: wram::WorkRam::new(),
            timer: timer::Timer::new(),
            clock: clock::Clock::new(),
        }
    }
}

pub struct Gameboy {
    mmu: mmu::Mmu,
    boot_rom: crate::bootrom::BootRom,
    cartridge: cartridge::Cartridge,
    joypad: joypad::Joypad,
    ppu: ppu::Ppu,
    cpu: cpu::Cpu,
    bus: bus::Bus,
    interrupt: interrupt::Interrupt,
    work_ram: wram::WorkRam<0x1000>,
    timer: timer::Timer,
    clock: clock::Clock,
}

impl Gameboy {
    pub fn tick(&mut self) -> Option<screen::Event> {
        if self.clock.is_m_cycle() {
            let memory_operation = self.cpu.tick(&self.bus, &mut self.interrupt);
            self.bus.tick(
                memory_operation,
                &mut self.mmu,
                &mut components::MmuContext {
                    joypad: &mut self.joypad,
                    ppu: &mut self.ppu,
                    timer: &mut self.timer,
                    interrupt: &mut self.interrupt,
                    boot_rom: &mut self.boot_rom,
                    cartridge: &mut self.cartridge,
                    work_ram: &mut self.work_ram,
                },
            );
        }

        let screen_event = self.ppu.tick(&mut self.interrupt);
        self.timer.tick(&mut self.interrupt);
        self.joypad.tick(&mut self.interrupt);

        self.clock.tick();

        screen_event
    }

    pub fn clock(&self) -> &clock::Clock {
        &self.clock
    }

    pub fn joypad(&mut self) -> &mut joypad::Joypad {
        &mut self.joypad
    }

    pub fn screen(&self) -> &screen::Screen {
        self.ppu.screen()
    }
}
