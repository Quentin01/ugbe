use std::io;

use ugbe::gameboy;

fn main() -> Result<(), io::Error> {
    gameboy::GameboyBuilder::new("/home/quentin/git/ugbe/roms/boot.gb")?
        .build()
        .run();
    Ok(())
}
