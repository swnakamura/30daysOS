use crate::gdt;
use crate::{print, println};
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    /// register handler functions to IDT
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // breakpoint
        idt.breakpoint.set_handler_fn(handler::breakpoint);
        // double fault
        unsafe {
            idt.double_fault
                .set_handler_fn(handler::double_fault)
                .set_stack_index(gdt::DOUBLE_FAULT_DEFAULT_IST_INDEX);
        }
        // timer
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(handler::timer_interrupt);
        // keyboard
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(handler::keyboard_interrupt);
        idt
    };
}

/// handler functions
mod handler {
    use super::*;
    pub extern "x86-interrupt" fn breakpoint(stack_frame: &mut InterruptStackFrame) {
        println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    }

    pub extern "x86-interrupt" fn double_fault(
        stack_frame: &mut InterruptStackFrame,
        _error_code: u64,
    ) -> ! {
        panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
        loop {}
    }

    pub extern "x86-interrupt" fn timer_interrupt(_stack_frame: &mut InterruptStackFrame) {
        print!(".");
        unsafe {
            PICS.lock()
                .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
        }
    }

    pub extern "x86-interrupt" fn keyboard_interrupt(stack_frame: &mut InterruptStackFrame) {
        use x86_64::instructions::port::Port;
        let port: Port = Port::new(0x60);
        let data = port.read();
    }
}

pub fn init_idt() {
    IDT.load();
}

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 0x08;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
