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
mod io_func;
mod vga;

mod font;

#[no_mangle]
#[start]
pub extern "C" fn haribote_os() -> ! {
    let screen = vga::Screen::new();
    screen.init();

    // let mcursor = vga::init_mouse_cursor8(&vga::Color::DarkGreen);

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
