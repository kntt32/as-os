#![no_main]
#![no_std]

use core::panic::PanicInfo;
use uefi::prelude::*;
use uefi::println;

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();
    println!("Hello!");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
