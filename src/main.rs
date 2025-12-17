#![no_main]
#![no_std]

extern crate alloc;

mod buffer;

use core::time::Duration;

use uefi::proto::console::gop::{BltPixel, GraphicsOutput};
use uefi::proto::console::text::{Key, ScanCode};
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

struct Paddle {
    y: f64,
    height: usize,
    width: usize
}

const BALL_SIZE: usize = 7;
const PADDLE_HEIGHT: usize = 40;
const PADDLE_WIDTH: usize = 6;
const PADDLE_SPEED: f64 = 40.0;
const PADDLE_DISTANCE_WALL: usize = 20;

const WHITE: BltPixel = BltPixel::new(255, 255, 255);

fn game() -> Result {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let (width, height) = gop.current_mode_info().resolution();
    let mut buffer = Buffer::new(width, height);

    let mut running = true;

    let mut ball = Ball {
        x: ((width / 2) - (BALL_SIZE / 2)) as f64,
        y: ((height / 2) - (BALL_SIZE / 2)) as f64,
        speed_x: 8.0,
        speed_y: 8.0,
        size: 7
    };

    let mut paddle = Paddle {
        y: ((height / 2) - (PADDLE_HEIGHT / 2)) as f64,
        height: PADDLE_HEIGHT,
        width: PADDLE_WIDTH
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
                Key::Special(ScanCode::UP) => {
                    paddle.y -= 40.0;
                }
                Key::Special(ScanCode::DOWN) => {
                    paddle.y += 40.0;
                }
                _ => {}
            }
        }

        // moving
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

        // clearing buffer
        buffer.clear();

        // rendering ball
        buffer.rectangle(
            ball.x as usize,
            ball.y as usize,
            ball.size,
            ball.size,
            WHITE,
            true,
        );

        // rendering paddle
        buffer.rectangle(
            width - PADDLE_WIDTH - PADDLE_DISTANCE_WALL,
            paddle.y as usize, paddle.width, paddle.height, WHITE, true);

        // draw buffer to screen
        let _ = buffer.blit(&mut gop);

        boot::stall(Duration::from_millis(10));
    }

    Ok(())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    game().unwrap();
    Status::SUCCESS
}
