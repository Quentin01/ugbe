extern crate anyhow;
extern crate blip_buf;
extern crate crossbeam_channel;
extern crate sdl2;

use anyhow::{Context, Result};

use ugbe::{bootrom, cartridge, gameboy};

const BOOT_ROM_PATH: &str = "/home/quentin/git/ugbe/roms/boot.gb";
const ROM_PATH: &str = "/home/quentin/git/ugbe/roms/ZeldaLinksAwakeningDX.gb";

const PIXEL_SCALE: u32 = 4;

const TEXTURE_FORMAT: sdl2::pixels::PixelFormatEnum = sdl2::pixels::PixelFormatEnum::RGB555;
const BYTES_PER_PIXEL: u32 = 2;

const TEXTURE_WIDTH: u32 = gameboy::screen::Screen::WIDTH as u32;
const TEXTURE_HEIGHT: u32 = gameboy::screen::Screen::HEIGHT as u32;
const TEXTURE_PITCH: usize = (TEXTURE_WIDTH * BYTES_PER_PIXEL) as usize;

const SAMPLE_RATE: usize = 48000;
const SAMPLE_COUNT_PER_EVENT: usize = 1024;
const SAMPLE_BUFFER_SIZE: u16 = 512;

#[derive(Debug)]
struct SdlError(String);

impl std::fmt::Display for SdlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SdlError {}

enum ExternalGameboyEvent {
    Stop,
    Keyup(gameboy::joypad::Button),
    Keydown(gameboy::joypad::Button),
}

