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

    draw_haribote_desktop();

    loop {
        hlt()
    }
}

fn draw_haribote_desktop() {
    use vga::Color::*;
    let (xsize, ysize, vram_pointer);
    unsafe {
        let binfo_screenx = 0x0ff4 as *const u16;
        let binfo_screeny = 0x0ff6 as *const u16;
        let binfo_vram = 0x0ff8 as *const *mut u8;
        xsize = *binfo_screenx;
        ysize = *binfo_screeny;
        vram_pointer = *binfo_vram;
    }

    let boxes_to_draw = [
        (DarkCyan, 0, 0, xsize - 1, ysize - 29),
        (LightGray, 0, ysize - 28, xsize - 1, ysize - 28),
        (White, 0, ysize - 27, xsize - 1, ysize - 27),
        (LightGray, 0, ysize - 26, xsize - 1, ysize - 1),
        (White, 3, ysize - 24, 59, ysize - 24),
        (White, 2, ysize - 24, 2, ysize - 4),
        (DarkGray, 3, ysize - 4, 59, ysize - 4),
        (DarkGray, 59, ysize - 23, 59, ysize - 5),
        (Black, 2, ysize - 3, 59, ysize - 3),
        (Black, 60, ysize - 24, 60, ysize - 3),
        (DarkGray, xsize - 47, ysize - 24, xsize - 4, ysize - 24),
        (DarkGray, xsize - 47, ysize - 23, xsize - 47, ysize - 4),
        (White, xsize - 47, ysize - 3, xsize - 4, ysize - 3),
        (White, xsize - 3, ysize - 24, xsize - 3, ysize - 3),
    ];

    for info in boxes_to_draw.iter() {
        vga::boxfill8(vram_pointer, xsize, info.0, info.1, info.2, info.3, info.4);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        hlt()
    }
}
