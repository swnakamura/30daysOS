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
use bootloader::entry_point;
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
/// PIT settings
pub mod timer;
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
use bootloader::BootInfo;
pub fn init(boot_info: &'static BootInfo) -> x86_64::VirtAddr {
    // initialize GDT
    gdt::init();

    // set timer interrupt frequency
    timer::init_pit();

    // initialize IDT
    interrupts::init_idt();
    unsafe {
        interrupts::PICS.lock().initialize();
    }

    // initialize memory allocation
    use x86_64::VirtAddr;
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // enable interrupts
    // This should be later than the initialization of memory allocation, since this starts timer
    // interrupt and timer uses FIFO, which internally uses Vec.
    x86_64::instructions::interrupts::enable();

    phys_mem_offset
}

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
/// initializes kernel when testing
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    kernel_loop()
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

use alloc::{vec, vec::Vec};

/// uses static-sized vector as a buffer
#[derive(Debug, Clone)]
pub struct FIFO<T> {
    buf: Vec<T>,
    p: usize,
    q: usize,
    size: usize,
    free: usize,
}

pub const KEY_BUF_SIZE: usize = 32;
pub const MOUSE_BUF_SIZE: usize = 1024;

use lazy_static::lazy_static;
use spin::Mutex;
lazy_static! {
    pub static ref KEY_BUF: Mutex<FIFO<char>> = Mutex::new(FIFO::new(KEY_BUF_SIZE, '0'));
    pub static ref MOUSE_BUF: Mutex<FIFO<u8>> = Mutex::new(FIFO::new(MOUSE_BUF_SIZE, 0));
}

impl<T: Clone> FIFO<T> {
    pub fn new(buf_size: usize, default_value: T) -> Self {
        Self {
            buf: vec![default_value; buf_size],
            p: 0,
            q: 0,
            free: buf_size,
            size: buf_size,
        }
    }
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
    use vga_graphic::SHEET_CONTROL;
    use vga_graphic::{SCREEN_HEIGHT, SCREEN_WIDTH};

    // initialize background and test_sheet
    let (background_id, test_sheet_id) = {
        let mut sheet_control = SHEET_CONTROL.lock();
        let background_id = sheet_control
            .allocate((SCREEN_WIDTH, SCREEN_HEIGHT))
            .unwrap();
        sheet_control.sheets[background_id].change_color(Color::White, Color::Cyan);
        sheet_control.sheets[background_id].make_background();
        sheet_control.change_sheet_height(background_id, 0);

        let test_sheet_id = sheet_control.allocate((160, 68)).unwrap();
        sheet_control.change_sheet_height(test_sheet_id, 1);
        sheet_control.sheets[test_sheet_id].make_sheet("counting up...");
        sheet_control.sheets[test_sheet_id].moveto((30, 30));

        sheet_control.refresh_screen(None, None);
        sheet_control.refresh_sheet_map(None, None);

        (background_id, test_sheet_id)
    };

    let timer_10_sec_id = {
        let mut locked_tc = timer::TIMER_CONTROL.lock();
        // 0.01s x 1000 = 10s
        let timer_id = locked_tc.allocate().unwrap();
        locked_tc.set_time(timer_id, 500);
        timer_id
    };

    loop {
        asm::cli();
        // 先に評価しておかないと、lockが開放されない
        let keybuf_pop_result = KEY_BUF.lock().pop();
        let mousebuf_pop_result = MOUSE_BUF.lock().pop();
        if let Ok(c) = keybuf_pop_result {
            use crate::alloc::string::ToString;
            write!(
                SHEET_CONTROL.lock().sheets[background_id],
                "{}",
                c.to_string().as_str()
            )
            .unwrap();
            asm::sti();
        } else if let Ok(packet) = mousebuf_pop_result {
            crate::interrupts::MOUSE.lock().process_packet(packet);
            asm::sti();
        } else {
            {
                let mut sheet_control = SHEET_CONTROL.lock();
                let initial_column_position =
                    sheet_control.sheets[test_sheet_id].initial_column_position;
                sheet_control.sheets[test_sheet_id].column_position = initial_column_position;
                sheet_control.sheets[test_sheet_id]
                    .boxfill(Color::LightGrey, ((3, 23), (3 + 8 * 15, 23 + 16)));
                write!(
                    sheet_control.sheets[test_sheet_id],
                    "Uptime:{:>08}",
                    timer::TIMER_CONTROL.lock().count
                )
                .unwrap();
                let test_sheet_height = sheet_control.sheets[test_sheet_id].height as isize;
                let test_sheet_area = sheet_control.sheets[test_sheet_id].area();
                {
                    let mut tc_locked = timer::TIMER_CONTROL.lock();
                    if let Ok(_) = tc_locked.timers[timer_10_sec_id].pop() {
                        write!(sheet_control.sheets[test_sheet_id], "10 secs have passed",)
                            .unwrap();
                    }
                }
                asm::sti();
                sheet_control.refresh_screen(Some(test_sheet_area), Some(test_sheet_height));
            }
        }
    }
}
