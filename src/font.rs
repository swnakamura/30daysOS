type Font = [[u16; 16]; 256];

pub const fontdata: Font = include!("../build/font.in");
