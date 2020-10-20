type Font = [[u16; 16]; 256];

pub const FONT_DATA: Font = include!("../build/font.in");
