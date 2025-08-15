#![no_std]

use core::slice;

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
            let frame_buffer_slice = unsafe {
                slice::from_raw_parts_mut(self.base_ptr, self.scanline_pixels * self.y_pixels)
            };

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
                    frame_buffer_slice[i * self.scanline_pixels + k] = color_raw;
                }
            }
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
