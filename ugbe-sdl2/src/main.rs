extern crate sdl2;

use std::{cell::RefCell, io, rc::Rc};

use ugbe::gameboy;

struct RendererData {
    vblank: bool,
    pixels:
        [gameboy::screen::Color; gameboy::screen::Screen::WIDTH * gameboy::screen::Screen::HEIGHT],
}

struct Renderer<'a> {
    data: &'a Rc<RefCell<RendererData>>,
}

impl<'a> gameboy::screen::Renderer for Renderer<'a> {
    fn on(&mut self) {
        println!("LCD is on");
    }

    fn off(&mut self) {
        println!("LCD is off");
    }

    fn vblank(&mut self, screen: &gameboy::screen::Screen) {
        self.data.borrow_mut().vblank = true;

        let pixels = &mut self.data.borrow_mut().pixels;
        for (i, color) in screen.pixels().iter().enumerate() {
            pixels[i] = *color;
        }
    }
}

impl<'a> Renderer<'a> {
    pub fn new(data: &'a Rc<RefCell<RendererData>>) -> Self {
        Self { data }
    }
}

fn main() -> Result<(), io::Error> {
    let renderer_data: &'static Rc<RefCell<RendererData>> =
        Box::leak(Box::new(Rc::new(RefCell::new(RendererData {
            vblank: false,
            pixels: [gameboy::screen::Color::White;
                gameboy::screen::Screen::WIDTH * gameboy::screen::Screen::HEIGHT],
        }))));

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let zoom: usize = 4;

    let window = video_subsystem
        .window(
            "UGBE",
            (gameboy::screen::Screen::WIDTH * zoom) as u32,
            (gameboy::screen::Screen::HEIGHT * zoom) as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut gameboy = gameboy::GameboyBuilder::new("/home/quentin/git/ugbe/roms/boot.gb", "/home/quentin/git/ugbe/roms/gb-test-roms/cpu_instrs/individual/06-ld r,r.gb")?
        .add_renderer(Box::new(Renderer::new(renderer_data)))
        .build();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let expected_duration_frame = std::time::Duration::new(0, (1_000_000_000f64 / 59.7) as u32);

    'running: loop {
        let now = std::time::Instant::now();

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

        let before_run = std::time::Instant::now();
        while !renderer_data.borrow().vblank {
            gameboy.run(1);
        }
        let duration_run = before_run.elapsed();

        renderer_data.borrow_mut().vblank = false;

        let before_render = std::time::Instant::now();
        let pixels = renderer_data.borrow().pixels;
        for x in 0..gameboy::screen::Screen::WIDTH {
            for y in 0..gameboy::screen::Screen::HEIGHT {
                let color = pixels[y * gameboy::screen::Screen::WIDTH + x];
                canvas.set_draw_color(match color {
                    gameboy::screen::Color::White => sdl2::pixels::Color::RGB(255, 255, 255),
                    gameboy::screen::Color::LightGray => sdl2::pixels::Color::RGB(170, 170, 170),
                    gameboy::screen::Color::DarkGray => sdl2::pixels::Color::RGB(85, 85, 85),
                    gameboy::screen::Color::Black => sdl2::pixels::Color::RGB(0, 0, 0),
                });

                canvas
                    .fill_rect(sdl2::rect::Rect::new(
                        (x * zoom) as i32,
                        (y * zoom) as i32,
                        zoom as u32,
                        zoom as u32,
                    ))
                    .unwrap();
            }
        }

        canvas.present();
        let duration_render = before_render.elapsed();

        let duration_frame = now.elapsed();
        if duration_frame > expected_duration_frame {
            // println!("Warning: A frame took more time than expected (frame={:?} / run={:?} / render={:?} / expected={:?})", duration_frame, duration_run, duration_render, expected_duration_frame);
        } else {
            ::std::thread::sleep(expected_duration_frame - duration_frame);
        }
    }

    Ok(())
}
