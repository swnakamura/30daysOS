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
        idt.breakpoint.set_handler_fn(handler::breakpoint_handler);
        // double fault
        unsafe {
            idt.double_fault
                .set_handler_fn(handler::double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_DEFAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(handler::page_fault_handler);
        // timer
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(handler::timer_interrupt_handler);
        // keyboard
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(handler::keyboard_interrupt_handler);
        // TODO: mouse
        idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(handler::mouse_interrupt_handler);
        idt
    };
}

/// handler functions
mod handler {
    use super::*;
    pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
        println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    }

    pub extern "x86-interrupt" fn double_fault_handler(
        stack_frame: &mut InterruptStackFrame,
        _error_code: u64,
    ) -> ! {
        panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    }

    pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
        // print!(".");

        // notify end of interrupt
        unsafe {
            PICS.lock()
                .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
        }
    }

    pub extern "x86-interrupt" fn keyboard_interrupt_handler(
        _stack_frame: &mut InterruptStackFrame,
    ) {
        use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
        use spin::Mutex;
        use x86_64::instructions::port::Port;

        lazy_static! {
            static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
                Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
            );
        }

        let mut port: Port<u8> = Port::new(0x60);
        let scancode = unsafe { port.read() };

        let mut keyboard = KEYBOARD.lock();

        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }

        // notify end of interrupt
        unsafe {
            PICS.lock()
                .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8())
        }
    }

    pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
        // use spin::Mutex;
        // use x86_64::instructions::port::PortReadOnly;

        lazy_static! {
            // static ref MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());
        }

        // notify end of interrupt
        unsafe {
            PICS.lock()
                .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8())
        }
    }

    use x86_64::structures::idt::PageFaultErrorCode;
    pub extern "x86-interrupt" fn page_fault_handler(
        stack_frame: &mut InterruptStackFrame,
        error_code: PageFaultErrorCode,
    ) {
        use crate::hlt_loop;
        use x86_64::registers::control::Cr2;
        println!("EXCEPTION: PAGE FAULT");
        println!("Accessed address: {:?}", Cr2::read());
        println!("Error code: {:?}", error_code);
        println!("{:#?}", stack_frame);
        hlt_loop();
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
    Mouse = PIC_2_OFFSET + 4,
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