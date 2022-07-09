use std::{fs, io, path::Path};

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
            cpu: cpu::Cpu::new(),
            hardware: hardware::Hardware::new(self.boot_rom),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gameboy {
    cpu: cpu::Cpu,
    hardware: hardware::Hardware,
}

impl Gameboy {
    pub fn run(&mut self) {
        // TODO: Currently the CPU is ticking every m-cycle and the hardware needs it every t-cycle
        //       In the future, this should be handled by ticking every t-cycle for each
        loop {
            self.cpu.tick(&mut self.hardware);

            for _ in 0..4 {
                self.hardware.tick();
            }
        }
    }
}
