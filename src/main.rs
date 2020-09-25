#![no_std]
#![no_main]
#![feature(asm)]

extern crate rlibc;

use core::fmt::Write;
use core::panic::PanicInfo;

use haribote2::println;

fn hlt() {
    unsafe {
        asm!("HLT");
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    haribote2::init();

    x86_64::instructions::interrupts::int3();
    println!("It did not crash!");
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
