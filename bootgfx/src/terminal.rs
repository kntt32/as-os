use super::*;
use core::ops::Deref;
use core::ops::DerefMut;

#[derive(Debug)]
pub struct Terminal {
    frame_buffer: FrameBuffer,
    cursor_x: usize,
    cursor_y: usize,
    width: usize,
    height: usize,
    buffer: [u8; Self::BUFF_WIDTH_MAX * Self::BUFF_HEIGHT_MAX],
}

impl Terminal {
    pub const BUFF_WIDTH_MAX: usize = 256;
    pub const BUFF_HEIGHT_MAX: usize = 128;
    pub const BACKGROUND: Color = Color::new(0x0D, 0x1B, 0x2A);
    pub const FOREGROUND: Color = Color::new(0xE0, 0xFB, 0xFC);
    pub const CURSOR: Color = Color::new(0x00, 0xFF, 0xFF);

    pub fn new(frame_buffer: FrameBuffer) -> Self {
        let width = frame_buffer.width() / 8;
        let height = frame_buffer.height() / 16;

        Self {
            frame_buffer: frame_buffer,
            cursor_x: 0,
            cursor_y: 0,
            width: width,
            height: height,
            buffer: [0u8; Self::BUFF_WIDTH_MAX * Self::BUFF_HEIGHT_MAX],
        }
    }

    pub fn clean(&mut self) {
        self.buffer[.. self.width * self.height].fill(0);
        self.cursor_x = 0;
        self.cursor_y = 0;

        self.flush();
    }

    pub fn write_ascii(&mut self, ascii: u8) {
        self.clean_cursor();

        match ascii {
            b'\n' => self.new_line(),
            b'\r' => self.cursor_x = 0,
            _ => {
        self.buffer[self.cursor_x + self.cursor_y * self.width] = ascii;
        self.frame_buffer.draw_font(
            ascii,
            self.cursor_x * 8,
            self.cursor_y * 16,
            Self::FOREGROUND,
            Self::BACKGROUND,
        );
        self.seek_cursor();
            },
        }
        
        self.draw_cursor();
    }

    pub fn write(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_ascii(byte);
        }
    }

    fn seek_cursor(&mut self) {
        assert!(self.cursor_x < self.width);
        assert!(self.cursor_y < self.height);

        self.cursor_x += 1;

        if self.width == self.cursor_x {
            self.new_line();
        }
    }

    fn new_line(&mut self) {
        assert!(self.cursor_y < self.height);

        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y == self.height {
            self.scroll();
        }
    }

    fn scroll(&mut self) {
        if 0 < self.cursor_y {
            self.cursor_y -= 1;
        }

        unsafe {
            let dst_ptr: *mut u8 = self.buffer.as_mut_ptr();
            let src_ptr: *const u8 = dst_ptr.add(self.width);
            let copy_len = self.width * (self.height - 1);
            src_ptr.copy_to(dst_ptr, copy_len);
        }

        self.buffer[self.width * (self.height - 1)..].fill(0);

        self.flush();
    }

    fn draw_cursor(&mut self) {
        self.frame_buffer.draw_rect(
            self.cursor_x * 8 + 1,
            self.cursor_y * 16 + 14,
            6,
            2,
            Self::CURSOR,
        );
    }

    fn clean_cursor(&mut self) {
        self.draw_at(self.cursor_x, self.cursor_y);
    }

    fn draw_at(&mut self, x: usize, y: usize) {
        let ascii = self.buffer[x + y * self.width];
        if ascii != 0 {
            self.frame_buffer
                .draw_font(ascii, x * 8, y * 16, Self::FOREGROUND, Self::BACKGROUND);
        } else {
            self.frame_buffer
                .draw_rect(x * 8, y * 16, 8, 16, Self::BACKGROUND);
        }
    }

    pub fn flush(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.draw_at(x, y);
            }
        }

        self.draw_cursor();
    }
}

impl Deref for Terminal {
    type Target = FrameBuffer;

    fn deref(&self) -> &FrameBuffer {
        &self.frame_buffer
    }
}

impl DerefMut for Terminal {
    fn deref_mut(&mut self) -> &mut FrameBuffer {
        &mut self.frame_buffer
    }
}
