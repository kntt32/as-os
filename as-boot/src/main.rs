#![no_main]
#![no_std]

#[allow(unused)]
mod efi;

use bootgfx::Color;
use bootgfx::terminal::Terminal;
use efi::EFI_STATUS_SUCCESS;
use efi::EfiHandle;
use efi::EfiStatus;
use efi::EfiSystemTable;
use efi::wrapper;
use elf::Elf64;
use wrapper::File;
use wrapper::PageBox;
use wrapper::stdclean;

pub fn main() -> Result<(), &'static str> {
    println!("Hello, World!");
    println!("as-boot alpha version");

    let kernel = Kernel::new("kernel.elf")?;
    println!("KERNEL: {:?}", kernel);

    let mut frame_buffer = wrapper::get_frame_buffer()?;
    println!("FRAME_BUFFER: {:?}", frame_buffer);

    frame_buffer.draw_rect(10, 30, 200, 100, Color::new(0x00, 0xff, 0xff));
    frame_buffer.draw_str(
        "Hello, World!!!!!",
        0,
        0,
        Color::new(0x00, 0x00, 0xff),
        Color::new(0x00, 0xff, 0x00),
    );
    let mut terminal = Terminal::new(frame_buffer);
    for _ in 0..500 {
        terminal.write("Hello, Terminal!");
    }
    terminal.clean();
    terminal.write("Hello, as-boot!\n");
    loop {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Kernel {
    kernel_buff_addr: usize,
    kernel_virtual_addr: usize,
    entry_point: usize,
}

impl Kernel {
    pub fn new(path: &str) -> Result<Kernel, &'static str> {
        let (kernel_tmp_pagebox, kernel_tmp_buff_size) = Self::load_kernel_to_tmp_buffer(path)?;
        let kernel_temp_buff: &[u8] = &kernel_tmp_pagebox[0..kernel_tmp_buff_size];
        Self::expand_kernel(kernel_temp_buff)
    }

    fn load_kernel_to_tmp_buffer(path: &str) -> Result<(PageBox, usize), &'static str> {
        let mut file = File::new(path)?;
        let file_size = file.size();
        let mut page_box = PageBox::new_from_bytes(file_size);
        let load_size = file.read(&mut *page_box)?;
        assert_eq!(file_size, load_size);

        Ok((page_box, load_size))
    }

    fn expand_kernel(kernel_temp_buff: &[u8]) -> Result<Kernel, &'static str> {
        let elf64 = Elf64::new(kernel_temp_buff)?;
        let expand_info = elf64.expand_info()?;
        let expand_size = (expand_info.upper_addr - expand_info.lower_addr) as usize;

        let kernel_virtual_addr = expand_info.lower_addr as usize;
        let entry_point = elf64.entry()? as usize;

        let kernel_buff_pagebox = PageBox::new_from_bytes(expand_size);
        let kernel_buff: &mut [u8] = kernel_buff_pagebox.leak();
        elf64.expand(kernel_buff)?;
        let kernel_buff_addr = kernel_buff.as_ptr().addr();

        Ok(Kernel {
            kernel_buff_addr: kernel_buff_addr,
            kernel_virtual_addr: kernel_virtual_addr,
            entry_point: entry_point,
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "efiapi" fn efi_main(
    _image_handle: EfiHandle,
    system_table: *const EfiSystemTable,
) -> EfiStatus {
    unsafe {
        wrapper::init(system_table);
    }

    stdclean().expect("failed to clear screen");

    if let Err(msg) = main() {
        panic!("ERROR: {}", msg);
    } else {
        EFI_STATUS_SUCCESS
    }
}
