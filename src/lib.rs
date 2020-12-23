#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(alloc_error_handler)]
#![feature(const_in_array_repeat_expressions)]
#![feature(const_mut_refs)]
#![feature(const_fn_fn_ptr_basics)]

#[macro_use]
extern crate bitflags;

extern crate alloc;
extern crate rlibc;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

// pub mod vga_graphic;

pub mod allocator;
/// assembly-specific functions
pub mod asm;
/// font files
pub mod font;
/// global description table
pub mod gdt;
/// PICs and IDTs for interruptions
pub mod interrupts;
/// memory management
pub mod memory;
/// communicating with serial port
pub mod serial;
/// utility functions
pub mod util;
/// GUI
pub mod vga_graphic;
/// TUI
pub mod vga_text;

/// We use 0x10 as success exit code of test for Qemu.
/// This is configured in package.metadata.bootimage.test-success-exit-code.
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Exit Qemu with given exit code.
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// This function is used for unit tests.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// Test functions hold this trait.
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) -> () {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/// initializes kernel
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe {
        interrupts::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
/// initializes kernel when testing
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    kernel_loop(None)
}

#[cfg(test)]
#[panic_handler]
/// panic handler for test.
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/// When tests panicked, this function is called from `panic` function.
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

pub struct FIFO<T: 'static> {
    buf: &'static mut [T],
    p: usize,
    q: usize,
    size: usize,
    free: usize,
}

pub const KEY_BUF_SIZE: usize = 32;
pub const MOUSE_BUF_SIZE: usize = 1024;

pub static mut KEY_BUF: FIFO<char> = FIFO {
    buf: &mut ['0'; KEY_BUF_SIZE],
    p: 0,
    q: 0,
    free: KEY_BUF_SIZE,
    size: KEY_BUF_SIZE,
};

pub static mut MOUSE_BUF: FIFO<u8> = FIFO {
    buf: &mut [0; MOUSE_BUF_SIZE],
    p: 0,
    q: 0,
    free: MOUSE_BUF_SIZE,
    size: MOUSE_BUF_SIZE,
};

impl<T: Clone> FIFO<T> {
    pub fn push(&mut self, data: T) -> Result<(), ()> {
        if self.free == 0 {
            return Err(());
        }
        self.buf[self.p] = data;
        self.p += 1;
        if self.p == self.size {
            self.p = 0;
        }
        self.free -= 1;
        Ok(())
    }
    pub fn pop(&mut self) -> Result<T, ()> {
        if self.free == self.size {
            return Err(());
        }
        let data = self.buf[self.q].clone();
        self.q += 1;
        if self.q == self.size {
            self.q = 0;
        }
        self.free += 1;
        return Ok(data);
    }
    pub fn status(&self) -> usize {
        return self.size - self.free;
    }
}

/// loops `HLT` instruction
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn kernel_loop() -> ! {
    use core::fmt::Write;
    use vga_graphic::colors256::Color;
    use vga_graphic::WINDOW_CONTROL;
    // use vga_graphic::{MOUSE_ID, SCREEN_HEIGHT, SCREEN_WIDTH};

    let background_id = WINDOW_CONTROL.lock().allocate((150, 100)).unwrap();
    WINDOW_CONTROL.lock().windows[background_id].change_color(Color::White, Color::Cyan);
    WINDOW_CONTROL.lock().change_window_height(background_id, 0);

    let test_window_id = WINDOW_CONTROL.lock().allocate((30, 40)).unwrap();
    WINDOW_CONTROL
        .lock()
        .change_window_height(test_window_id, 1);
    WINDOW_CONTROL.lock().windows[test_window_id].change_color(Color::White, Color::Red);

    write!(WINDOW_CONTROL.lock().windows[background_id], "Hello world!").unwrap();

    WINDOW_CONTROL
        .lock()
        .refresh_screen(Some(((0, 0), (150, 100))));
    loop {
        asm::cli();
        // we assume this is single-threaded as static variables are used here
        unsafe {
            if KEY_BUF.status() != 0 {
                let c = KEY_BUF.pop().unwrap();
                asm::sti();
                use crate::alloc::string::ToString;
                WINDOW_CONTROL.lock().windows[background_id]
                    .write_str(c.to_string().as_str())
                    .unwrap();
                let background_area = WINDOW_CONTROL.lock().windows[background_id].line_area();
                WINDOW_CONTROL.lock().refresh_screen(Some(background_area));
            } else if MOUSE_BUF.status() != 0 {
                let packet = MOUSE_BUF.pop().unwrap();
                asm::sti();
                crate::interrupts::MOUSE.lock().process_packet(packet);
            } else {
                asm::stihlt();
            }
        }
    }
}
