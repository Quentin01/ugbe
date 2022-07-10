use std::{fs, io, path::Path};

mod cpu;
mod hardware;

pub use hardware::ppu::screen;

pub struct GameboyBuilder {
    boot_rom: hardware::bootrom::BootRom,
    cartbridge: hardware::cartbridge::Cartbridge,
    renderer: Option<Box<dyn screen::Renderer>>,
}

impl GameboyBuilder {
    pub fn new<P: ?Sized + AsRef<Path>>(boot_rom_path: &P, rom_path: &P) -> Result<Self, io::Error> {
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
            renderer: None,
        })
    }

    pub fn add_renderer(mut self, renderer: Box<dyn screen::Renderer>) -> Self {
        self.renderer = Some(renderer);
        self
    }

    pub fn build(self) -> Gameboy {
        Gameboy {
            cpu: cpu::Cpu::new(),
            hardware: hardware::Hardware::new(self.boot_rom, self.cartbridge, self.renderer),
        }
    }
}

#[derive(Debug)]
pub struct Gameboy {
    cpu: cpu::Cpu,
    hardware: hardware::Hardware,
}

impl Gameboy {
    pub fn run(&mut self, cycle_count: usize) {
        // TODO: Currently the CPU is ticking every m-cycle and the hardware needs it every t-cycle
        //       In the future, this should be handled by ticking every t-cycle for each
        for t_cycle in 0..cycle_count {
            if t_cycle % 4 == 0 {
                self.cpu.tick(&mut self.hardware);
            }
            
            self.hardware.tick();
        }
    }
}
