use crate::util::clip;
use alloc::vec::Vec;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use vga::drawing::Point;
use vga::writers::{Graphics320x200x256, GraphicsWriter};

pub const SCREEN_WIDTH: isize = 320;
pub const SCREEN_HEIGHT: isize = 200;

pub const CURSOR_WIDTH: usize = 16;
pub const CURSOR_HEIGHT: usize = 16;

pub mod colors256;

use colors256::Color;

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

lazy_static! {
    pub static ref MODE: Graphics320x200x256 = {
        let mode = Graphics320x200x256::new();
        mode.set_mode();
        mode.clear_screen(Color::Black as u8);
        mode
    };
    pub static ref WINDOW_CONTROL: Mutex<WindowControl<'static>> =
        Mutex::new(WindowControl::new(&MODE));
    pub static ref MOUSE_ID: usize = {
        let mouse_id = WINDOW_CONTROL
            .lock()
            .allocate((CURSOR_WIDTH as isize, CURSOR_HEIGHT as isize))
            .unwrap();
        for y in 0..CURSOR_HEIGHT {
            for x in 0..CURSOR_WIDTH {
                let color = match CURSOR[x][y] {
                    b'*' => Some(Color::Black),
                    b'O' => Some(Color::White),
                    _ => None,
                };
                WINDOW_CONTROL.lock().windows[mouse_id]
                    .write_pixel_to_buf((x as isize, y as isize), color);
            }
        }
        WINDOW_CONTROL.lock().change_window_height(mouse_id, 1);
        mouse_id
    };
}

pub fn graphic_mode() {
    // Need this code to evaluate MOUSE_ID
    crate::println!("{:?}", *MOUSE_ID);
}

const MAX_WIN_NUM: usize = 256;

bitflags! {
    struct WinFlag: u32 {
        const USE = 0b00000001;
    }
}

pub struct WindowControl<'a> {
    pub mode: &'a Graphics320x200x256,
    /// pointers to the registered windows.
    pub windows: Vec<Window>,
    /// map height to windows index. windows with height==-1 is not mapped.
    height_to_windows_idx: [usize; MAX_WIN_NUM],
    top: isize,
}