enum InternalGameboyEvent {
    VBlank([u8; (TEXTURE_WIDTH * TEXTURE_HEIGHT * BYTES_PER_PIXEL) as usize]),
    AudioSamples([gameboy::spu::SampleFrame; SAMPLE_COUNT_PER_EVENT]),
}

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

    let audio_subsystem = sdl_context
        .audio()
        .map_err(SdlError)
        .context("unable to init SDL2 audio subsystem")?;

    let game_controller_subsystem = sdl_context
        .game_controller()
        .map_err(SdlError)
        .context("unable to init SDL2 game controller subsystem")?;

    let mut controllers = vec![];

    let window = video_subsystem
        .window(
            &format!("UGBE - {}", cartridge.header().title),
            TEXTURE_WIDTH * PIXEL_SCALE,
            TEXTURE_HEIGHT * PIXEL_SCALE,
        )
        .position_centered()
        .build()
        .context("unable to init SDL2 window")?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .context("unable to init SDL2 canvas")?;

    canvas
        .set_integer_scale(true)
        .map_err(SdlError)
        .context("unable to enable integer scaling")?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_target(TEXTURE_FORMAT, TEXTURE_WIDTH, TEXTURE_HEIGHT)
        .context("unable to init SDL2 texture")?;

    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context
        .event_pump()
        .map_err(SdlError)
        .context("unable to init SDL2 event pump")?;

    let mut texture_data = [0; (TEXTURE_WIDTH * TEXTURE_HEIGHT * BYTES_PER_PIXEL) as usize];

    let audio_desired_spec = sdl2::audio::AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32),
        channels: Some(2),
        samples: Some(SAMPLE_BUFFER_SIZE),
    };

    let audio_queue = audio_subsystem
        .open_queue::<i16, _>(None, &audio_desired_spec)
        .map_err(SdlError)
        .context("unable to init SDL2 audio queue")?;

    audio_queue.resume();

    let mut audio_buff_left = blip_buf::BlipBuf::new((SAMPLE_COUNT_PER_EVENT * 10) as u32);
    let mut audio_buff_right = blip_buf::BlipBuf::new((SAMPLE_COUNT_PER_EVENT * 10) as u32);

    audio_buff_left.set_rates(gameboy::spu::SAMPLE_RATE as f64, SAMPLE_RATE as f64);
    audio_buff_right.set_rates(gameboy::spu::SAMPLE_RATE as f64, SAMPLE_RATE as f64);

    let mut audio_buff_previous_left = 0;
    let mut audio_buff_previous_right = 0;

    let (sender_internal, receiver_internal) = crossbeam_channel::unbounded();
    let (sender_external, receiver_external) = crossbeam_channel::unbounded();

    let gameboy = gameboy::GameboyBuilder::new(boot_rom, cartridge)
        .set_screen_color_grayscale()
        .set_screen_frame_blending(None)
        .build();

    let emulation_thread =
        { std::thread::spawn(|| run_emulation(gameboy, sender_internal, receiver_external)) };

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => {
                    sender_external.send(ExternalGameboyEvent::Stop)?;
                    break 'running;
                }
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
                    sdl2::controller::Button::A => sender_external
                        .send(ExternalGameboyEvent::Keyup(gameboy::joypad::Button::A))?,
                    sdl2::controller::Button::B => sender_external
                        .send(ExternalGameboyEvent::Keyup(gameboy::joypad::Button::B))?,
                    sdl2::controller::Button::Start => sender_external
                        .send(ExternalGameboyEvent::Keyup(gameboy::joypad::Button::Start))?,
                    sdl2::controller::Button::Back => sender_external
                        .send(ExternalGameboyEvent::Keyup(gameboy::joypad::Button::Select))?,
                    sdl2::controller::Button::DPadUp => sender_external
                        .send(ExternalGameboyEvent::Keyup(gameboy::joypad::Button::Up))?,
                    sdl2::controller::Button::DPadDown => sender_external
                        .send(ExternalGameboyEvent::Keyup(gameboy::joypad::Button::Down))?,
                    sdl2::controller::Button::DPadLeft => sender_external
                        .send(ExternalGameboyEvent::Keyup(gameboy::joypad::Button::Left))?,
                    sdl2::controller::Button::DPadRight => sender_external
                        .send(ExternalGameboyEvent::Keyup(gameboy::joypad::Button::Right))?,
                    _ => {}
                },
                sdl2::event::Event::ControllerButtonDown { button, .. } => match button {
                    sdl2::controller::Button::A => sender_external
                        .send(ExternalGameboyEvent::Keydown(gameboy::joypad::Button::A))?,
                    sdl2::controller::Button::B => sender_external
                        .send(ExternalGameboyEvent::Keydown(gameboy::joypad::Button::B))?,
                    sdl2::controller::Button::Start => sender_external.send(
                        ExternalGameboyEvent::Keydown(gameboy::joypad::Button::Start),
                    )?,
                    sdl2::controller::Button::Back => sender_external.send(
                        ExternalGameboyEvent::Keydown(gameboy::joypad::Button::Select),
                    )?,
                    sdl2::controller::Button::DPadUp => sender_external
                        .send(ExternalGameboyEvent::Keydown(gameboy::joypad::Button::Up))?,
                    sdl2::controller::Button::DPadDown => sender_external
                        .send(ExternalGameboyEvent::Keydown(gameboy::joypad::Button::Down))?,
                    sdl2::controller::Button::DPadLeft => sender_external
                        .send(ExternalGameboyEvent::Keydown(gameboy::joypad::Button::Left))?,
                    sdl2::controller::Button::DPadRight => sender_external.send(
                        ExternalGameboyEvent::Keydown(gameboy::joypad::Button::Right),
                    )?,
                    _ => {}
                },
                _ => {}
            }
        }

        // Wait for VBlank from the emulation
        {
            let mut vblank = false;
            for event in receiver_internal.try_iter() {
                match event {
                    InternalGameboyEvent::VBlank(frame_data) => {
                        vblank = true;
                        texture_data = frame_data;
                    }
                    InternalGameboyEvent::AudioSamples(sample_frames) => {
                        let mut audio_buff_clock_time = 0;

                        for sample_frame in sample_frames {
                            audio_buff_left.add_delta(
                                audio_buff_clock_time,
                                sample_frame.left() - audio_buff_previous_left,
                            );
                            audio_buff_previous_left = sample_frame.left() as i32;

                            audio_buff_right.add_delta(
                                audio_buff_clock_time,
                                sample_frame.right() - audio_buff_previous_right,
                            );
                            audio_buff_previous_right = sample_frame.right() as i32;

                            audio_buff_clock_time += 1;
                        }

                        audio_buff_left.end_frame(audio_buff_clock_time - 1);
                        audio_buff_right.end_frame(audio_buff_clock_time - 1);

                        while audio_buff_left.samples_avail() > 0 {
                            let mut samples = [0; 2048];
                            let nb_samples_left = audio_buff_left.read_samples(&mut samples, true);
                            let nb_samples_right =
                                audio_buff_right.read_samples(&mut samples[1..], true);
                            assert!(nb_samples_left == nb_samples_right);

                            audio_queue
                                .queue_audio(&samples[0..(nb_samples_left + nb_samples_right)])
                                .map_err(SdlError)
                                .context("unable to push audio samples to queue")?;
                        }
                    }
                }
            }

            if !vblank {
                continue;
            }
        }

        // Update the canvas with the pixels from the gameboy screen
        {
            texture
                .update(None, &texture_data, TEXTURE_PITCH)
                .context("unable to update SDL2 texture")?;
            canvas
                .copy(&texture, None, None)
                .map_err(SdlError)
                .context("unable to copy the SDL2 texture inside the SDL2 canvas")?;
        };

        // Present the texture on the screen
        canvas.present();
    }

    emulation_thread.join().expect("unable to join the thread");

    Ok(())
}

