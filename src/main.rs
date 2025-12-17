#![no_main]
#![no_std]

extern crate alloc;

mod buffer;

use core::time::Duration;

use uefi::proto::console::gop::{BltPixel, GraphicsOutput};
use uefi::proto::console::text::Key;
use uefi::{boot, Result};
use uefi::{prelude::*, Char16};

use crate::buffer::Buffer;

#[derive(Clone, Copy)]
struct Ball {
    x: f64,
    y: f64,
    speed_x: f64,
    speed_y: f64,
    size: usize
}

const BALL_SIZE: usize = 7;

fn game() -> Result {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let (width, height) = gop.current_mode_info().resolution();
    let mut buffer = Buffer::new(width, height);

    let mut running = true;

    let mut ball = Ball {
        x: ((width / 2) - (BALL_SIZE / 2)) as f64,
        y: ((height / 2) - (BALL_SIZE / 2)) as f64,
        speed_x: 5.0,
        speed_y: 5.0,
        size: 7
    };

    while running {
        while let Ok(Some(key)) = system::with_stdin(|stdin| stdin.read_key()) {
            match key {
                Key::Printable(c) => {
                    if c == Char16::try_from('q').unwrap_or_default()
                        || c == Char16::try_from('Q').unwrap_or_default()
                    {
                        running = false;
                    }
                }
                _ => {}
            }
        }

        ball.x += ball.speed_x;
        ball.y += ball.speed_y;

        if ball.x >= width as f64 - ball.size as f64 {
            ball.x = width as f64 - ball.size as f64;
            ball.speed_x = -ball.speed_x;
        } else if ball.x <= 0.0 {
            ball.x = 0.0;
            ball.speed_x = -ball.speed_x;
        }

        // Vertical collision detection and response
        if ball.y >= height as f64 - ball.size as f64 {
            ball.y = height as f64 - ball.size as f64;
            ball.speed_y = -ball.speed_y;
        } else if ball.y <= 0.0 {
            ball.y = 0.0;
            ball.speed_y = -ball.speed_y;
        }

        //buffer.clear();
        buffer.rectangle(
            ball.x as usize,
            ball.y as usize,
            ball.size,
            ball.size,
            BltPixel::new(255, 255, 255),
            true,
        );
        let _ = buffer.blit(&mut gop);

        boot::stall(Duration::from_millis(0));
    }

    Ok(())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    game().unwrap();
    Status::SUCCESS
}
