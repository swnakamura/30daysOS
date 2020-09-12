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

    let abc123 = "ABC 123";
    let haribote_os = "I love Leafeon the best.How about you?";
    let japanese_sentence = "日本語";

    vga::putfonts8_ascii(&sinfo, 8, 10, vga::Color::White, abc123);
    vga::putfonts8_ascii(&sinfo, 9, 27, vga::Color::Black, haribote_os);
    vga::putfonts8_ascii(&sinfo, 8, 26, vga::Color::White, haribote_os);
    vga::putfonts8_ascii(&sinfo, 8, 42, vga::Color::White, japanese_sentence);

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
