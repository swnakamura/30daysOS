use crate::util::clip;
use alloc::vec;
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
    pub static ref SHEET_CONTROL: Mutex<SheetControl<'static>> =
        Mutex::new(SheetControl::new(&MODE));
    pub static ref MOUSE_ID: usize = {
        let mut sheet_control = SHEET_CONTROL.lock();
        let mouse_id = sheet_control
            .allocate((CURSOR_WIDTH as isize, CURSOR_HEIGHT as isize))
            .unwrap();
        for y in 0..CURSOR_HEIGHT {
            for x in 0..CURSOR_WIDTH {
                let color = match CURSOR[x][y] {
                    b'*' => Some(Color::Black),
                    b'O' => Some(Color::White),
                    _ => None,
                };
                sheet_control.sheets[mouse_id].write_pixel_to_buf((x as isize, y as isize), color);
            }
        }
        sheet_control.change_sheet_height(mouse_id, 1);
        mouse_id
    };
}

pub fn graphic_mode() {
    // this code is needed to evaluate MOUSE_ID
    crate::println!("{:?}", *MOUSE_ID);
}

const MAX_WIN_NUM: usize = 256;

bitflags! {
    struct WinFlag: u32 {
        const USE = 0b00000001;
    }
}

pub struct SheetControl<'a> {
    /// Reference to the graphic mode.
    pub mode: &'a Graphics320x200x256,
    /// Reference to the registered sheets.
    pub sheets: Vec<Sheet>,
    /// Map height to sheets index. Sheets with height==-1 is not mapped.
    height_to_sheets_idx: [usize; MAX_WIN_NUM],
    /// The highest sheet height.
    top: isize,
    /// Represents the height of the "owner" sheet of the pixel which is the highest sheet at the
    /// pixel.
    map: Vec<Vec<isize>>,
}

