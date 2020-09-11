#![no_std]
#![feature(asm)]
#![feature(start)]

use core::panic::PanicInfo;

fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

fn show_white(i: u32) {
    // 白色なので15
    let a: u8 = 15;
    // 生ポインタを使って、15を代入
    let ptr = unsafe { &mut *(i as *mut u8) };
    *ptr = a
}

fn show_color(i: u32) {
    let ptr = unsafe { &mut *(i as *mut u32) };
    *ptr = i & 0x0f;
}

#[no_mangle]
#[start]
pub extern "C" fn haribote_os() -> ! {
    // 本にある通り、0xa0000から0xaffffまで描画
    for i in 0xa0000..0xaffff {
        show_color(i);
    }
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
