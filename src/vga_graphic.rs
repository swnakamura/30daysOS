use crate::util::clip;
use alloc::vec::Vec;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use vga::colors::Color16;
use vga::drawing::Point;
use vga::writers::{Graphics640x480x16, GraphicsWriter};
pub const SCREEN_WIDTH: isize = 640;
pub const SCREEN_HEIGHT: isize = 480;

lazy_static! {
    pub static ref MODE: Graphics640x480x16 = {
        let mode = Graphics640x480x16::new();
        mode.set_mode();
        mode.clear_screen(Color16::Black);
        mode
    };
    pub static ref WINDOW_CONTROL: Mutex<WindowControl<'static>> =
        Mutex::new(WindowControl::new(&MODE));
}

const MAX_WIN_NUM: usize = 256;

bitflags! {
    struct WinFlag: u32 {
        const USE = 0b00000001;
    }
}

pub struct WindowControl<'a> {
    pub mode: &'a Graphics640x480x16,
    /// pointers to the registered windows.
    pub windows: Vec<Window>,
    /// map height to windows index. windows with height==-1 is not mapped.
    height_to_windows_idx: [usize; MAX_WIN_NUM],
    top: isize,
}

impl<'a> WindowControl<'a> {
    pub fn new(mode: &'a Graphics640x480x16) -> Self {
        let mut windows = Vec::with_capacity(MAX_WIN_NUM);
        for _ in 0..MAX_WIN_NUM {
            windows.push(Window::new((0, 0), (0, 0), (0, 0)));
        }
        Self {
            mode,
            windows,
            height_to_windows_idx: [0; MAX_WIN_NUM],
            top: -1,
        }
    }
    /// register a new window
    pub fn allocate(&mut self) -> usize {
        for i in 0..MAX_WIN_NUM {
            if !self.windows[i].flag.contains(WinFlag::USE) {
                let win = &mut self.windows[i];
                win.flag = WinFlag::USE;
                win.height = -1;
                return i;
            }
        }
        return 0;
    }

    pub fn change_window_height(&mut self, idx_to_move: usize, new_height: i32) {
        let new_height = clip(new_height, -1, self.top as i32 + 1);
        let old_height = self.windows[idx_to_move].height;
        self.windows[idx_to_move].height = new_height;
        if new_height < old_height {
            if new_height > -1 {
                for h in (new_height + 1..=old_height).rev() {
                    let h_usize = h as usize;
                    self.height_to_windows_idx[h_usize] = self.height_to_windows_idx[h_usize - 1];
                    self.windows[self.height_to_windows_idx[h_usize]].height = h;
                }
                self.height_to_windows_idx[new_height as usize] = idx_to_move;
            } else {
                // hide window
                for h in old_height..self.top as i32 {
                    let h_usize = h as usize;
                    self.height_to_windows_idx[h_usize] = self.height_to_windows_idx[h_usize + 1];
                    self.windows[self.height_to_windows_idx[h_usize]].height = h;
                }
                self.top -= 1;
            }
        } else if old_height < new_height {
            if old_height >= 0 {
                for h in old_height..new_height {
                    let h_usize = h as usize;
                    self.height_to_windows_idx[h_usize] = self.height_to_windows_idx[h_usize + 1];
                    self.windows[self.height_to_windows_idx[h_usize]].height = h;
                }
                self.height_to_windows_idx[new_height as usize] = idx_to_move;
            } else {
                // unhide window
                for h in (new_height..self.top as i32).rev() {
                    let h_usize = h as usize;
                    self.height_to_windows_idx[h_usize + 1] = self.height_to_windows_idx[h_usize];
                    self.windows[self.height_to_windows_idx[h_usize + 1]].height = h + 1;
                }
                self.height_to_windows_idx[new_height as usize] = idx_to_move;
                self.top += 1;
            }
        }
        self.refresh_screen();
    }

    pub fn free(&mut self, window_id: usize) {
        if self.windows[window_id].height >= 0 {
            self.change_window_height(window_id, -1);
        }
        unimplemented!()
    }

    pub fn refresh_screen(&mut self) {
        self.mode.clear_screen(Color16::Black);
        for h in 0..=self.top {
            let window = &self.windows[self.height_to_windows_idx[h as usize]];
            let buf = &window.buf;
            for (line_num, line) in buf.iter().enumerate() {
                for (row_num, row) in line.iter().enumerate() {
                    MODE.set_pixel(
                        (window.top_left.0 + row_num as isize) as usize,
                        (window.top_left.1 + line_num as isize) as usize,
                        *row,
                    );
                }
            }
        }
    }
}

pub struct Window {
    top_left: Point<isize>,
    size: Point<isize>,
    column_position: Point<isize>,
    // column_len: isize,
    // line_len: isize,
    buf: Vec<Vec<Color16>>,
    foreground: Color16,
    background: Color16,
    height: i32,
    /// 透明/色番号（color and invisible）
    // col_inv: i32,
    flag: WinFlag,
}