impl<'a> WindowControl<'a> {
    pub fn new(mode: &'a Graphics320x200x256) -> Self {
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
    pub fn allocate(&mut self, size: Point<isize>) -> Option<usize> {
        for i in 0..MAX_WIN_NUM {
            if !self.windows[i].flag.contains(WinFlag::USE) {
                let win = &mut self.windows[i];
                win.flag = WinFlag::USE;
                win.height = -1;
                win.adjust(size);
                return Some(i);
            }
        }
        return None;
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
        let window_area = self.windows[idx_to_move].area();
        // self.refresh_screen(Some(window_area));
    }

    pub fn free(&mut self, window_id: usize) {
        if self.windows[window_id].height >= 0 {
            self.change_window_height(window_id, -1);
        }
        unimplemented!()
    }

    /// Refresh screen in the refresh_area.
    /// If refresh_area is not given, whole screen is refreshed.
    /// 1. refresh with black
    /// 2. refresh with windows
    pub fn refresh_screen(&mut self, refresh_area: Option<(Point<isize>, Point<isize>)>) {
        use core::cmp::{max, min};

        // refresh with black
        let (xrange, yrange) = if let Some(refresh_area) = refresh_area {
            let area_topleft = refresh_area.0;
            let area_bottomright = refresh_area.1;
            (
                area_topleft.0..area_bottomright.0,
                area_topleft.1..area_bottomright.1,
            )
        } else {
            (0..SCREEN_WIDTH, 0..SCREEN_HEIGHT)
        };
        for y in yrange.clone() {
            for x in xrange.clone() {
                {
                    if 0 <= x && x < SCREEN_WIDTH && 0 <= y && y < SCREEN_HEIGHT {
                        MODE.set_pixel(x as usize, y as usize, Color::Black as u8);
                    }
                }
            }
        }

        // refresh with windows
        for h in 0..=self.top {
            let window = &self.windows[self.height_to_windows_idx[h as usize]];
            let buf = &window.buf;
            let buffer_topleft = window.top_left;
            let buffer_bottomright = (
                window.top_left.0 + window.size.0,
                window.top_left.1 + window.size.1,
            );
            let (xrange, yrange) = if let Some(refresh_area) = refresh_area {
                let area_topleft = refresh_area.0;
                let area_bottomright = refresh_area.1;
                (
                    max(buffer_topleft.0, area_topleft.0)
                        ..min(buffer_bottomright.0, area_bottomright.0),
                    max(buffer_topleft.1, area_topleft.1)
                        ..min(buffer_bottomright.1, area_bottomright.1),
                )
            } else {
                (
                    buffer_topleft.0..buffer_topleft.0 + window.size.0,
                    buffer_topleft.1..buffer_topleft.1 + window.size.1,
                )
            };
            for y in yrange.clone() {
                for x in xrange.clone() {
                    if let Some(row) =
                        buf[(y - buffer_topleft.1) as usize][(x - buffer_topleft.0) as usize]
                    {
                        if 0 <= x && x < SCREEN_WIDTH && 0 <= y && y < SCREEN_HEIGHT {
                            MODE.set_pixel(x as usize, y as usize, row as u8);
                        }
                    }
                }
            }
        }
        // TODO: update mouse window
    }
}

pub struct Window {
    top_left: Point<isize>,
    size: Point<isize>,
    column_position: Point<isize>,
    // column_len: isize,
    // line_len: isize,
    buf: Vec<Vec<Option<Color>>>,
    foreground: Color,
    background: Color,
    height: i32,
    /// 透明/色番号（color and invisible）
    // col_inv: i32,
    flag: WinFlag,
}

impl Window {
    pub fn new(top_left: Point<isize>, size: Point<isize>, column_position: Point<isize>) -> Self {
        Self {
            foreground: Color::White,
            background: Color::Black,
            top_left,
            size,
            buf: Self::create_buffer(size, Color::Black),
            column_position,
            // col_inv: 0,
            height: 0,
            flag: WinFlag::empty(),
        }
    }
    /// returns position and size of the window
    pub fn position(&self) -> (Point<isize>, Point<isize>) {
        return (self.top_left, self.size);
    }
    /// returns area of the window in the screen.
    pub fn area(&self) -> (Point<isize>, Point<isize>) {
        (
            self.top_left,
            (self.top_left.0 + self.size.0, self.top_left.1 + self.size.1),
        )
    }
    pub fn line_area(&self) -> (Point<isize>, Point<isize>) {
        (
            (self.top_left.0, self.column_position.1),
            (
                self.top_left.0 + self.size.0,
                self.column_position.1 + FONT_HEIGHT,
            ),
        )
    }
    pub fn adjust(&mut self, new_size: Point<isize>) {
        self.size = new_size;
        self.buf = Self::create_buffer(new_size, self.background);
    }
    pub fn moveby(&mut self, movement: Point<isize>) {
        self.top_left.0 += movement.0;
        self.top_left.0 = clip(self.top_left.0, 0, SCREEN_WIDTH);
        self.top_left.1 += movement.1;
        self.top_left.1 = clip(self.top_left.1, 0, SCREEN_HEIGHT);
    }
    pub fn moveto(&mut self, movement: Point<isize>) {
        self.top_left.0 = movement.0;
        self.top_left.0 = clip(self.top_left.0, 0, SCREEN_WIDTH);
        self.top_left.1 = movement.1;
        self.top_left.1 = clip(self.top_left.1, 0, SCREEN_HEIGHT);
    }
    pub fn change_color(&mut self, foreground: Color, background: Color) {
        self.foreground = foreground;
        self.background = background;
        for line in &mut self.buf {
            for i in 0..line.len() {
                line[i] = Some(background);
            }
        }
    }
    fn create_buffer(size: Point<isize>, background: Color) -> Vec<Vec<Option<Color>>> {
        use alloc::vec;
        vec![vec![Some(background); size.0 as usize]; size.1 as usize]
    }

