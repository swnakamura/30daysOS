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

    let sinfo;
    unsafe {
        sinfo = vga::ScreenInfo {
            screenx: *(0x0ff4 as *const u16),
            screeny: *(0x0ff6 as *const u16),
            vram_pointer: *(0x0ff8 as *const *mut u8),
        };
    }

    vga::init_screen(sinfo);

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
