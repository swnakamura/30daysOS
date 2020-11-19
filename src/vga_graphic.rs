use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter};
pub const SCREEN_WIDTH: isize = 640;
pub const SCREEN_HEIGHT: isize = 480;

lazy_static! {
    static ref MODE: Graphics640x480x16 = {
        let mode = Graphics640x480x16::new();
        mode.set_mode();
        mode.clear_screen(Color16::Black);
        mode
    };
    static ref SCREEN_BG: Mutex<Window<'static>> = Mutex::new(Window::new(
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        (0, 0),
        &MODE
    ));
}

pub fn graphic_mode() {
    // let mut window = Window::new((80, 60), (460, 360), (0, 0), &MODE);
    // window.draw_frame();
    // use core::fmt::Write;
    // write!(window, "Hello,World!").unwrap();
    // write!(SCREEN_BG.lock(), "HI");
    crate::println_graphic!("HI");
}

pub struct Window<'a> {
    top_left: Point<isize>,
    size: Point<isize>,
    column_position: Point<isize>,
    column_len: isize,
    line_len: isize,
    color_code: Color16,
    mode: &'a Graphics640x480x16,
}

impl<'a> Window<'a> {
    pub fn new(
        top_left: Point<isize>,
        size: Point<isize>,
        column_position: Point<isize>,
        mode: &'a Graphics640x480x16,
    ) -> Self {
        Self {
            color_code: Color16::White,
            top_left,
            size,
            column_position,
            mode,
            column_len: size.0 / 8,
            line_len: size.1 / 16,
        }
    }
    pub fn draw_frame(&self) {
        self.mode.draw_line(
            self.top_left,
            (self.top_left.0 + self.size.0, self.top_left.1),
            Color16::White,
        );
        self.mode.draw_line(
            self.top_left,
            (self.top_left.0, self.top_left.1 + self.size.1),
            Color16::White,
        );
        self.mode.draw_line(
            (self.top_left.0, self.top_left.1 + self.size.1),
            (self.top_left.0 + self.size.0, self.top_left.1 + self.size.1),
            Color16::White,
        );
        self.mode.draw_line(
            (self.top_left.0 + self.size.0, self.top_left.1),
            (self.top_left.0 + self.size.0, self.top_left.1 + self.size.1),
            Color16::White,
        );
    }
}

impl fmt::Write for Window<'_> {
    fn write_str(&mut self, string: &str) -> Result<(), core::fmt::Error> {
        string.chars().for_each(|c| {
            if c == '\n' {
                self.column_position = (0, self.column_position.1 + 10);
            } else {
                self.mode.draw_character(
                    (self.top_left.0 + self.column_position.0) as usize,
                    (self.top_left.1 + self.column_position.1) as usize,
                    c,
                    Color16::White,
                );
            }
            self.column_position.0 += 8;
            if self.column_position.0 > self.size.0 {
                self.column_position.0 = 0;
                self.column_position.1 += 10;
            }
            if self.column_position.1 > self.size.1 {
                self.mode.clear_screen(Color16::Black);
                self.column_position = (0, 0);
            }
        });
        Ok(())
    }
}
#[macro_export]
macro_rules! print_graphic {
        ($($arg:tt)*) => ($crate::vga_graphic::_print(format_args!($($arg)*)));
    }

#[macro_export]
macro_rules! println_graphic {
        () => (print!("\n"));
        ($($arg:tt)*) => ($crate::print_graphic!("{}\n", format_args!($($arg)*)));
    }

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        SCREEN_BG.lock().write_fmt(args).unwrap();
    });
}

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

pub fn draw_mouse(location: &Point<isize>, prev_location: &Point<isize>, bc: &Color16) {
    for y in 0..CURSOR_HEIGHT {
        for x in 0..CURSOR_WIDTH {
            let color = *bc;
            SCREEN_BG.lock().mode.set_pixel(
                x + prev_location.0 as usize,
                y + prev_location.1 as usize,
                color,
            );
        }
    }
    for y in 0..CURSOR_HEIGHT {
        for x in 0..CURSOR_WIDTH {
            let color = match CURSOR[x][y] {
                b'*' => Color16::Black,
                b'O' => Color16::White,
                _ => *bc,
            };
            SCREEN_BG.lock().mode.set_pixel(
                x + location.0 as usize,
                y + location.1 as usize,
                color,
            );
        }
    }
}
