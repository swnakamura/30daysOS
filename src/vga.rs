use crate::io_func;

const basic_rgb_table: [[u32; 3]; 16] = [
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

/// Initialize palette with basic_rgb_table.
pub fn init_palette() {
    set_palette(0, 15, basic_rgb_table);
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