impl<'a> SheetControl<'a> {
    pub fn new(mode: &'a Graphics320x200x256) -> Self {
        let mut sheets = Vec::with_capacity(MAX_WIN_NUM);
        for _ in 0..MAX_WIN_NUM {
            sheets.push(Sheet::new((0, 0), (0, 0), (0, 0)));
        }
        Self {
            mode,
            sheets,
            height_to_sheets_idx: [0; MAX_WIN_NUM],
            top: -1,
            map: vec![vec![0; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],
        }
    }
    /// Register a new sheet with the given size.
    pub fn allocate(&mut self, size: Point<isize>) -> Option<usize> {
        for i in 0..MAX_WIN_NUM {
            if !self.sheets[i].flag.contains(WinFlag::USE) {
                let win = &mut self.sheets[i];
                win.flag = WinFlag::USE;
                win.height = -1;
                win.adjust(size);
                return Some(i);
            }
        }
        return None;
    }

    pub fn change_sheet_height(&mut self, idx_to_move: usize, new_height: i32) {
        let new_height = clip(new_height, -1, self.top as i32 + 1);
        let old_height = self.sheets[idx_to_move].height;
        self.sheets[idx_to_move].height = new_height;
        if new_height < old_height {
            if new_height > -1 {
                for h in (new_height + 1..=old_height).rev() {
                    let h_usize = h as usize;
                    self.height_to_sheets_idx[h_usize] = self.height_to_sheets_idx[h_usize - 1];
                    self.sheets[self.height_to_sheets_idx[h_usize]].height = h;
                }
                self.height_to_sheets_idx[new_height as usize] = idx_to_move;
            } else {
                // hide sheet
                for h in old_height..self.top as i32 {
                    let h_usize = h as usize;
                    self.height_to_sheets_idx[h_usize] = self.height_to_sheets_idx[h_usize + 1];
                    self.sheets[self.height_to_sheets_idx[h_usize]].height = h;
                }
                self.top -= 1;
            }
        } else if old_height < new_height {
            if old_height >= 0 {
                for h in old_height..new_height {
                    let h_usize = h as usize;
                    self.height_to_sheets_idx[h_usize] = self.height_to_sheets_idx[h_usize + 1];
                    self.sheets[self.height_to_sheets_idx[h_usize]].height = h;
                }
                self.height_to_sheets_idx[new_height as usize] = idx_to_move;
            } else {
                // unhide sheet
                for h in (new_height..self.top as i32).rev() {
                    let h_usize = h as usize;
                    self.height_to_sheets_idx[h_usize + 1] = self.height_to_sheets_idx[h_usize];
                    self.sheets[self.height_to_sheets_idx[h_usize + 1]].height = h + 1;
                }
                self.height_to_sheets_idx[new_height as usize] = idx_to_move;
                self.top += 1;
            }
        }
        // let sheet_area = self.sheets[idx_to_move].area();
        // self.refresh_screen(Some(sheet_area));
    }

    /// Remove sheet from allocation.
    pub fn free(&mut self, sheet_id: usize) {
        if self.sheets[sheet_id].height >= 0 {
            self.change_sheet_height(sheet_id, -1);
        }
        unimplemented!()
    }

    /// Refreshes screen for the pixels within the refresh_area.
    /// If refresh_area is not given, whole screen is refreshed.
    pub fn refresh_screen(
        &mut self,
        refresh_area: Option<(Point<isize>, Point<isize>)>,
        refreshed_sheet_height: Option<isize>,
    ) {
        use core::cmp::{max, min};

        // refresh with sheets
        for h in refreshed_sheet_height.unwrap_or(0)..=self.top {
            let sheet = &self.sheets[self.height_to_sheets_idx[h as usize]];
            let buf = &sheet.buf;
            let buffer_topleft = sheet.top_left;
            let buffer_bottomright = (
                sheet.top_left.0 + sheet.size.0,
                sheet.top_left.1 + sheet.size.1,
            );
            let (xrange, yrange) = if let Some(refresh_area) = refresh_area {
                // refresh_areaが与えられているなら、bufferの範囲とrefresh_areaの範囲のintersectにする
                let area_topleft = refresh_area.0;
                let area_bottomright = refresh_area.1;
                (
                    max(buffer_topleft.0, area_topleft.0)
                        ..min(buffer_bottomright.0, area_bottomright.0),
                    max(buffer_topleft.1, area_topleft.1)
                        ..min(buffer_bottomright.1, area_bottomright.1),
                )
            } else {
                // そうでないなら単純に全体
                (
                    buffer_topleft.0..buffer_topleft.0 + sheet.size.0,
                    buffer_topleft.1..buffer_topleft.1 + sheet.size.1,
                )
            };
            for y in yrange.clone() {
                for x in xrange.clone() {
                    if let Some(row) =
                        buf[(y - buffer_topleft.1) as usize][(x - buffer_topleft.0) as usize]
                    {
                        if 0 <= x
                            && x < SCREEN_WIDTH
                            && 0 <= y
                            && y < SCREEN_HEIGHT
                            && self.map[y as usize][x as usize] == h
                        {
                            MODE.set_pixel(x as usize, y as usize, row as u8);
                        }
                    }
                }
            }
        }
    }
    /// Refreshes screen for the pixels according to the `areas_to_refresh`, which are the
    /// accumulation of the characters written into buffer.
    pub fn flush_printed_chars(&mut self, refreshed_sheet_height: Option<isize>) {
        for h in refreshed_sheet_height.unwrap_or(0)..=self.top {
            for area in self.sheets[self.height_to_sheets_idx[h as usize]]
                .areas_to_refresh
                .clone()
                .iter()
            {
                // let tl = self.sheets[self.height_to_sheets_idx[h as usize]].top_left;
                // let area = (area.0 + tl.0, area.1 + tl.1);
                self.refresh_screen(Some(*area), Some(h));
            }
            self.sheets[self.height_to_sheets_idx[h as usize]].areas_to_refresh = Vec::new();
        }
    }

    /// Refreshes map for the pixels within the refresh_area.
    /// If refresh_area is not given, whole screen is refreshed.
    ///
    /// TODO: the logic is mostly the same with refresh_screen... we can simplify them somehow
    pub fn refresh_sheet_map(
        &mut self,
        refresh_area: Option<(Point<isize>, Point<isize>)>,
        refreshed_sheet_height: Option<isize>,
    ) {
        use core::cmp::{max, min};

        let refreshed_sheet_height = refreshed_sheet_height.unwrap_or(0);

        for h in refreshed_sheet_height..=self.top {
            let sheet = &self.sheets[self.height_to_sheets_idx[h as usize]];
            let buf = &sheet.buf;
            let buffer_topleft = sheet.top_left;
            let buffer_bottomright = (
                sheet.top_left.0 + sheet.size.0,
                sheet.top_left.1 + sheet.size.1,
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
                    buffer_topleft.0..buffer_topleft.0 + sheet.size.0,
                    buffer_topleft.1..buffer_topleft.1 + sheet.size.1,
                )
            };
            for y in yrange.clone() {
                for x in xrange.clone() {
                    // if buffer is not none at this pixel, then screen should be updated with the
                    // buffer
                    if let Some(_) =
                        buf[(y - buffer_topleft.1) as usize][(x - buffer_topleft.0) as usize]
                    {
                        if 0 <= x && x < SCREEN_WIDTH && 0 <= y && y < SCREEN_HEIGHT {
                            self.map[y as usize][x as usize] = h;
                        }
                    }
                }
            }
        }
    }
}

