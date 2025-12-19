#![no_main]
#![no_std]

extern crate alloc;
extern crate libm;
extern crate num_traits;

mod buffer;
mod misc;
mod rand;

use core::arch::x86_64::_rdtsc;
use core::time::Duration;

use alloc::string::{String, ToString};
use alloc::{format, vec};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
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
const BALL_START_SPEED: f64 = 300.0;
const MAX_BALL_SPEED: f64 = 800.0;
const PADDLE_HEIGHT: usize = 80;
const PADDLE_WIDTH: usize = 6;
const PADDLE_SPEED: f64 = 40.0;
const PADDLE_DISTANCE_WALL: usize = 40;
const BOUNCE_ANGLE: f64 = 5.0 * core::f64::consts::PI / 12.0;

const WHITE: BltPixel = BltPixel::new(255, 255, 255);

fn game() -> Result {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let (width, height) = gop.current_mode_info().resolution();
    let mut buffer = Buffer::new(width, height);

    let mut running = true;
    let mut in_game = false;

    // rng for ball direction after scoring
    let mut rng = Rng::new();

    let mut ball = get_default_ball(&width, &height, &mut rng);

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

    // estimate cpu frequency
    let tsc_start = unsafe { _rdtsc() };
    boot::stall(Duration::from_millis(100));
    let tsc_end = unsafe { _rdtsc() };
    let ticks_per_second = (tsc_end - tsc_start) * 10; // 100ms * 10 = 1 second

    let mut last_tsc = unsafe { _rdtsc() };

    while running {
        // calculate delta time
        let current_tsc = unsafe { _rdtsc() };
        let delta_ticks = current_tsc.saturating_sub(last_tsc);
        last_tsc = current_tsc;
        let delta = (delta_ticks as f64) / (ticks_per_second as f64); // delta in seconds

        if in_game {
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

            // moving - scale by delta time (in seconds)
            ball.x += ball.speed_x * delta;
            ball.y += ball.speed_y * delta;

            // when a "goal" is scored
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

            // stop game at 11 points
            if paddle_l.score >= 11 || paddle_r.score >= 11 {
                in_game = false;
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
            handle_paddle_hit(&mut ball, &paddle_r, &width);
            handle_paddle_hit(&mut ball, &paddle_l, &width);

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

            let score_text = format!("{} | {}", paddle_l.score, paddle_r.score);
            let _ = Text::new(
                &score_text,
                Point::new(
                    ((width - (score_text.len() * 10)) / 2) as i32,
                    20,
                ),
                MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 255)),
            )
            .draw(&mut buffer);

            // draw buffer to screen
            let _ = buffer.blit(&mut gop);

            boot::stall(Duration::from_millis(10));
        } else {
            // restart if space is pressed.
            while let Ok(Some(key)) = system::with_stdin(|stdin| stdin.read_key()) {
                match key {
                    Key::Printable(c) => {
                        match char::try_from(c).ok().map(|ch| ch.to_ascii_lowercase()) {
                            Some('q') => {
                                running = false;
                            }
                            Some(' ') => {
                                in_game = true;
                                paddle_l.score = 0;
                                paddle_r.score = 0;
                                ball = get_default_ball(&width, &height, &mut rng);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            buffer.clear();

            let final_text: String;

            // decide which text to display based on scores
            if paddle_l.score >= 11 {
                final_text = format!(
                    "Left won by {} points. Press space to restart.",
                    paddle_l.score - paddle_r.score
                );
            } else if paddle_r.score >= 11 {
                final_text = format!(
                    "Right won by {} points. Press space to restart.",
                    paddle_r.score - paddle_l.score
                );
            } else {
                final_text = "Press space to start the game.".to_string();
            }

            // create text
            let _ = Text::new(
                &final_text,
                Point::new(
                    ((width - (final_text.len() * 10)) / 2) as i32,
                    ((height - 20) / 2) as i32,
                ),
                MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 255)),
            )
            .draw(&mut buffer);

            let _ = buffer.blit(&mut gop);
        }
    }

    Ok(())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    game().unwrap();
    Status::SUCCESS
}

fn handle_paddle_hit(ball: &mut Ball, paddle: &Paddle, width: &usize) {
    let is_left_paddle = paddle.x < *width as f64 / 2.0;
    
    let paddle_x = if is_left_paddle {
        0
    } else {
        paddle.x as usize
    };

    if rectangles_overlapping(
        Rectangle {
            x: ball.x as usize,
            y: ball.y as usize,
            width: ball.size,
            height: ball.size,
        },
        Rectangle {
            x: paddle_x,
            y: paddle.y as usize,
            width: paddle.width + PADDLE_DISTANCE_WALL,
            height: paddle.height,
        },
    ) {
        let hit = if is_left_paddle {
            Hit::Left
        } else {
            Hit::Right
        };

        // snap ball to paddle
        match hit {
            Hit::Left => {
                ball.x = paddle.x + paddle.width as f64;
            }
            Hit::Right => {
                ball.x = paddle.x - ball.size as f64;
            }
        }

        let paddle_center = paddle.y + (paddle.height as f64 / 2.0);
        let ball_center = ball.y + (ball.size as f64 / 2.0);
        let hit_y = ((ball_center - paddle_center) / (paddle.height as f64 / 2.0)).clamp(-1.0, 1.0);

        // calculate new direction and speed
        let mut ball_speed = sqrt(ball.speed_x.powi(2) + ball.speed_y.powi(2));
        ball_speed = (ball_speed * 1.1).min(MAX_BALL_SPEED);

        match hit {
            Hit::Right => {
                ball.speed_x = -ball_speed * cos(BOUNCE_ANGLE * hit_y);
            }
            Hit::Left => {
                ball.speed_x = ball_speed * cos(BOUNCE_ANGLE * hit_y);
            }
        }
        ball.speed_y = ball_speed * sin(BOUNCE_ANGLE * hit_y);
    }
}

fn get_default_ball(width: &usize, height: &usize, rng: &mut Rng) -> Ball {
    Ball {
        x: ((width / 2) - (BALL_SIZE / 2)) as f64,
        y: ((height / 2) - (BALL_SIZE / 2)) as f64,
        speed_x: if rng.random_bool(0.5) {
            BALL_START_SPEED
        } else {
            -BALL_START_SPEED
        },
        speed_y: 0.0,
        size: 7,
    }
}

enum Hit {
    Left,
    Right,
}
