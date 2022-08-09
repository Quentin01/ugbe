extern crate sdl2;

use anyhow::{Context, Result};

use ugbe::{bootrom, cartridge, gameboy};

const BOOT_ROM_PATH: &str = "/home/quentin/git/ugbe/roms/boot.gb";
const ROM_PATH: &str = "/home/quentin/git/ugbe/roms/ZeldaLinksAwakeningDX.gb";

const PIXEL_SCALE: usize = 4;
const FRAMES_TO_BLEND: usize = 1;

const TEXTURE_FORMAT: sdl2::pixels::PixelFormatEnum = sdl2::pixels::PixelFormatEnum::RGB24;
const BYTES_PER_PIXEL: u32 = 3;

const TEXTURE_WIDTH: u32 = (gameboy::screen::Screen::WIDTH * PIXEL_SCALE) as u32;
const TEXTURE_HEIGHT: u32 = (gameboy::screen::Screen::HEIGHT * PIXEL_SCALE) as u32;
const TEXTURE_PITCH: usize = (TEXTURE_WIDTH * BYTES_PER_PIXEL) as usize;

#[derive(Debug)]
struct SdlError(String);

impl std::fmt::Display for SdlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SdlError {}

fn main() -> Result<()> {
    let boot_rom = bootrom::BootRom::from_path(BOOT_ROM_PATH)
        .context(format!("unable to parse boom rom '{}'", BOOT_ROM_PATH))?;
    let cartridge = cartridge::Cartridge::from_rom_path(ROM_PATH)
        .context(format!("unable to parse rom '{}'", ROM_PATH))?;

    println!("Cartridge:");
    println!("    Title: {}", cartridge.header().title);
    println!("    Kind: {}", cartridge.header().kind);
    println!("    ROM size: {}", cartridge.header().rom_size);
    println!("    RAM size: {}", cartridge.header().ram_size);
    println!(
        "    Manufacturer code: {}",
        cartridge.header().manufacturer_code
    );
    println!("    Licensee code: {}", cartridge.header().licensee_code);
    println!(
        "    Destination code: {}",
        cartridge.header().destination_code
    );
    println!("    CGB support: {}", cartridge.header().cgb_suppport);
    println!("    SGB support: {}", cartridge.header().sgb_suppport);
    println!("    Version: {}", cartridge.header().rom_version);

    let sdl_context = sdl2::init()
        .map_err(SdlError)
        .context("unable to init SDL2")?;
    let video_subsystem = sdl_context
        .video()
        .map_err(SdlError)
        .context("unable to init SDL2 video subsystem")?;
    let game_controller_subsystem = sdl_context
        .game_controller()
        .map_err(SdlError)
        .context("unable to init SDL2 game controller subsystem")?;

    let mut controllers = vec![];

    let window = video_subsystem
        .window(
            &format!("UGBE - {}", cartridge.header().title),
            TEXTURE_WIDTH,
            TEXTURE_HEIGHT,
        )
        .position_centered()
        .build()
        .context("unable to init SDL2 window")?;

    let mut texture_data = [0; (TEXTURE_WIDTH * TEXTURE_HEIGHT * BYTES_PER_PIXEL) as usize];

    let mut canvas = window
        .into_canvas()
        .build()
        .context("unable to init SDL2 canvas")?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_target(TEXTURE_FORMAT, TEXTURE_WIDTH, TEXTURE_HEIGHT)
        .context("unable to init SDL2 texture")?;

    texture
        .update(None, &texture_data, TEXTURE_PITCH)
        .context("unable to update SDL2 texture")?;
    canvas
        .copy(&texture, None, None)
        .map_err(SdlError)
        .context("unable to copy SDL2 texture inside the SDL2 canvas")?;
    canvas.present();

    let mut event_pump = sdl_context
        .event_pump()
        .map_err(SdlError)
        .context("unable to init SDL2 event pump")?;

    let mut gameboy = gameboy::GameboyBuilder::new(boot_rom, cartridge).build();

    let mut idx_frame = 0;
    let mut frames = [[sdl2::pixels::Color::RGB(255, 255, 255);
        gameboy::screen::Screen::WIDTH * gameboy::screen::Screen::HEIGHT];
        FRAMES_TO_BLEND];

    let mut lag_duration = std::time::Duration::new(0, 0);

    let mut before_frame = std::time::Instant::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                sdl2::event::Event::ControllerDeviceAdded { which, .. } => {
                    if let Ok(controller) = game_controller_subsystem.open(which) {
                        println!("Successfully added controller '{}'", controller.name());
                        controllers.push(controller);
                    } else {
                        println!("Failed to open the added controller with index = {}", which);
                    }
                }
                sdl2::event::Event::ControllerDeviceRemoved { which, .. } => {
                    let idx = controllers
                        .iter()
                        .position(move |controller| controller.instance_id() == which);

                    match idx {
                        Some(idx) => {
                            println!(
                                "Successfully removed controller '{}'",
                                controllers[idx].name()
                            );
                            controllers.remove(idx);
                        }
                        None => {}
                    }
                }
                sdl2::event::Event::ControllerButtonUp { button, .. } => match button {
                    sdl2::controller::Button::A => {
                        gameboy.joypad().keyup(gameboy::joypad::Button::A)
                    }
                    sdl2::controller::Button::B => {
                        gameboy.joypad().keyup(gameboy::joypad::Button::B)
                    }
                    sdl2::controller::Button::Start => {
                        gameboy.joypad().keyup(gameboy::joypad::Button::Start)
                    }
                    sdl2::controller::Button::Back => {
                        gameboy.joypad().keyup(gameboy::joypad::Button::Select)
                    }
                    sdl2::controller::Button::DPadUp => {
                        gameboy.joypad().keyup(gameboy::joypad::Button::Up)
                    }
                    sdl2::controller::Button::DPadDown => {
                        gameboy.joypad().keyup(gameboy::joypad::Button::Down)
                    }
                    sdl2::controller::Button::DPadLeft => {
                        gameboy.joypad().keyup(gameboy::joypad::Button::Left)
                    }
                    sdl2::controller::Button::DPadRight => {
                        gameboy.joypad().keyup(gameboy::joypad::Button::Right)
                    }
                    _ => {}
                },
                sdl2::event::Event::ControllerButtonDown { button, .. } => match button {
                    sdl2::controller::Button::A => {
                        gameboy.joypad().keydown(gameboy::joypad::Button::A)
                    }
                    sdl2::controller::Button::B => {
                        gameboy.joypad().keydown(gameboy::joypad::Button::B)
                    }
                    sdl2::controller::Button::Start => {
                        gameboy.joypad().keydown(gameboy::joypad::Button::Start)
                    }
                    sdl2::controller::Button::Back => {
                        gameboy.joypad().keydown(gameboy::joypad::Button::Select)
                    }
                    sdl2::controller::Button::DPadUp => {
                        gameboy.joypad().keydown(gameboy::joypad::Button::Up)
                    }
                    sdl2::controller::Button::DPadDown => {
                        gameboy.joypad().keydown(gameboy::joypad::Button::Down)
                    }
                    sdl2::controller::Button::DPadLeft => {
                        gameboy.joypad().keydown(gameboy::joypad::Button::Left)
                    }
                    sdl2::controller::Button::DPadRight => {
                        gameboy.joypad().keydown(gameboy::joypad::Button::Right)
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Running the emulation until a LCDOff or a VBlank
        let (run_duration, expected_frame_duration) = {
            let before_run = std::time::Instant::now();
            let before_emulation = gameboy.clock().now();
            loop {
                match gameboy.tick() {
                    Some(screen_event) => match screen_event {
                        gameboy::screen::Event::VBlank => break,
                        gameboy::screen::Event::LCDOn => {}
                        gameboy::screen::Event::LCDOff => {
                            break;
                        }
                    },
                    None => {}
                }
            }
            (
                before_run.elapsed(),
                before_emulation.elapsed(gameboy.clock()),
            )
        };

        // Update the texture with the pixels from the gameboy screen
        let render_duration = {
            let before_render = std::time::Instant::now();

            // Add the frame in the list of frames
            let pixels = gameboy.screen().pixels();
            for x in 0..gameboy::screen::Screen::WIDTH {
                for y in 0..gameboy::screen::Screen::HEIGHT {
                    let pixel_idx = y * gameboy::screen::Screen::WIDTH + x;

                    frames[idx_frame][pixel_idx] = match pixels[pixel_idx] {
                        gameboy::screen::Color::Off => sdl2::pixels::Color::RGB(255, 255, 255),
                        gameboy::screen::Color::White => sdl2::pixels::Color::RGB(255, 255, 255),
                        gameboy::screen::Color::LightGray => {
                            sdl2::pixels::Color::RGB(170, 170, 170)
                        }
                        gameboy::screen::Color::DarkGray => sdl2::pixels::Color::RGB(85, 85, 85),
                        gameboy::screen::Color::Black => sdl2::pixels::Color::RGB(0, 0, 0),
                    };
                }
            }

            // Change the texture with a blending of all the frames
            for x in 0..gameboy::screen::Screen::WIDTH {
                for y in 0..gameboy::screen::Screen::HEIGHT {
                    let color = {
                        let mut red: f64 = 0.0;
                        let mut green: f64 = 0.0;
                        let mut blue: f64 = 0.0;

                        let mut total_coeff = 0.0;

                        // Go from frame to frames from the older one first
                        for offset in (0..FRAMES_TO_BLEND).rev() {
                            let idx = (idx_frame + offset) % FRAMES_TO_BLEND;

                            // More recent frames have less influence on the frame
                            let coeff = 1.0
                                - ((FRAMES_TO_BLEND - offset - 1) as f64
                                    * (1.0 / FRAMES_TO_BLEND as f64));
                            total_coeff += coeff;

                            red += frames[idx][y * gameboy::screen::Screen::WIDTH + x].r as f64
                                * coeff;
                            green += frames[idx][y * gameboy::screen::Screen::WIDTH + x].g as f64
                                * coeff;
                            blue += frames[idx][y * gameboy::screen::Screen::WIDTH + x].b as f64
                                * coeff;
                        }

                        sdl2::pixels::Color::RGB(
                            (red / total_coeff) as u8,
                            (green / total_coeff) as u8,
                            (blue / total_coeff) as u8,
                        )
                    };

                    for x_zoom in 0..PIXEL_SCALE {
                        let x = x * PIXEL_SCALE + x_zoom;
                        for y_zoom in 0..PIXEL_SCALE {
                            let y = y * PIXEL_SCALE + y_zoom;

                            let base_idx = y * TEXTURE_PITCH + x * 3;

                            texture_data[base_idx] = color.r;
                            texture_data[base_idx + 1] = color.g;
                            texture_data[base_idx + 2] = color.b;
                        }
                    }
                }
            }

            idx_frame = (idx_frame + 1) % FRAMES_TO_BLEND;
            before_render.elapsed()
        };

        // Present the texture on the screen
        let present_duration = {
            let before_present = std::time::Instant::now();

            texture
                .update(None, &texture_data, TEXTURE_PITCH)
                .context("unable to update SDL2 texture")?;
            canvas
                .copy(&texture, None, None)
                .map_err(SdlError)
                .context("unable to copy the SDL2 texture inside the SDL2 canvas")?;
            canvas.present();

            before_present.elapsed()
        };

        // Fix the framing time by sleeping if necessary
        let frame_duration = before_frame.elapsed();

        if frame_duration + lag_duration < expected_frame_duration {
            ::std::thread::sleep(expected_frame_duration - (frame_duration + lag_duration));
        } else if frame_duration > expected_frame_duration {
            #[cfg(debug_assertions)]
            println!(
                "Warning: A frame took more time than expected (frame={:?} / run={:?} / render={:?} / present={:?} / expected={:?} / lag={:?})",
                frame_duration, run_duration, render_duration, present_duration,
                expected_frame_duration, lag_duration
            );
        }

        lag_duration = before_frame.elapsed() + lag_duration - expected_frame_duration;
        before_frame = std::time::Instant::now();
    }

    Ok(())
}
