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
    xsize: u32,
    color: Color,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            unsafe {
                *vga_pointer.offset((y * xsize + x) as isize) = color as u8;
            }
        }
    }
}
