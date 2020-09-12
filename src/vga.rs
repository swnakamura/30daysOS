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

/// Initialize palette with ScreenStringWriterble.
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
pub fn boxfill8(sinfo: &ScreenInfo, color: Color, x0: u16, y0: u16, x1: u16, y1: u16) {
    let ScreenInfo {
        screenx: xsize,
        vram_pointer,
        ..
    } = sinfo;

    for y in y0..=y1 {
        for x in x0..=x1 {
            unsafe {
                *vram_pointer.offset((y * xsize + x) as isize) = color as u8;
            }
        }
    }
}

pub struct ScreenInfo {
    pub screenx: u16,
    pub screeny: u16,
    pub vram_pointer: *mut u8,
}

pub fn init_screen(sinfo: &ScreenInfo) {
    let ScreenInfo {
        screenx: xsize,
        screeny: ysize,
        ..
    } = sinfo;
    use Color::*;

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
        boxfill8(&sinfo, info.0, info.1, info.2, info.3, info.4);
    }
}

use crate::font::FONT_DATA;
const FONT_WIDTH: isize = 8;
const FONT_HEIGHT: isize = 16;

pub struct ScreenStringWriter<'a> {
    sinfo: &'a ScreenInfo,
    x: isize,
    x0: isize,
    y: isize,
    color: Color,
}

impl<'a> ScreenStringWriter<'a> {
    pub fn new(sinfo: &'a ScreenInfo, x: isize, y: isize, color: Color) -> Self {
        ScreenStringWriter {
            sinfo,
            x,
            x0: x,
            y,
            color,
        }
    }

    pub fn newline(&mut self) {
        self.x = self.x0;
        self.y += FONT_HEIGHT;
    }

    fn putfont8(&mut self, id: u8) {
        let ScreenStringWriter {
            sinfo, x, y, color, ..
        } = *self;
        let xsize = sinfo.screenx;
        let font = FONT_DATA[id as usize];
        for i in 0..16 {
            let d = font[i];
            unsafe {
                let p = sinfo
                    .vram_pointer
                    .offset(((y + i as isize) * xsize as isize + x) as isize);
                if d & 0x80 != 0 {
                    *p = color as u8;
                }
                if d & 0x40 != 0 {
                    *p.offset(1) = color as u8;
                }
                if d & 0x20 != 0 {
                    *p.offset(2) = color as u8;
                }
                if d & 0x10 != 0 {
                    *p.offset(3) = color as u8;
                }
                if d & 0x08 != 0 {
                    *p.offset(4) = color as u8;
                }
                if d & 0x04 != 0 {
                    *p.offset(5) = color as u8;
                }
                if d & 0x02 != 0 {
                    *p.offset(6) = color as u8;
                }
                if d & 0x01 != 0 {
                    *p.offset(7) = color as u8;
                }
            }
        }
    }

    fn putfonts8_ascii(&mut self, string: &str) {
        let screenx = self.sinfo.screenx;
        if !string.is_ascii() {
            self.putfonts8_ascii("NOT ASCII!!!");
            return;
        }
        for item in string.chars() {
            if item == '\n' {
                self.newline();
                continue;
            }
            self.putfont8(item as u8);
            if screenx as isize <= self.x + FONT_WIDTH {
                self.newline();
            } else {
                self.x += FONT_WIDTH;
            }
        }
    }
}

impl<'a> core::fmt::Write for ScreenStringWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.putfonts8_ascii(s);
        Ok(())
    }
}