    pub fn draw_character(&mut self, coord: Point<isize>, chara: char, color: Color) {
        let font = FONT_DATA[chara as usize];
        for i in 0..FONT_HEIGHT {
            let d = font[i as usize];
            for bit in 0..FONT_WIDTH {
                if d & 1 << (FONT_WIDTH - bit - 1) != 0 {
                    self.write_pixel_to_buf(((coord.0 + bit), (coord.1 + i)), Some(color));
                }
            }
        }
    }
    #[inline(always)]
    fn write_pixel_to_buf(&mut self, coord: Point<isize>, color: Option<Color>) {
        self.buf[coord.1 as usize][coord.0 as usize] = color;
    }
    fn clear_buf(&mut self) {
        for i in 0..self.buf.len() {
            for j in 0..self.buf[i].len() {
                self.buf[i][j] = Some(self.background);
            }
        }
    }
    pub fn boxfill(&mut self, color: Color, area: (Point<isize>, Point<isize>)) {
        let (topleft, bottomright) = area;
        for x in topleft.0..=bottomright.0 {
            for y in topleft.1..=bottomright.1 {
                self.write_pixel_to_buf((x, y), Some(color));
            }
        }
    }
    pub fn make_window(&mut self, title: &str) {
        const CLOSE_BUTTON_WIDTH: usize = 16;
        const CLOSE_BUTTON_HEIGHT: usize = 14;
        const CLOSE_BUTTON: [[u8; CLOSE_BUTTON_WIDTH]; CLOSE_BUTTON_HEIGHT] = [
            *b"OOOOOOOOOOOOOOO@",
            *b"OQQQQQQQQQQQQQ$@",
            *b"OQQQQQQQQQQQQQ$@",
            *b"OQQ@@QQQQ@@QQQ$@",
            *b"OQQQ@@QQ@@QQQQ$@",
            *b"OQQQQ@@@@QQQQQ$@",
            *b"OQQQQQ@@QQQQQQ$@",
            *b"OQQQQ@@@@QQQQQ$@",
            *b"OQQQ@@QQ@@QQQQ$@",
            *b"OQQ@@QQQQ@@QQQ$@",
            *b"OQQQQQQQQQQQQQ$@",
            *b"OQQQQQQQQQQQQQ$@",
            *b"O$$$$$$$$$$$$$$@",
            *b"@@@@@@@@@@@@@@@@",
        ];

        let (xsize, ysize) = self.size;

        self.boxfill(Color::LightGrey, ((0, 0), (xsize - 1, 0)));
        self.boxfill(Color::White, ((1, 1), (xsize - 2, 1)));
        self.boxfill(Color::LightGrey, ((0, 0), (0, ysize - 1)));
        self.boxfill(Color::White, ((1, 1), (1, ysize - 2)));
        self.boxfill(Color::DarkGrey, ((xsize - 2, 1), (xsize - 2, ysize - 2)));
        self.boxfill(Color::Black, ((xsize - 1, 0), (xsize - 1, ysize - 1)));
        self.boxfill(Color::LightGrey, ((2, 2), (xsize - 3, ysize - 3)));
        self.boxfill(Color::Black, ((3, 3), (xsize - 4, 20)));
        self.boxfill(Color::DarkGrey, ((1, ysize - 2), (xsize - 2, ysize - 2)));
        self.boxfill(Color::Black, ((0, ysize - 1), (xsize - 1, ysize - 1)));
        for y in 0..CLOSE_BUTTON_HEIGHT {
            for x in 0..CLOSE_BUTTON_WIDTH {
                let c = CLOSE_BUTTON[y][x];
                let color = match c {
                    b'@' => Color::Black,
                    b'$' => Color::DarkGrey,
                    b'Q' => Color::LightGrey,
                    _ => Color::White,
                };
                self.write_pixel_to_buf((xsize - 21 + x as isize, y as isize + 5), Some(color))
            }
        }
        // WINDOW_CONTROL.lock().refresh_screen(Some(self.area()))
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

pub fn draw_mouse(
    // window: &mut Window,
    location: &Point<isize>,
    prev_location: &Point<isize>,
    bc: &Color,
) {
    // overwrite previous location
    for y in 0..CURSOR_HEIGHT {
        for x in 0..CURSOR_WIDTH {
            let color = *bc;
            MODE.set_pixel(
                x + prev_location.0 as usize,
                y + prev_location.1 as usize,
                color as u8,
            );
        }
    }
    // write to next location
    for y in 0..CURSOR_HEIGHT {
        for x in 0..CURSOR_WIDTH {
            let color = match CURSOR[x][y] {
                b'*' => Color::Black,
                b'O' => Color::White,
                _ => *bc,
            };
            MODE.set_pixel(
                x + location.0 as usize,
                y + location.1 as usize,
                color as u8,
            );
        }
    }
}
