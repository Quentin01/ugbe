use std::{fs, io, path::Path};

use self::{cpu::Cpu, hardware::Hardware};

mod cpu;
mod hardware;

#[derive(Debug, Clone)]
pub struct GameboyBuilder {
    boot_rom: hardware::BootRom,
}

impl GameboyBuilder {
    pub fn new<P: ?Sized + AsRef<Path>>(boot_rom_path: &P) -> Result<GameboyBuilder, io::Error> {
        let boot_rom_file = fs::File::open(boot_rom_path)?;
        let mut reader = io::BufReader::new(boot_rom_file);
        let mut buffer = Vec::new();

        io::Read::read_to_end(&mut reader, &mut buffer)?;

        Ok(GameboyBuilder {
            boot_rom: buffer.into(),
        })
    }

    pub fn build(self) -> Gameboy {
        Gameboy {
            cpu: Cpu::new(),
            hardware: Hardware::new(self.boot_rom),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gameboy {
    cpu: cpu::Cpu,
    hardware: Hardware,
}

impl Gameboy {
    pub fn run(&mut self) {
        loop {
            self.cpu.tick(&mut self.hardware);
            self.hardware.tick();
        }
    }
}
