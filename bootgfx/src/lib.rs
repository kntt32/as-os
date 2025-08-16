#![no_std]
pub mod font;

use core::ops::Index;
use core::ops::IndexMut;
use core::slice;
use font::BitmapFont;

#[derive(Clone, Copy, Debug)]
pub struct FrameBuffer {
    mode: FrameBufferMode,
    base_ptr: *mut u32,
    x_pixels: usize,
    y_pixels: usize,
    scanline_pixels: usize,
}

impl FrameBuffer {
    pub fn new(
        mode: FrameBufferMode,
        base_ptr: *mut u32,
        x_pixels: usize,
        y_pixels: usize,
        scanline_pixels: usize,
    ) -> Self {
        Self {
            mode: mode,
            base_ptr: base_ptr,
            x_pixels: x_pixels,
            y_pixels: y_pixels,
            scanline_pixels: scanline_pixels,
        }
    }

    pub fn draw_rect(
        &mut self,
        mut x: usize,
        mut y: usize,
        mut width: usize,
        mut height: usize,
        color: Color,
    ) {
        if self.mode == FrameBufferMode::RGB || self.mode == FrameBufferMode::BGR {
            if self.x_pixels <= x {
                x = 0;
                width = 0;
            }
            if self.y_pixels <= y {
                y = 0;
                height = 0;
            }
            if self.x_pixels <= x + width {
                width = self.x_pixels - x;
            }
            if self.y_pixels <= y + height {
                height = self.y_pixels - y;
            }

            let color_raw = color.as_raw(self.mode);
            for i in y..y + height {
                for k in x..x + width {
                    self[(k, i)] = color_raw;
                }
            }
        }
    }

    pub fn draw_font(&mut self, ascii: u8, x: usize, y: usize, color: Color, background: Color) {
        if x < self.x_pixels || y < self.y_pixels {
            let font = BitmapFont::from(ascii);
            let width = 8.min(self.x_pixels - x);
            let height = 16.min(self.y_pixels - y);

            let color_raw = color.as_raw(self.mode);
            let background_raw = background.as_raw(self.mode);

            for i in 0..height {
                for k in 0..width {
                    self[(x + k, y + i)] = if font.is_on(k, i) {
                        color_raw
                    } else {
                        background_raw
                    };
                }
            }
        }
    }

    pub fn draw_str(&mut self, s: &str, x: usize, y: usize, color: Color, background: Color) {
        let mut cursor_x = x;
        for ascii in s.bytes() {
            self.draw_font(ascii, cursor_x, y, color, background);
            cursor_x += 8;
        }
    }

    pub fn as_slice(&self) -> &[u32] {
        unsafe { slice::from_raw_parts(self.base_ptr, self.scanline_pixels * self.y_pixels) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u32] {
        unsafe { slice::from_raw_parts_mut(self.base_ptr, self.scanline_pixels * self.y_pixels) }
    }

    pub const fn width(&self) -> usize {
        self.x_pixels
    }

    pub const fn height(&self) -> usize {
        self.y_pixels
    }
}

impl Index<(usize, usize)> for FrameBuffer {
    type Output = u32;

    fn index(&self, index: (usize, usize)) -> &u32 {
        let (x, y) = index;
        if self.x_pixels <= x || self.y_pixels <= y {
            panic!("out of range");
        }
        unsafe { &*self.base_ptr.add(self.scanline_pixels * y + x) }
    }
}

impl IndexMut<(usize, usize)> for FrameBuffer {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut u32 {
        let (x, y) = index;
        if self.x_pixels <= x || self.y_pixels <= y {
            panic!("out of range");
        }
        unsafe { &mut *self.base_ptr.add(self.scanline_pixels * y + x) }
    }
}

impl Index<usize> for FrameBuffer {
    type Output = [u32];

    fn index(&self, index: usize) -> &[u32] {
        if self.y_pixels <= index {
            panic!("out of range");
        }
        unsafe {
            slice::from_raw_parts(
                self.base_ptr.add(self.scanline_pixels * index),
                self.scanline_pixels,
            )
        }
    }
}

impl IndexMut<usize> for FrameBuffer {
    fn index_mut(&mut self, index: usize) -> &mut [u32] {
        if self.y_pixels <= index {
            panic!("out of range");
        }
        unsafe {
            slice::from_raw_parts_mut(
                self.base_ptr.add(self.scanline_pixels * index),
                self.scanline_pixels,
            )
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameBufferMode {
    RGB,
    BGR,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: red,
            green: green,
            blue: blue,
        }
    }

    pub fn as_rgb(&self) -> u32 {
        let red = self.red as u32;
        let green = self.green as u32;
        let blue = self.blue as u32;
        red | (green << 8) | (blue << 16)
    }

    pub fn as_bgr(&self) -> u32 {
        let blue = self.blue as u32;
        let green = self.green as u32;
        let red = self.red as u32;
        blue | (green << 8) | (red << 16)
    }

    pub fn as_raw(&self, mode: FrameBufferMode) -> u32 {
        match mode {
            FrameBufferMode::RGB => self.as_rgb(),
            FrameBufferMode::BGR => self.as_bgr(),
            FrameBufferMode::Unknown => 0,
        }
    }
}
