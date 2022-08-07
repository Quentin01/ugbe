extern crate sdl2;

use std::io;

use ugbe::gameboy;

const PIXEL_SCALE: usize = 4;
const FRAMES_TO_BLEND: usize = 1;

const GAMEBOY_FREQUENCY: usize = 4_194_304;
const T_CYCLE_DURATION: std::time::Duration =
    std::time::Duration::new(0, (1_000_000_000f64 / GAMEBOY_FREQUENCY as f64) as u32);

const TEXTURE_FORMAT: sdl2::pixels::PixelFormatEnum = sdl2::pixels::PixelFormatEnum::RGB24;
const BYTES_PER_PIXEL: u32 = 3;

const TEXTURE_WIDTH: u32 = (gameboy::screen::Screen::WIDTH * PIXEL_SCALE) as u32;
const TEXTURE_HEIGHT: u32 = (gameboy::screen::Screen::HEIGHT * PIXEL_SCALE) as u32;
const TEXTURE_PITCH: usize = (TEXTURE_WIDTH * BYTES_PER_PIXEL) as usize;

fn main() -> Result<(), io::Error> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("UGBE", TEXTURE_WIDTH, TEXTURE_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut texture_data = [0; (TEXTURE_WIDTH * TEXTURE_HEIGHT * BYTES_PER_PIXEL) as usize];

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_target(TEXTURE_FORMAT, TEXTURE_WIDTH, TEXTURE_HEIGHT)
        .unwrap();

    texture.update(None, &texture_data, TEXTURE_PITCH).unwrap();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut gameboy = gameboy::GameboyBuilder::new(
        "/home/quentin/git/ugbe/roms/boot.gb",
        "/home/quentin/git/ugbe/roms/ZeldaLinksAwakeningDX.gb",
    )?
    .build();

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
                _ => {}
            }
        }

        // Running the emulation until a LCDOff or a VBlank
        let (run_duration, elapsed_cycles) = {
            let before_run = std::time::Instant::now();
            let mut elapsed_cycles: u32 = 0;
            loop {
                elapsed_cycles += 1;

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
            (before_run.elapsed(), elapsed_cycles)
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

            texture.update(None, &texture_data, TEXTURE_PITCH).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();

            before_present.elapsed()
        };

        // Fix the framing time by sleeping if necessary
        let expected_frame_duration = T_CYCLE_DURATION * elapsed_cycles;
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