pub struct Sheet {
    top_left: Point<isize>,
    size: Point<isize>,
    pub column_position: Point<isize>,
    pub initial_column_position: Point<isize>,
    pub buf: Vec<Vec<Option<Color>>>,
    foreground: Color,
    pub background: Color,
    pub height: i32,
    flag: WinFlag,
    pub areas_to_refresh: Vec<(Point<isize>, Point<isize>)>,
}

impl Sheet {
    pub fn new(top_left: Point<isize>, size: Point<isize>, column_position: Point<isize>) -> Self {
        Self {
            foreground: Color::White,
            background: Color::Black,
            top_left,
            size,
            buf: Self::create_buffer(size, Color::Black),
            column_position,
            initial_column_position: (3, 23),
            height: 0,
            flag: WinFlag::empty(),
            areas_to_refresh: Vec::new(),
        }
    }
    /// Returns position and size of the sheet
    pub fn position(&self) -> (Point<isize>, Point<isize>) {
        return (self.top_left, self.size);
    }
    /// Returns area of the sheet in the screen.
    pub fn area(&self) -> (Point<isize>, Point<isize>) {
        (
            self.top_left,
            (self.top_left.0 + self.size.0, self.top_left.1 + self.size.1),
        )
    }
    /// Returns the current line area of the sheet.
    pub fn line_area(&self) -> (Point<isize>, Point<isize>) {
        (
            (self.top_left.0, self.column_position.1),
            (
                self.top_left.0 + self.size.0,
                self.column_position.1 + FONT_HEIGHT,
            ),
        )
    }
    /// Adjust the size of the sheet.
    /// You need to rewrite the buffer after this function.
    pub fn adjust(&mut self, new_size: Point<isize>) {
        self.size = new_size;
        self.buf = Self::create_buffer(new_size, self.background);
    }
    /// Move sheet by the given movement.
    pub fn moveby(&mut self, movement: Point<isize>) {
        self.top_left.0 += movement.0;
        self.top_left.0 = clip(self.top_left.0, 0, SCREEN_WIDTH);
        self.top_left.1 += movement.1;
        self.top_left.1 = clip(self.top_left.1, 0, SCREEN_HEIGHT);
    }
    /// Move sheet to the given movement.
    pub fn moveto(&mut self, coordinate: Point<isize>) {
        self.top_left.0 = coordinate.0;
        self.top_left.0 = clip(self.top_left.0, 0, SCREEN_WIDTH);
        self.top_left.1 = coordinate.1;
        self.top_left.1 = clip(self.top_left.1, 0, SCREEN_HEIGHT);
    }
    /// Change foreground/background of the sheet.
    pub fn change_color(&mut self, foreground: Color, background: Color) {
        self.foreground = foreground;
        self.background = background;
        for line in &mut self.buf {
            for i in 0..line.len() {
                line[i] = Some(background);
            }
        }
    }
    /// Create new buffer.
    fn create_buffer(size: Point<isize>, background: Color) -> Vec<Vec<Option<Color>>> {
        vec![vec![Some(background); size.0 as usize]; size.1 as usize]
    }

