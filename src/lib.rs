#![no_std]
#![feature(asm)]
#![feature(start)]

use core::panic::PanicInfo;

use core::fmt::Write;

mod dsc_tbl;
mod font;
mod io_func;
mod vga;

mod pic {
    const PIC0_ICW1: u32 = 0x0020;
    const PIC0_OCW2: u32 = 0x0020;
    const PIC0_IMR: u32 = 0x0021;
    const PIC0_ICW2: u32 = 0x0021;
    const PIC0_ICW3: u32 = 0x0021;
    const PIC0_ICW4: u32 = 0x0021;
    const PIC1_ICW1: u32 = 0x00a0;
    const PIC1_OCW2: u32 = 0x00a0;
    const PIC1_IMR: u32 = 0x00a1;
    const PIC1_ICW2: u32 = 0x00a1;
    const PIC1_ICW3: u32 = 0x00a1;
    const PIC1_ICW4: u32 = 0x00a1;

    pub fn init() {
        use super::io_func::out8;
        out8(PIC0_IMR, 0xff); /* 全ての割り込みを受け付けない */
        out8(PIC1_IMR, 0xff); /* 全ての割り込みを受け付けない */

        out8(PIC0_ICW1, 0x11); /* エッジトリガモード */
        out8(PIC0_ICW2, 0x20); /* IRQ0-7は、INT20-27で受ける */
        out8(PIC0_ICW3, 1 << 2); /* PIC1はIRQ2にて接続 */
        out8(PIC0_ICW4, 0x01); /* ノンバッファモード */

        out8(PIC1_ICW1, 0x11); /* エッジトリガモード */
        out8(PIC1_ICW2, 0x28); /* IRQ8-15は、INT28-2fで受ける */
        out8(PIC1_ICW3, 2); /* PIC1はIRQ2にて接続 */
        out8(PIC1_ICW4, 0x01); /* ノンバッファモード */

        out8(PIC0_IMR, 0xfb); /* 11111011 PIC1以外は全て禁止 */
        out8(PIC1_IMR, 0xff); /* 11111111 全ての割り込みを受け付けない */
    }
}

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
