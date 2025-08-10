#![no_main]
#![no_std]

use core::panic::PanicInfo;
use core::arch::asm;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