    /// draw one character to the buffer.
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
    /// Write given color to the buffer at the given coordinate.
    /// This is useful since we need to specify the coordinate like `buf[y][x]`, which is quite
    /// swappy.
    #[inline(always)]
    fn write_pixel_to_buf(&mut self, coord: Point<isize>, color: Option<Color>) {
        self.buf[coord.1 as usize][coord.0 as usize] = color;
    }
    /// Clear out buffer with `self.background`.
    fn clear_buf(&mut self) {
        for i in 0..self.buf.len() {
            for j in 0..self.buf[i].len() {
                self.buf[i][j] = Some(self.background);
            }
        }
    }
    /// Fill the area given by `area` (inclusive) with the color.
    pub fn boxfill(&mut self, color: Color, area: (Point<isize>, Point<isize>)) {
        let (topleft, bottomright) = area;
        for x in topleft.0..=bottomright.0 {
            for y in topleft.1..=bottomright.1 {
                self.write_pixel_to_buf((x, y), Some(color));
            }
        }
    }
    /// Set up this sheet as background.
    /// Paint it with Cyan, draw menu bar, etc.
    pub fn make_background(&mut self) {
        let (xsize, ysize) = self.size;
        use Color::*;
        self.boxfill(Cyan, ((0, 0), (xsize - 1, ysize - 29)));
        self.boxfill(LightGrey, ((0, ysize - 28), (xsize - 1, ysize - 28)));
        self.boxfill(White, ((0, ysize - 27), (xsize - 1, ysize - 27)));
        self.boxfill(LightGrey, ((0, ysize - 26), (xsize - 1, ysize - 1)));

        self.boxfill(White, ((3, ysize - 24), (59, ysize - 24)));
        self.boxfill(White, ((2, ysize - 24), (2, ysize - 4)));
        self.boxfill(DarkBlue, ((3, ysize - 4), (59, ysize - 4)));
        self.boxfill(DarkBlue, ((59, ysize - 23), (59, ysize - 5)));
        self.boxfill(Black, ((2, ysize - 3), (59, ysize - 3)));
        self.boxfill(Black, ((60, ysize - 24), (60, ysize - 3)));

        self.boxfill(
            DarkBlue,
            ((xsize - 47, ysize - 24), (xsize - 4, ysize - 24)),
        );
        self.boxfill(
            DarkBlue,
            ((xsize - 47, ysize - 23), (xsize - 47, ysize - 4)),
        );
        self.boxfill(White, ((xsize - 47, ysize - 3), (xsize - 4, ysize - 3)));
        self.boxfill(White, ((xsize - 3, ysize - 24), (xsize - 3, ysize - 3)));
    }
    /// Set up this sheet as an ordinary sheetby painting it with LightGrey, draw CLOSE_BUTTON, etc.
    pub fn make_sheet(&mut self, title: &str) {
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
        self.boxfill(Color::Black, ((xsize - 2, 1), (xsize - 2, ysize - 2)));
        self.boxfill(Color::Black, ((xsize - 1, 0), (xsize - 1, ysize - 1)));
        self.boxfill(Color::LightGrey, ((2, 2), (xsize - 3, ysize - 3)));
        self.boxfill(Color::Blue, ((3, 3), (xsize - 4, 20)));
        self.boxfill(Color::Black, ((1, ysize - 2), (xsize - 2, ysize - 2)));
        self.boxfill(Color::Black, ((0, ysize - 1), (xsize - 1, ysize - 1)));
        for y in 0..CLOSE_BUTTON_HEIGHT {
            for x in 0..CLOSE_BUTTON_WIDTH {
                let c = CLOSE_BUTTON[y][x];
                let color = match c {
                    b'@' => Color::Black,
                    b'$' => Color::DarkBlue,
                    b'Q' => Color::LightGrey,
                    _ => Color::White,
                };
                self.write_pixel_to_buf((xsize - 21 + x as isize, y as isize + 5), Some(color))
            }
        }
        use core::fmt::Write;
        self.column_position = (2, 2);
        write!(self, "{}", title).unwrap();
        self.column_position = self.initial_column_position;
    }
}

const FONT_WIDTH: isize = 8;
const FONT_HEIGHT: isize = 16;
type Font = [[u16; 16]; 256];
const FONT_DATA: Font = include!("../build/font.in");

impl fmt::Write for Sheet {
    fn write_str(&mut self, string: &str) -> Result<(), core::fmt::Error> {
        string.chars().for_each(|c| {
            if c == '\n' {
                self.column_position = (
                    self.initial_column_position.0,
                    self.column_position.1 + FONT_HEIGHT,
                );
                return;
            } else {
                self.draw_character(
                    (self.column_position.0, self.column_position.1),
                    c,
                    self.foreground,
                );
                // self.areas_to_refresh.push((
                //     self.column_position,
                //     (
                //         self.column_position.0 + FONT_WIDTH,
                //         self.column_position.1 + FONT_HEIGHT,
                //     ),
                // ));
            }
            self.column_position.0 += FONT_WIDTH;
            if self.column_position.0 + FONT_WIDTH > self.size.0 {
                self.column_position.0 = self.initial_column_position.0;
                self.column_position.1 += FONT_HEIGHT;
            }
            if self.column_position.1 + FONT_HEIGHT > self.size.1 {
                self.clear_buf();
                self.column_position = self.initial_column_position;
            }
        });
        Ok(())
    }
}

pub fn draw_mouse(
    // sheet: &mut Sheet,
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
