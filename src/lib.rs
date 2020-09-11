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

    let vga_pointer = 0xa0000 as *mut u8;
    vga::boxfill8(vga_pointer, 320, vga::Color::LightRed, 20, 20, 120, 120);
    vga::boxfill8(vga_pointer, 320, vga::Color::LightGreen, 70, 70, 170, 170);
    vga::boxfill8(vga_pointer, 320, vga::Color::LightBlue, 120, 120, 220, 220);
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