impl Window {
    pub fn new(top_left: Point<isize>, size: Point<isize>, column_position: Point<isize>) -> Self {
        Self {
            foreground: Color16::White,
            background: Color16::Black,
            top_left,
            size,
            buf: Self::create_buffer(size, Color16::Black),
            column_position,
            // col_inv: 0,
            height: 0,
            flag: WinFlag::empty(),
        }
    }
    pub fn adjust(&mut self, new_size: Point<isize>) {
        self.size = new_size;
        self.buf = Self::create_buffer(new_size, self.background);
    }
    pub fn change_color(&mut self, foreground: Color16, background: Color16) {
        self.foreground = foreground;
        self.background = background;
        for line in &mut self.buf {
            for i in 0..line.len() {
                line[i] = background;
            }
        }
    }
    fn create_buffer(size: Point<isize>, background: Color16) -> Vec<Vec<Color16>> {
        use alloc::vec;
        vec![vec![background; size.0 as usize]; size.1 as usize]
    }

    pub fn draw_character(&mut self, coord: Point<isize>, chara: char, color: Color16) {
        let font = FONT_DATA[chara as usize];
        for i in 0..FONT_HEIGHT {
            let d = font[i as usize];
            for bit in 0..FONT_WIDTH {
                if d & 1 << (FONT_WIDTH - bit - 1) != 0 {
                    self.write_pixel_to_buf(((coord.0 + bit), (coord.1 + i)), color);
                }
            }
        }
    }
    fn write_pixel_to_buf(&mut self, coord: Point<isize>, color: Color16) {
        self.buf[coord.1 as usize][coord.0 as usize] = color;
    }
    fn clear_buf(&mut self) {
        for i in 0..self.buf.len() {
            for j in 0..self.buf[i].len() {
                self.buf[i][j] = self.background;
            }
        }
    }
}

const FONT_WIDTH: isize = 8;
const FONT_HEIGHT: isize = 16;
type Font = [[u16; 16]; 256];
const FONT_DATA: Font = include!("../build/font.in");

impl fmt::Write for Window {
    fn write_str(&mut self, string: &str) -> Result<(), core::fmt::Error> {
        string.chars().for_each(|c| {
            if c == '\n' {
                self.column_position = (0, self.column_position.1 + FONT_HEIGHT);
                return;
            } else {
                self.draw_character(
                    (self.column_position.0, self.column_position.1),
                    c,
                    self.foreground,
                );
            }
            self.column_position.0 += FONT_WIDTH;
            if self.column_position.0 + FONT_WIDTH > self.size.0 {
                self.column_position.0 = 0;
                self.column_position.1 += FONT_HEIGHT;
            }
            if self.column_position.1 + FONT_HEIGHT > self.size.1 {
                self.clear_buf();
                self.column_position = (0, 0);
            }
        });
        Ok(())
    }
}
// #[macro_export]
// macro_rules! print_graphic {
//         ($($arg:tt)*) => ($crate::vga_graphic::_print(format_args!($($arg)*)));
//     }

// #[macro_export]
// macro_rules! println_graphic {
//         () => (print!("\n"));
//         ($($arg:tt)*) => ($crate::print_graphic!("{}\n", format_args!($($arg)*)));
//     }

// #[doc(hidden)]
// pub fn _print(args: fmt::Arguments) {
//     use core::fmt::Write;
//     use x86_64::instructions::interrupts;
//     interrupts::without_interrupts(|| {
//         SCREEN_BG.write_fmt(args).unwrap();
//     });
// }

const CURSOR_WIDTH: usize = 16;
const CURSOR_HEIGHT: usize = 16;

const CURSOR: [[u8; CURSOR_WIDTH]; CURSOR_HEIGHT] = [
    *b"**************..",
    *b"*OOOOOOOOOOO*...",
    *b"*OOOOOOOOOO*....",
    *b"*OOOOOOOOO*.....",
    *b"*OOOOOOOO*......",
    *b"*OOOOOOO*.......",
    *b"*OOOOOOO*.......",
    *b"*OOOOOOOO*......",
    *b"*OOOO**OOO*.....",
    *b"*OOO*..*OOO*....",
    *b"*OO*....*OOO*...",
    *b"*O*......*OOO*..",
    *b"**........*OOO*.",
    *b"*..........*OOO*",
    *b"............*OO*",
    *b".............***",
];

pub fn draw_mouse(
    // window: &mut Window,
    location: &Point<isize>,
    prev_location: &Point<isize>,
    bc: &Color16,
) {
    // overwrite previous location
    for y in 0..CURSOR_HEIGHT {
        for x in 0..CURSOR_WIDTH {
            let color = *bc;
            MODE.set_pixel(
                x + prev_location.0 as usize,
                y + prev_location.1 as usize,
                color,
            );
        }
    }
    // write to next location
    for y in 0..CURSOR_HEIGHT {
        for x in 0..CURSOR_WIDTH {
            let color = match CURSOR[x][y] {
                b'*' => Color16::Black,
                b'O' => Color16::White,
                _ => *bc,
            };
            MODE.set_pixel(x + location.0 as usize, y + location.1 as usize, color);
        }
    }
}
