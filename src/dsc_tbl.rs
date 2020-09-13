mod gdt;
mod idt;

use gdt::*;
use idt::*;

const GDT_ADDR: u32 = 0x00270000;
const IDT_ADDR: u32 = 0x0026f800;

pub fn init_gdtidt() {
    let gdt = GDT_ADDR as *mut SegmentDescriptor;

    unsafe {
        for i in 0..8192 {
            *gdt.offset(i) = SegmentDescriptor::new(0, 0, 0);
        }
        // メモリ全体
        *gdt.offset(1) = SegmentDescriptor::new(0xffffffff, 0x00000000, 0x4092);
        // bootpackのため
        *gdt.offset(2) = SegmentDescriptor::new(0x0007ffff, 0x00280000, 0x409a);
    }
    load_gdtr(0xffff, GDT_ADDR);

    let idt = IDT_ADDR as *mut GateDescriptor;
    unsafe {
        for i in 0..256 {
            *idt.offset(i) = GateDescriptor::new(0, 0, 0);
        }
    }
    load_idtr(0x7ff, 0x0026f800);

    return;
}
