#![no_std]
#![feature(asm)]
#![feature(start)]

use core::panic::PanicInfo;

use core::fmt::Write;

fn hlt() {
    unsafe {
        asm!("HLT");
    }
}
mod font;
mod io_func;
mod vga;

#[no_mangle]
#[start]
pub extern "C" fn haribote_os() -> ! {
    let mut screen = vga::Screen::new();
    screen.init();

    loop {
        hlt()
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        hlt()
    }
}
