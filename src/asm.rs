#![allow(dead_code)]
pub fn cli() {
    unsafe {
        asm!("CLI");
    }
}
pub fn sti() {
    unsafe {
        asm!("STI");
    }
}
pub fn stihlt() {
    unsafe {
        asm!("STI");
        asm!("HLT");
    }
}
