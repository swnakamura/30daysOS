/// 16 colors from 256 color space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    /// Represents the color `Black (0x0)`.
    Black = 0x0,
    /// Represents the color `Blue (0x1)`.
    Blue = 0x1,
    /// Represents the color `Green (0x2)`.
    Green = 0x2,
    /// Represents the color `Cyan (0x3)`.
    Cyan = 0x3,
    /// Represents the color `Red (0x4)`.
    Red = 0x4,
    /// Represents the color `Magenta (0x5)`.
    Magenta = 0x5,
    /// Represents the color `Brown (0x6)`.
    Brown = 0x6,
    /// Represents the color `LightGrey (0x7)`.
    LightGrey = 0x7,
    /// Represents the color `DarkGrey (0x8)`.
    DarkGrey = 0x8,
    /// Represents the color `LightBlue (0x9)`.
    LightBlue = 0x9,
    /// Represents the color `LightGreen (0xA)`.
    LightGreen = 0xA,
    /// Represents the color `LightCyan (0xB)`.
    LightCyan = 0xB,
    /// Represents the color `LightRed (0xC)`.
    LightRed = 0xC,
    /// Represents the color `Pink (0xD)`.
    Pink = 0xD,
    /// Represents the color `Yellow (0xE)`.
    Yellow = 0xE,
    /// Represents the color `White`.
    White = 65,
}

// // somehow this doesn't work.
// impl From<Color> for u8 {
//     fn from(value: Color) -> u8 {
//         value as u8
//     }
// }
