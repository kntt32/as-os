mod bitmap;

use core::convert::From;

#[derive(Clone, Copy, Debug)]
pub struct BitmapFont {
    bitmap: [u8; 16],
}

impl BitmapFont {
    pub const fn new(bitmap: [u8; 16]) -> Self {
        Self { bitmap: bitmap }
    }

    pub const fn is_on(&self, x: usize, y: usize) -> bool {
        if 8 <= x || 16 <= y {
            false
        } else {
            self.bitmap[y] & (0x80 >> x) != 0
        }
    }

    pub const fn is_off(&self, x: usize, y: usize) -> bool {
        !self.is_on(x, y)
    }
}

impl From<u8> for BitmapFont {
    fn from(mut value: u8) -> Self {
        if Self::FONT_TABLE.len() <= value as usize {
            value = 0;
        }
        Self::FONT_TABLE[value as usize]
    }
}
