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

mod font {
    type Font = [u16; 16];
    pub const A: Font = [
        0x00, 0x18, 0x18, 0x18, 0x18, 0x24, 0x24, 0x24, 0x24, 0x7e, 0x42, 0x42, 0x42, 0xe7, 0x00,
        0x00,
    ];
}

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

    vga::putfont8(&sinfo, 10, 10, vga::Color::Black, &font::A);

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
