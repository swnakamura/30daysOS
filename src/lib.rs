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

mod font;

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

    vga::init_screen(&sinfo);

    vga::putfont8(&sinfo, 8, 10, vga::Color::Black, &font::fontdata[65]);
    vga::putfont8(&sinfo, 16, 10, vga::Color::Black, &font::fontdata[66]);
    vga::putfont8(&sinfo, 24, 10, vga::Color::Black, &font::fontdata[67]);
    vga::putfont8(&sinfo, 40, 10, vga::Color::Black, &font::fontdata[49]);
    vga::putfont8(&sinfo, 48, 10, vga::Color::Black, &font::fontdata[50]);
    vga::putfont8(&sinfo, 56, 10, vga::Color::Black, &font::fontdata[51]);

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
