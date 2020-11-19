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

// use x86_64::instructions::port;

// pub fn out8(port: u32, data: u32) {
//     unsafe {
//         asm!("MOV RDX, {:r}", in(reg) port);
//         asm!("MOV RAX, {:r}", in(reg) data);
//         asm!("OUT DX, AL");
//     }
// }

// pub fn out16(port: u32, data: u32) {
//     unsafe {
//         asm!("MOV RDX, {:r}", in(reg) port);
//         asm!("MOV RAX, {:r}", in(reg) data);
//         asm!("OUT DX, AX");
//     }
// }

// pub fn out32(port: u32, data: u32) {
//     unsafe {
//         asm!("MOV RDX, {:r}", in(reg) port);
//         asm!("MOV RAX, {:r}", in(reg) data);
//         asm!("OUT DX, RAX");
//     }
// }

// pub fn in8(port: u32) -> u32 {
//     let data: u32;
//     unsafe {
//         asm!("MOV RDX, {:r}", in(reg) port);
//         asm!("MOV RAX, 0");
//         asm!("IN  AL,  DX");
//         asm!("MOV {},  AL", out(reg) data);
//     }
//     data
// }

// pub fn in16(port: u32) -> u32 {
//     let data: u32;
//     unsafe {
//         asm!("MOV RDX, {:r}", in(reg) port);
//         asm!("MOV RAX, 0");
//         asm!("MOV {},  AL", out(reg) data);
//     }
//     data
// }

// pub fn in32(port: u32) -> u32 {
//     let data: u32;
//     unsafe {
//         asm!("MOV RDX, {:r}", in(reg) port);
//         asm!("MOV RAX, 0");
//         asm!("MOV {},  AL", out(reg) data);
//     }
//     data
// }

// pub fn load_eflags() -> u32 {
//     let ret: u32;
//     unsafe {
//         asm!("PUSHFD");
//         asm!("POP {0}", out(reg) ret);
//     }
//     ret
// }

// pub fn store_eflags(eflags: u32) {
//     unsafe {
//         asm!("PUSH {0}", in(reg) eflags);
//         asm!("POPFD");
//     }
// }
