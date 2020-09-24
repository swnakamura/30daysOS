#![no_std]
#![no_main]
#![feature(asm)]

extern crate rlibc;

use core::fmt::Write;
use core::panic::PanicInfo;

mod vga_text;

fn hlt() {
    unsafe {
        asm!("HLT");
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello, world{}", "!");
    loop {
        hlt();
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
