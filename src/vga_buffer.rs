use core::fmt::{self, Write};
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// Размеры VGA-буфера
pub(crate) const BUFFER_HEIGHT: usize = 25;
pub(crate) const BUFFER_WIDTH: usize = 80;

// Адрес VGA-памяти
const VGA_BUFFER_ADDR: usize = 0xb8000;

// Цвета VGA (4 бита)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    Pink,
    Yellow,
    White,
}

/// Цветовая пара: foreground + background
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self((bg as u8) << 4 | (fg as u8))
    }
}

/// Символ в VGA-буфере
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

// Двумерный буфер VGA (25x80)
#[repr(transparent)]
pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Глобальный писатель в VGA
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}

// Структура для вывода текста в VGA
pub struct Writer {
    pub column_position: usize,
    pub color_code: ColorCode,
    pub buffer: &'static mut Buffer,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) },
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });

                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let c = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(c);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Выводит строку по центру последней строки экрана
    pub fn write_centered(&mut self, text: &str) {
        let len = text.len().min(BUFFER_WIDTH);
        let padding = (BUFFER_WIDTH - len) / 2;
        self.column_position = padding;
        self.write_string(text);
    }

    /// Выводит многострочный ASCII-арт, начиная с верхней строки, по центру
    pub fn write_ascii_art(&mut self, art: &str) {
        for line in art.lines() {
            let len = line.len().min(BUFFER_WIDTH);
            let padding = (BUFFER_WIDTH - len) / 2;
            self.column_position = padding;
            self.write_string(line);
            self.new_line();
        }
    }

    pub fn draw_char(&self, x: usize, y: usize, ch: char) {
        use crate::vga_buffer::WRITER;
        use crate::vga_buffer::ScreenChar;
        use volatile::Volatile;

        let mut writer = WRITER.lock();
        let color = writer.color_code;
        let buffer = &mut writer.buffer;


        if x < crate::vga_buffer::BUFFER_WIDTH && y < crate::vga_buffer::BUFFER_HEIGHT {
            buffer.chars[y][x].write(ScreenChar {
                ascii_character: ch as u8,
                color_code: color,
            });
        }
    }


}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Макрос `print!`, выводит в VGA
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::vga_buffer::_print(format_args!($($arg)*));
    };
}

/// Макрос `println!`, выводит в VGA с переносом
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

/// Внутренняя функция, используется макросами
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}
