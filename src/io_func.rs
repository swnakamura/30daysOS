pub fn cli() {
    unsafe {
        asm!("CLI");
    }
}
pub fn sti(eflags: u32) {
    unsafe {
        asm!("STI");
    }
}

pub fn out8(port: u32, data: u32) {
    unsafe {
        asm!("MOV EDX, {}", in(reg) port);
        asm!("MOV EAX, {}", in(reg) data);
        asm!("OUT DX, AL");
    }
}

pub fn load_eflags() -> u32 {
    let ret: u32;
    unsafe {
        asm!("PUSHFD");
        asm!("POP {0}", out(reg) ret);
    }
    ret
}

pub fn store_eflags(eflags: u32) {
    unsafe {
        asm!("PUSH {0}", in(reg) eflags);
        asm!("POPFD");
    }
}
