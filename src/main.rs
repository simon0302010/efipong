#![no_main]
#![no_std]

extern crate alloc;
extern crate libm;
extern crate num_traits;

mod buffer;
mod misc;
mod rand;

use core::time::Duration;

use alloc::vec;
use num_traits::float::FloatCore;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltPixel, GraphicsOutput};
use uefi::proto::console::text::{Key, ScanCode};
use uefi::{boot, Result};

use crate::buffer::Buffer;
use crate::misc::{rectangles_overlapping, Rectangle};
use crate::rand::Rng;
use core::f64;
use libm::{cos, sin, sqrt};

#[derive(Clone, Copy)]
struct Ball {
    x: f64,
    y: f64,
    speed_x: f64,
    speed_y: f64,
    size: usize,
}

#[derive(Clone, Copy)]
struct Paddle {
    x: f64,
    y: f64,
    height: usize,
    width: usize,
    score: usize,
}

const BALL_SIZE: usize = 7;
const BALL_START_SPEED: f64 = 3.0;
const MAX_BALL_SPEED: f64 = 7.0;
const PADDLE_HEIGHT: usize = 80;
const PADDLE_WIDTH: usize = 6;
const PADDLE_SPEED: f64 = 40.0;
const PADDLE_DISTANCE_WALL: usize = 30;
const BOUNCE_ANGLE: f64 = 5.0 * core::f64::consts::PI / 12.0;

const WHITE: BltPixel = BltPixel::new(255, 255, 255);

fn game() -> Result {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let (width, height) = gop.current_mode_info().resolution();
    let mut buffer = Buffer::new(width, height);

    let mut running = true;

    // rng for ball direction after scoring
    let mut rng = Rng::new();

    let mut ball = Ball {
        x: ((width / 2) - (BALL_SIZE / 2)) as f64,
        y: ((height / 2) - (BALL_SIZE / 2)) as f64,
        speed_x: if rng.random_bool(0.5) {
            BALL_START_SPEED
        } else {
            -BALL_START_SPEED
        },
        speed_y: 0.0,
        size: 7,
    };

    let mut paddle_r = Paddle {
        x: (width - PADDLE_WIDTH - PADDLE_DISTANCE_WALL) as f64,
        y: ((height / 2) - (PADDLE_HEIGHT / 2)) as f64,
        height: PADDLE_HEIGHT,
        width: PADDLE_WIDTH,
        score: 0,
    };

    let mut paddle_l = Paddle {
        x: PADDLE_DISTANCE_WALL as f64,
        y: ((height / 2) - (PADDLE_HEIGHT / 2)) as f64,
        height: PADDLE_HEIGHT,
        width: PADDLE_WIDTH,
        score: 0,
    };

    while running {
        while let Ok(Some(key)) = system::with_stdin(|stdin| stdin.read_key()) {
            match key {
                Key::Printable(c) => {
                    match char::try_from(c).ok().map(|ch| ch.to_ascii_lowercase()) {
                        Some('q') => {
                            running = false;
                        }
                        Some('w') => {
                            paddle_l.y = (paddle_l.y - PADDLE_SPEED)
                                .clamp(0.0, (height - PADDLE_HEIGHT) as f64);
                        }
                        Some('s') => {
                            paddle_l.y = (paddle_l.y + PADDLE_SPEED)
                                .clamp(0.0, (height - PADDLE_HEIGHT) as f64);
                        }
                        _ => {}
                    }
                }
                Key::Special(ScanCode::UP) => {
                    paddle_r.y =
                        (paddle_r.y - PADDLE_SPEED).clamp(0.0, (height - PADDLE_HEIGHT) as f64);
                }
                Key::Special(ScanCode::DOWN) => {
                    paddle_r.y =
                        (paddle_r.y + PADDLE_SPEED).clamp(0.0, (height - PADDLE_HEIGHT) as f64);
                }
                _ => {}
            }
        }

        // moving
        ball.x += ball.speed_x;
        ball.y += ball.speed_y;

        if ball.x >= width as f64 - ball.size as f64 {
            ball.x = (width / 2 - ball.size / 2) as f64;
            ball.y = paddle_r.y + (paddle_r.height / 2) as f64 - (ball.size / 2) as f64;
            ball.speed_x = BALL_START_SPEED;
            ball.speed_y = rng.random_range(-BALL_START_SPEED, BALL_START_SPEED);
            paddle_l.score += 1;
        } else if ball.x <= 0.0 {
            ball.x = (width / 2 - ball.size / 2) as f64;
            ball.y = paddle_l.y + (paddle_l.height / 2) as f64 - (ball.size / 2) as f64;
            ball.speed_x = -BALL_START_SPEED;
            ball.speed_y = rng.random_range(-BALL_START_SPEED, BALL_START_SPEED);
            paddle_r.score += 1;
        }

        // Vertical collision detection and response
        if ball.y >= height as f64 - ball.size as f64 {
            ball.y = height as f64 - ball.size as f64;
            ball.speed_y = -ball.speed_y;
        } else if ball.y <= 0.0 {
            ball.y = 0.0;
            ball.speed_y = -ball.speed_y;
        }

        // handling ball paddle collisions
        handle_paddle_hit(&mut ball, &paddle_r);
        handle_paddle_hit(&mut ball, &paddle_l);

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

        // rendering paddles
        for paddle in vec![paddle_l, paddle_r] {
            buffer.rectangle(
                paddle.x as usize,
                paddle.y as usize,
                paddle.width,
                paddle.height,
                WHITE,
                true,
            );
        }

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

fn handle_paddle_hit(ball: &mut Ball, paddle: &Paddle) {
    if rectangles_overlapping(
        Rectangle {
            x: ball.x as usize,
            y: ball.y as usize,
            width: ball.size,
            height: ball.size,
        },
        Rectangle {
            x: paddle.x as usize,
            y: paddle.y as usize,
            width: paddle.width,
            height: paddle.height,
        },
    ) {
        // inversing direction
        ball.speed_x = -ball.speed_x;
        // snap to paddle
        let hit: Hit;

        if ball.x > paddle.x {
            // left paddle
            ball.x = paddle.x + paddle.width as f64;
            hit = Hit::Left;
        } else {
            // right paddle
            ball.x = paddle.x - ball.size as f64;
            hit = Hit::Right;
        }

        let paddle_center = paddle.y + (paddle.height as f64 / 2.0);
        let ball_center = ball.y + (ball.size as f64 / 2.0);
        let hit_y = ((ball_center - paddle_center) / (paddle.height as f64 / 2.0)).clamp(-1.0, 1.0);

        // calculate new direction and speed
        // Increase ball speed by 5% each hit, cap at 7.0
        let mut ball_speed = sqrt(ball.speed_x.powi(2) + ball.speed_y.powi(2));
        ball_speed = (ball_speed * 1.1).min(MAX_BALL_SPEED);

        match hit {
            Hit::Right => {
                ball.speed_x = -ball_speed * cos(BOUNCE_ANGLE * hit_y);
            }
            Hit::Left => {
                ball.speed_x = -(-ball_speed * cos(BOUNCE_ANGLE * hit_y));
            }
        }
        ball.speed_y = ball_speed * sin(BOUNCE_ANGLE * hit_y);
    }
}

enum Hit {
    Left,
    Right,
}
