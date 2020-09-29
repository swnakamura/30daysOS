#![no_std]
#![no_main]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(haribote2::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate rlibc;

use core::panic::PanicInfo;

use haribote2::{print, println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    haribote2::init();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    haribote2::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    haribote2::test_panic_handler(info)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    haribote2::hlt_loop()
}
