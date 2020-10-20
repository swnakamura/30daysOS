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

extern crate alloc;
extern crate rlibc;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

// pub mod vga_graphic;

pub mod allocator;
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
/// text I/O
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
    hlt_loop()
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

/// loops `HLT` instruction
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}
