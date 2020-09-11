#![no_std]
#![feature(asm)]
#![feature(start)]

use core::panic::PanicInfo;

fn hlt() {
    unsafe {
        asm!("HLT");
    }
}
mod io_func;
mod vga;

#[no_mangle]
#[start]
pub extern "C" fn haribote_os() -> ! {
    vga::init_palette();

    vga::draw_haribote_desktop();

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
