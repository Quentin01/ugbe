mod bus;
mod cartridge;
pub mod clock;
mod components;
mod cpu;
mod interrupt;
pub mod joypad;
mod mmu;
mod ppu;
pub mod spu;
mod timer;
mod wram;

pub use ppu::screen;

pub struct GameboyBuilder {
    boot_rom: crate::bootrom::BootRom,
    cartridge: crate::cartridge::Cartridge,
    screen_config: screen::Config,
}

impl GameboyBuilder {
    // TODO: Remove the requirements of the boot rom and allow to construct a Gameboy without it
    // TODO: Check that the cartridge is ok (for example if CGB is required we won't be able to boot the game
    pub fn new(boot_rom: crate::bootrom::BootRom, cartridge: crate::cartridge::Cartridge) -> Self {
        Self {
            boot_rom,
            cartridge,
            screen_config: screen::Config::default(),
        }
    }

    pub fn set_screen_config(self, screen_config: screen::Config) -> Self {
        Self {
            screen_config,
            ..self
        }
    }

    pub fn set_screen_frame_blending(
        mut self,
        frame_blending: Option<screen::FrameBlending>,
    ) -> Self {
        self.screen_config.set_frame_blending(frame_blending);
        self
    }

    pub fn set_screen_color_grayscale(mut self) -> Self {
        self.screen_config
            .set_color_palette(screen::ColorPalette::new_grayscale());
        self
    }

    pub fn build(self) -> Gameboy {
        Gameboy {
            mmu: mmu::MMU::new(),
            boot_rom: self.boot_rom,
            cartridge: self.cartridge.into(),
            joypad: joypad::Joypad::new(),
            ppu: ppu::PPU::new(self.screen_config),
            spu: spu::Spu::new(),
            cpu: cpu::Cpu::new(),
            bus: bus::Bus::new(),
            interrupt: interrupt::Interrupt::new(),
            work_ram: wram::WorkRam::new(),
            high_ram: wram::WorkRam::new(),
            timer: timer::Timer::new(),
            clock: clock::Clock::new(),
        }
    }
}

pub struct Gameboy {
    mmu: mmu::MMU,
    boot_rom: crate::bootrom::BootRom,
    cartridge: cartridge::Cartridge,
    joypad: joypad::Joypad,
    ppu: ppu::PPU,
    spu: spu::Spu,
    cpu: cpu::Cpu,
    bus: bus::Bus,
    interrupt: interrupt::Interrupt,
    work_ram: wram::WorkRam<0x2000>,
    high_ram: wram::WorkRam<0x7F>,
    timer: timer::Timer,
    clock: clock::Clock,
}

impl Gameboy {
    pub fn tick(&mut self) -> (Option<screen::Event>, Option<spu::SampleFrame>) {
        if self.clock.is_m_cycle() {
            let memory_operation = self.cpu.tick(&self.bus, &mut self.interrupt);
            self.bus.tick(
                memory_operation,
                &mut self.mmu,
                &mut components::MMUContext {
                    joypad: &mut self.joypad,
                    ppu: &mut self.ppu,
                    spu: &mut self.spu,
                    timer: &mut self.timer,
                    interrupt: &mut self.interrupt,
                    boot_rom: &mut self.boot_rom,
                    cartridge: &mut self.cartridge,
                    work_ram: &mut self.work_ram,
                    high_ram: &mut self.high_ram,
                },
            );
        }

        let screen_event = self.ppu.tick(&mut self.interrupt);
        self.spu.tick();

        let sample_frame = if self.clock.is_apu_cycle() {
            Some(self.spu.sample_frame())
        } else {
            None
        };

        self.timer.tick(&mut self.interrupt);
        self.joypad.tick(&mut self.interrupt);

        self.clock.tick();

        (screen_event, sample_frame)
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
