//! Arkanoid in VGA Text Mode for Platinum OS

use crate::vga_buffer::{WRITER, BUFFER_WIDTH, BUFFER_HEIGHT, ScreenChar, ColorCode, Color};
use core::fmt::Write;

const PADDLE_WIDTH: usize = 7;
const BLOCK_ROWS: usize = 5;
const BLOCK_COLS: usize = 20;
const BLOCK_WIDTH: usize = 3;

struct BufferWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BufferWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        BufferWriter { buf, pos: 0 }
    }

    fn len(&self) -> usize {
        self.pos
    }
}

impl<'a> core::fmt::Write for BufferWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let len = core::cmp::min(self.buf.len() - self.pos, bytes.len());
        self.buf[self.pos..self.pos + len].copy_from_slice(&bytes[..len]);
        self.pos += len;
        Ok(())
    }
}

pub struct Arkanoid {
    ball_x: usize,
    ball_y: usize,
    ball_dx: isize,
    ball_dy: isize,
    paddle_x: usize,
    blocks: [[bool; BLOCK_COLS]; BLOCK_ROWS],
    score: usize,
    game_over: bool,
    started: bool,
    tick_counter: usize,
}

impl Arkanoid {
    pub fn new() -> Self {
        let mut blocks = [[false; BLOCK_COLS]; BLOCK_ROWS];
        for row in 0..BLOCK_ROWS {
            for col in 0..BLOCK_COLS {
                blocks[row][col] = true;
            }
        }

        Self {
            ball_x: BUFFER_WIDTH / 2,
            ball_y: BUFFER_HEIGHT - 5,
            ball_dx: 1,
            ball_dy: -1,
            paddle_x: BUFFER_WIDTH / 2 - PADDLE_WIDTH / 2,
            blocks,
            score: 0,
            game_over: false,
            started: false,
            tick_counter: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.game_over {
            self.draw_game_over();
            return;
        }

        self.clear_ball();
        self.clear_paddle();

        if self.started {
            self.tick_counter += 1;

            if self.tick_counter % 35 == 0 {
                self.ball_x = self.ball_x.saturating_add_signed(self.ball_dx);
                self.ball_y = self.ball_y.saturating_add_signed(self.ball_dy);

                if self.ball_x == 0 || self.ball_x >= BUFFER_WIDTH - 1 {
                    self.ball_dx *= -1;
                }

                if self.ball_y == 0 {
                    self.ball_dy *= -1;
                }

                if self.ball_y >= BUFFER_HEIGHT - 1 {
                    self.game_over = true;
                    return;
                } else if self.ball_y == BUFFER_HEIGHT - 2 {
                    if self.ball_x >= self.paddle_x && self.ball_x < self.paddle_x + PADDLE_WIDTH {
                        self.ball_dy *= -1;
                    }
                }

                let block_row = self.ball_y / 2;
                let block_col = self.ball_x / BLOCK_WIDTH;
                if block_row < BLOCK_ROWS && block_col < BLOCK_COLS {
                    if self.blocks[block_row][block_col] {
                        self.blocks[block_row][block_col] = false;
                        self.ball_dy *= -1;
                        self.score += 1;
                    }
                }
            }
        } else {
            self.ball_x = self.paddle_x + PADDLE_WIDTH / 2;
            self.ball_y = BUFFER_HEIGHT - 2;
        }

        self.draw_blocks();
        self.draw_paddle();
        self.draw_ball();
        self.draw_score();
    }

    pub fn move_left(&mut self) {
        if self.game_over {
            return;
        }
        self.clear_paddle();
        if self.paddle_x > 1 {
            self.paddle_x -= 1;
        }
        self.draw_paddle();
    }

    pub fn move_right(&mut self) {
        if self.game_over {
            return;
        }
        self.clear_paddle();
        if self.paddle_x < BUFFER_WIDTH - PADDLE_WIDTH - 1 {
            self.paddle_x += 1;
        }
        self.draw_paddle();
    }

    pub fn start_ball(&mut self) {
        self.started = true;
    }

    pub fn restart(&mut self) {
        *self = Arkanoid::new();
        self.draw_restart_message();
    }

    fn draw_ball(&self) {
        if self.ball_x < BUFFER_WIDTH && self.ball_y < BUFFER_HEIGHT {
            self.draw_char_colored(self.ball_x, self.ball_y, 'o', ColorCode::new(Color::White, Color::Black));
        }
    }

    fn clear_ball(&self) {
        self.draw_char_colored(self.ball_x, self.ball_y, ' ', ColorCode::new(Color::Black, Color::Black));
    }

    fn draw_paddle(&self) {
        for i in 0..PADDLE_WIDTH {
            self.draw_char_colored(self.paddle_x + i, BUFFER_HEIGHT - 1, '=', ColorCode::new(Color::Green, Color::Black));
        }
    }

    fn clear_paddle(&self) {
        for i in 0..PADDLE_WIDTH {
            self.draw_char_colored(self.paddle_x + i, BUFFER_HEIGHT - 1, ' ', ColorCode::new(Color::Black, Color::Black));
        }
    }

    fn draw_blocks(&self) {
        for row in 0..BLOCK_ROWS {
            let color = match row {
                0 => Color::Red,
                1 => Color::Magenta,
                2 => Color::Brown,
                3 => Color::Cyan,
                _ => Color::LightGray,
            };
            let code = ColorCode::new(color, Color::Black);

            for col in 0..BLOCK_COLS {
                let x = col * BLOCK_WIDTH;
                let y = row * 2;
                let ch = if self.blocks[row][col] { '#' } else { ' ' };
                self.draw_char_colored(x, y, ch, code);
                self.draw_char_colored(x + 1, y, ch, code);
            }
        }
    }

    fn draw_score(&self) {
        let mut writer = WRITER.lock();
        let y = BUFFER_HEIGHT - 3;
        let color = ColorCode::new(Color::Pink, Color::Black);

        let mut buffer = [0u8; 80];
        let mut buf_writer = BufferWriter::new(&mut buffer);
        write!(buf_writer, "Score: {:3} | Platinum-tan cheering you on~!", self.score).ok();

        let len = buf_writer.len();
        for (i, &byte) in buffer[..len].iter().enumerate() {
            if i < BUFFER_WIDTH {
                writer.buffer.chars[y][i].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color,
                });
            }
        }
    }

    fn draw_game_over(&self) {
        let mut writer = WRITER.lock();
        let y = BUFFER_HEIGHT - 3;
        let color = ColorCode::new(Color::LightRed, Color::Black);

        let mut buffer = [0u8; 80];
        let mut buf_writer = BufferWriter::new(&mut buffer);
        write!(buf_writer, "\u{1F494} Game Over, senpai! Score: {}", self.score).ok();

        let len = buf_writer.len();
        for (i, &byte) in buffer[..len].iter().enumerate() {
            if i < BUFFER_WIDTH {
                writer.buffer.chars[y][i].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color,
                });
            }
        }
    }

    fn draw_restart_message(&self) {
        let mut writer = WRITER.lock();
        let y = BUFFER_HEIGHT - 4;
        let color = ColorCode::new(Color::Cyan, Color::Black);

        let mut buffer = [0u8; 80];
        let mut buf_writer = BufferWriter::new(&mut buffer);
        write!(buf_writer, "Platinum-tan: Ready! Press SPACE again, senpai~").ok();

        let len = buf_writer.len();
        for (i, &byte) in buffer[..len].iter().enumerate() {
            if i < BUFFER_WIDTH {
                writer.buffer.chars[y][i].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color,
                });
            }
        }
    }

    fn draw_char_colored(&self, x: usize, y: usize, ch: char, color: ColorCode) {
        let mut writer = WRITER.lock();
        if x < BUFFER_WIDTH && y < BUFFER_HEIGHT {
            writer.buffer.chars[y][x].write(ScreenChar {
                ascii_character: ch as u8,
                color_code: color,
            });
        }
    }
}
