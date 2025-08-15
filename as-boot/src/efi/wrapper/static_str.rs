use super::*;

#[derive(Clone, Copy, Debug)]
pub struct StaticStr<const C: usize> {
    buff: [u8; C],
    len: usize,
}

impl<const C: usize> Write for StaticStr<C> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if C < self.len + s.len() {
            return Err(fmt::Error);
        }

        unsafe {
            let buff_end_ptr = (self.buff.as_mut_ptr()).add(self.len);
            s.as_ptr().copy_to(buff_end_ptr, s.len());
        }
        self.len += s.len();

        Ok(())
    }
}

impl<const C: usize> Deref for StaticStr<C> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        str::from_utf8(&self.buff[..self.len]).expect("expected utf8")
    }
}

impl<const C: usize> DerefMut for StaticStr<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        str::from_utf8_mut(&mut self.buff[..self.len]).expect("expected utf8")
    }
}

impl<const C: usize> StaticStr<C> {
    pub fn new() -> Self {
        Self {
            buff: [0u8; C],
            len: 0,
        }
    }

    pub fn from(s: &str) -> Self {
        let mut static_str = Self::new();

        if C < s.len() {
            panic!("out of range");
        }
        unsafe {
            s.as_ptr().copy_to(static_str.buff.as_mut_ptr(), s.len());
        }
        static_str.len = s.len();

        static_str
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut buff = crate::efi::wrapper::static_str::StaticStr::<1024>::new();
        write!(&mut buff, $($arg)*).unwrap();
        write!(&mut buff, "\n\r").unwrap();
        let _ = crate::wrapper::stdout(&*buff);
    }};
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut buff = crate::efi::wrapper::static_str::StaticStr::<1024>::new();
        write!(&mut buff, $($arg)*).unwrap();
        let _ = crate::wrapper::stdout(&*buff);
    }};
}

#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut buff = crate::efi::wrapper::static_str::StaticStr::<1024>::new();
        write!(&mut buff, $($arg)*).unwrap();
        buff
    }}
}
