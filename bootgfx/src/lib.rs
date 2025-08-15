#![no_std]

#[derive(Clone, Copy, Debug)]
pub struct FrameBuffer {
    mode: FrameBufferMode,
    base_ptr: *const u32,
    x_pixels: usize,
    y_pixels: usize,
    scanline_pixels: usize,
}

impl FrameBuffer {
    pub fn new(mode: FrameBufferMode, base_ptr: *const u32, x_pixels: usize, y_pixels: usize, scanline_pixels: usize) -> Self {
        Self { mode: mode, base_ptr: base_ptr, x_pixels: x_pixels, y_pixels: y_pixels, scanline_pixels: scanline_pixels }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameBufferMode {
    RGB,
    BGR,
    Unknown,
}
