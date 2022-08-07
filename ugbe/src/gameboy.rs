use std::{fs, io, path::Path};

mod bus;
mod cpu;
mod hardware;
mod interrupt;
mod mmu;

pub use hardware::ppu::screen;

pub struct GameboyBuilder {
    boot_rom: hardware::bootrom::BootRom,
    cartbridge: hardware::cartbridge::Cartbridge,
}

impl GameboyBuilder {
    pub fn new<P: ?Sized + AsRef<Path>>(
        boot_rom_path: &P,
        rom_path: &P,
    ) -> Result<Self, io::Error> {
        let boot_rom_file = fs::File::open(boot_rom_path)?;
        let mut boot_rom_reader = io::BufReader::new(boot_rom_file);
        let mut boot_rom_buffer = Vec::new();

        io::Read::read_to_end(&mut boot_rom_reader, &mut boot_rom_buffer)?;

        let rom_file = fs::File::open(rom_path)?;
        let mut rom_reader = io::BufReader::new(rom_file);
        let mut rom_buffer = Vec::new();

        io::Read::read_to_end(&mut rom_reader, &mut rom_buffer)?;

        Ok(Self {
            boot_rom: boot_rom_buffer.into(),
            cartbridge: rom_buffer.into(),
        })
    }

    pub fn build(self) -> Gameboy {
        Gameboy {
            cpu: cpu::Cpu::new(),
            hardware: hardware::Hardware::new(self.boot_rom, self.cartbridge),
            bus: bus::Bus::new(),
            t_cycle_count: 0,
        }
    }
}

pub struct Gameboy {
    cpu: cpu::Cpu,
    hardware: hardware::Hardware,
    bus: bus::Bus,
    t_cycle_count: usize,
}

impl Gameboy {
    pub fn tick(&mut self) -> Option<screen::Event> {
        // TODO: Currently the CPU is ticking every m-cycle and the hardware needs it every t-cycle
        //       In the future, this should be handled by ticking every t-cycle for each
        if self.t_cycle_count % 4 == 0 {
            let memory_operation = self.cpu.tick(&self.bus, &mut self.hardware);
            self.bus.tick(memory_operation, &mut self.hardware);
        }

        let screen_event = self.hardware.tick();

        self.t_cycle_count = self.t_cycle_count.wrapping_add(1);

        screen_event
    }

    pub fn screen(&self) -> &screen::Screen {
        self.hardware.screen()
    }
}
