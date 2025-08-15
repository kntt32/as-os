#![no_main]
#![no_std]

#[allow(unused)]
mod efi;

use efi::EFI_STATUS_ERROR;
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

    let mut file = File::new("kernel.elf")?;
    let file_size = file.size();
    let mut page_box = PageBox::new_from_bytes(file_size);
    let load_size = file.read(&mut *page_box)?;
    let kernel_temp_buff: &[u8] = &page_box[0..load_size];
    /*
    let Some(elf64) = Elf64::new() else {
        return Err("invalid kernel file found");
    };
    let Some() = elf64.expand_info()
    */
    println!("hello, println! {}, {}", 12345, file.size());

    loop {}
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
        let _ = wrapper::stdout("ERROR: ");
        let _ = wrapper::stdout(msg);
        EFI_STATUS_ERROR
    } else {
        EFI_STATUS_SUCCESS
    }
}
