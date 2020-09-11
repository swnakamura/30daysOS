use crate::io_func;

const BASIC_RGB_TABLE: [[u32; 3]; 16] = [
    [0x00, 0x00, 0x00], // 0:黒
    [0xff, 0x00, 0x00], // 1:明るい赤
    [0x00, 0xff, 0x00], // 2:明るい緑
    [0xff, 0xff, 0x00], // 3:明るい黄色
    [0x00, 0x00, 0xff], // 4:明るい青
    [0xff, 0x00, 0xff], // 5:明るい紫
    [0x00, 0xff, 0xff], // 6:明るい水色
    [0xff, 0xff, 0xff], // 7:白
    [0xc6, 0xc6, 0xc6], // 8:明るい灰色
    [0x84, 0x00, 0x00], // 9:暗い赤
    [0x00, 0x84, 0x00], // 10:暗い緑
    [0x84, 0x84, 0x00], // 11:暗い黄色
    [0x00, 0x00, 0x84], // 12:暗い青
    [0x84, 0x00, 0x84], // 13:暗い紫
    [0x00, 0x84, 0x84], // 14:暗い水色
    [0x84, 0x84, 0x84], // 15:暗い灰色
];

#[derive(Copy, Clone)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Color {
    Black = 0,
    LightRed = 1,
    LightGreen = 2,
    LightYellow = 3,
    LightBlue = 4,
    LightPurple = 5,
    LightCyan = 6,
    White = 7,
    LightGray = 8,
    DarkRed = 9,
    DarkGreen = 10,
    DarkYellow = 11,
    DarkBlue = 12,
    DarkPurple = 13,
    DarkCyan = 14,
    DarkGray = 15,
}

/// Initialize palette with basic_rgb_table.
pub fn init_palette() {
    set_palette(0, 15, BASIC_RGB_TABLE);
}

/// set palette with given rgb table, from start to end (inclusive).
fn set_palette(start: u32, end: u32, rgb: [[u32; 3]; 16]) {
    let eflags;
    eflags = io_func::load_eflags();
    io_func::cli();
    io_func::out8(0x03c8, start);
    for i in start..=end {
        let i = i as usize;
        io_func::out8(0x03c9, rgb[i][0] >> 2);
        io_func::out8(0x03c9, rgb[i][1] >> 2);
        io_func::out8(0x03c9, rgb[i][2] >> 2);
    }
    io_func::store_eflags(eflags);
    return;
}

/// (x0, y0) から (x1, y1) の箱を塗る
/// 教科書ではcharポインタを使っているので、色の代入はu8のサイズとわかり、なのでu8が使われていることに注目
pub fn boxfill8(
    vga_pointer: *mut u8,
    xsize: u16,
    color: Color,
    x0: u16,
    y0: u16,
    x1: u16,
    y1: u16,
) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            unsafe {
                *vga_pointer.offset((y * xsize + x) as isize) = color as u8;
            }
        }
    }
}

pub fn draw_haribote_desktop() {
    use Color::*;
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
        boxfill8(vram_pointer, xsize, info.0, info.1, info.2, info.3, info.4);
    }
}
