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
    let newline_in_string = "ABC\n123";
    let haribote_os = "\"I love Leafeon the best. How about you\"? She answered \"Mesprit\".";
    let japanese_sentence = "日本語";
    let x = 42;

    let mut string_writer = vga::ScreenStringWriter::new(&sinfo, 8, 10, vga::Color::White);

    write!(&mut string_writer, "{}", abc123).unwrap();
    string_writer.newline();
    write!(&mut string_writer, "{}", newline_in_string).unwrap();
    string_writer.newline();
    write!(&mut string_writer, "{}", haribote_os).unwrap();
    string_writer.newline();
    write!(&mut string_writer, "{}", japanese_sentence).unwrap();
    string_writer.newline();
    write!(&mut string_writer, "x: {}", x).unwrap();

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
