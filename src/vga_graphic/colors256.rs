/// important colors from 256 color space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGrey = 0x7,
    DarkBlue = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    Sky = 0xB,
    WineRed = 0xC,
    Purple = 0xD,
    DullYellow = 0xE,
    White = 65,
}

// // somehow this doesn't work.
// impl From<Color> for u8 {
//     fn from(value: Color) -> u8 {
//         value as u8
//     }
// }
