mod bus;
mod cartridge;
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
            t_cycle_count: 0,
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
    t_cycle_count: usize,
}

impl Gameboy {
    pub fn tick(&mut self) -> Option<screen::Event> {
        // TODO: Currently the CPU is ticking every m-cycle and the hardware needs it every t-cycle
        //       In the future, this should be handled by ticking every t-cycle for each
        if self.t_cycle_count % 4 == 0 {
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

        self.t_cycle_count = self.t_cycle_count.wrapping_add(1);

        screen_event
    }

    pub fn keydown(&mut self, button: joypad::Button) {
        self.joypad.keydown(button)
    }

    pub fn keyup(&mut self, button: joypad::Button) {
        self.joypad.keyup(button)
    }

    pub fn screen(&self) -> &screen::Screen {
        self.ppu.screen()
    }
}
