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

    let mut running = true;

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

        buffer.clear();
        buffer.rectangle(
            (width / 2) - (rec_w / 2),
            (height / 2) - (rec_h / 2),
            rec_w,
            rec_h,
            BltPixel::new(255, 0, 0),
            false,
        );
        let _ = buffer.blit(&mut gop);

        boot::stall(Duration::from_millis(16));
    }

    Ok(())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    game().unwrap();
    Status::SUCCESS
}
