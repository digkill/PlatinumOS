#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;
mod vga_buffer;
mod port_io;
mod arkanoid;

use arkanoid::Arkanoid;
use crate::vga_buffer::{BUFFER_WIDTH, ScreenChar, WRITER};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut game = Arkanoid::new();

    // Показать инструкцию до начала игры на строке y = 10
    {
        let mut writer = WRITER.lock();
        let text = "Platinum-tan: Press SPACE to start!";
        let y = 10;
        let color = writer.color_code;
        for (i, ch) in text.chars().enumerate() {
            if i < BUFFER_WIDTH {
                writer.buffer.chars[y][i].write(ScreenChar {
                    ascii_character: ch as u8,
                    color_code: color,
                });
            }
        }
    }

    loop {
        game.tick();

        let sc = unsafe { port_io::inb(0x60) };

        match sc {
            0x1E => game.move_left(),   // A
            0x20 => game.move_right(),  // D
            0x39 => game.start_ball(),  // SPACE
            0x13 => game.restart(),     // R
            _ => {}
        }

        for _ in 0..50_000 {
            unsafe { core::arch::asm!("nop") };
        }
    }
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
