use crate::gdt;
use crate::println;
use crate::println_graphic;
use crate::util::clip;
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use ps2_mouse::{Mouse, MouseState};
use spin::Mutex;
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
        // mouse
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
        // do nothing

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
                unsafe {
                    crate::KEY_BUF.push(match key {
                        DecodedKey::Unicode(character) => character,
                        DecodedKey::RawKey(_) => '?',
                    });
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
        use x86_64::instructions::port::PortReadOnly;

        let mut port = PortReadOnly::new(0x60);
        let packet = unsafe { port.read() };
        // we assume this is single-threaded as static variables are used here
        unsafe {
            crate::MOUSE_BUF.push(packet);
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
} /* handler */

lazy_static! {
    pub static ref MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());
}

pub fn init_idt() {
    MOUSE.lock().init().unwrap();
    MOUSE.lock().set_on_complete(on_complete);
    IDT.load();
}

#[derive(Debug)]
struct CursorState {
    x: isize,
    y: isize,
}

static mut CURSOR_STATE: Mutex<CursorState> = Mutex::new(CursorState { x: 0, y: 0 });

fn on_complete(mouse_state: MouseState) {
    // assume this is single-threaded as we use spinlock here
    unsafe {
        let prev_x = CURSOR_STATE.lock().x;
        let prev_y = CURSOR_STATE.lock().y;
        CURSOR_STATE.lock().x += mouse_state.get_x() as isize;
        CURSOR_STATE.lock().y -= mouse_state.get_y() as isize;
        use crate::vga_graphic::{SCREEN_HEIGHT, SCREEN_WIDTH};
        let xmove = clip(CURSOR_STATE.lock().x, 0, SCREEN_WIDTH);
        CURSOR_STATE.lock().x = xmove;
        let ymove = clip(CURSOR_STATE.lock().y, 0, SCREEN_HEIGHT);
        CURSOR_STATE.lock().y = ymove;
        // println_graphic!("{:?}", CURSOR_STATE);
        use vga::colors::Color16;
        let x = CURSOR_STATE.lock().x;
        let y = CURSOR_STATE.lock().y;
        crate::vga_graphic::draw_mouse(&(x, y), &(prev_x, prev_y), &Color16::Black);
    }
}

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 0x08;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 0x01,
    Mouse = PIC_2_OFFSET + 0x04,
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
