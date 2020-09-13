#![no_std]
#![feature(asm)]
#![feature(start)]

use core::panic::PanicInfo;

use core::fmt::Write;

mod dsc_tbl;
mod font;
mod io_func;
mod vga;

fn hlt() {
    unsafe {
        asm!("HLT");
    }
}
#[no_mangle]
#[start]
pub extern "C" fn haribote_os() -> ! {
    let mut screen = vga::Screen::new();
    screen.init();

    dsc_tbl::init_gdtidt();

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
