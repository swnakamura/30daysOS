#[repr(C, packed)]
pub struct GateDescriptor {
    offset_low: u16,
    selector: u16,
    dw_count: u8,
    access_right: u8,
    offset_high: u16,
}
impl GateDescriptor {
    pub fn new(offset: u32, selector: u16, ar: u32) -> Self {
        GateDescriptor {
            offset_low: (offset & 0xffff) as u16,
            selector,
            dw_count: (ar >> 8) as u8,
            access_right: (ar & 0xff) as u8,
            offset_high: (offset >> 16) as u16,
        }
    }
}
struct Dtr {
    limit: u16,
    addr: u32,
}
pub fn load_idtr(limit: u16, addr: u32) {
    let idtr = Dtr { limit, addr };
    unsafe {
        #[cfg(feature = "inline_asm")]
        llvm_asm!("LIDT ($0)" :: "r" (gdtr) : "memory");
    }
}
