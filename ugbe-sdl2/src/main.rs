use std::io;

use ugbe::gameboy;

fn main() -> Result<(), io::Error> {
    gameboy::GameboyBuilder::new("/data/ugbe/roms/boot.gb")?
        .build()
        .run();
    Ok(())
}
