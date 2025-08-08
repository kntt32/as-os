#![no_main]
#![no_std]

#[allow(unused)]
mod efi;

use core::panic::PanicInfo;
use efi::{EfiStatus, EFI_STATUS_SUCCESS, EfiSystemTable, EfiHandle};

#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(image_handle: EfiHandle, system_table: *mut EfiSystemTable) -> EfiStatus {
    let system_table_mut = unsafe { system_table.as_mut().expect("expected non-null pointer") };
    let con_out = system_table_mut.con_out();
    con_out.output_string("Hello, World!");
    loop {}
}

/*
#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    println!("Hello!");
    loop {}
}
*/
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