fn run_emulation(
    mut gameboy: gameboy::Gameboy,
    internal_events: crossbeam_channel::Sender<InternalGameboyEvent>,
    external_events: crossbeam_channel::Receiver<ExternalGameboyEvent>,
) {
    let mut sample_frames_idx = 0;
    let mut sample_frames = [gameboy::spu::SampleFrame::default(); SAMPLE_COUNT_PER_EVENT];

    let mut lag_duration = std::time::Duration::new(0, 0);
    let mut before_frame = std::time::Instant::now();

    'running: loop {
        // Run the emulation until we need to display another frame
        let expected_frame_duration = {
            let before_emulation = gameboy.clock().now();

            // Deal with external events
            for event in external_events.try_iter() {
                match event {
                    ExternalGameboyEvent::Stop => break 'running,
                    ExternalGameboyEvent::Keyup(button) => gameboy.joypad().keyup(button),
                    ExternalGameboyEvent::Keydown(button) => gameboy.joypad().keydown(button),
                }
            }

            loop {
                let (screen_event, sample_frame) = gameboy.tick();

                match sample_frame {
                    Some(sample_frame) => {
                        sample_frames[sample_frames_idx] = sample_frame;
                        sample_frames_idx += 1;
                        if sample_frames_idx == SAMPLE_COUNT_PER_EVENT {
                            internal_events
                                .send(InternalGameboyEvent::AudioSamples(sample_frames))
                                .expect("Couldn't send audio samples");
                            sample_frames_idx = 0;
                        }
                    }
                    None => {}
                }

                match screen_event {
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

            before_emulation.elapsed(gameboy.clock())
        };

        // Send the new frame to the main thread
        {
            let frame_data = unsafe {
                std::mem::transmute::<
                    [gameboy::screen::Color; (TEXTURE_WIDTH * TEXTURE_HEIGHT) as usize],
                    [u8; (TEXTURE_WIDTH * TEXTURE_HEIGHT * BYTES_PER_PIXEL) as usize],
                >(*gameboy.screen().pixels())
            };

            internal_events
                .send(InternalGameboyEvent::VBlank(frame_data))
                .expect("Couldn't send VBlank");
        }

        let frame_duration = before_frame.elapsed();

        if frame_duration + lag_duration < expected_frame_duration {
            ::std::thread::sleep(expected_frame_duration - (frame_duration + lag_duration));
        } else if frame_duration > expected_frame_duration {
            #[cfg(debug_assertions)]
            println!(
                "Warning: A frame took more time than expected (frame={:?} / expected={:?} / lag={:?})",
                frame_duration, expected_frame_duration, lag_duration
            );
        }

        lag_duration = before_frame.elapsed() + lag_duration - expected_frame_duration;
        before_frame = std::time::Instant::now();
    }
}
