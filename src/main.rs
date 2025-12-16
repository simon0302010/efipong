#![no_main]
#![no_std]

extern crate alloc;

mod buffer;

use core::time::Duration;

use uefi::prelude::*;
use uefi::proto::console::gop::{BltPixel, GraphicsOutput};
use uefi::{boot, Result};

use crate::buffer::Buffer;

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: isize,
    pub y: isize,
}

fn game() -> Result {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let (width, height) = gop.current_mode_info().resolution();
    let mut buffer = Buffer::new(width, height);

    let rec_w = 50;
    let rec_h = 30;

    buffer.clear();
    buffer.rectangle((width / 2) - (rec_w / 2), (height / 2) - (rec_h / 2), rec_w, rec_h, BltPixel::new(255, 0, 0), false);
    let _ = buffer.blit(&mut gop);

    boot::stall(Duration::from_secs(10));

    Ok(())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    game().unwrap();
    Status::SUCCESS
}
