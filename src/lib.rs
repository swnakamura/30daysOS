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

pub mod allocator;
/// assembly-specific functions
pub mod asm;
/// Unified FIFO buffer
pub mod fifo;
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

        let test_sheet_id = sheet_control.allocate((190, 100)).unwrap();
        sheet_control.change_sheet_height(test_sheet_id, 1);
        sheet_control.sheets[test_sheet_id].make_sheet("counting up...");
        sheet_control.sheets[test_sheet_id].moveto((30, 30));

        sheet_control.refresh_sheet_map(None, None);
        sheet_control.refresh_screen(None, None);

        (background_id, test_sheet_id)
    };

    asm::cli();
    let timer_ticking_id = {
        let mut locked_tc = timer::TIMER_CONTROL.lock();
        // 0.01s x 1000 = 10s
        let timer_10_sec_id = locked_tc.allocate().unwrap();
        locked_tc.set_time(timer_10_sec_id, 1000);
        locked_tc.timers[timer_10_sec_id].data = 10;
        // 0.01s x 300 = 3s
        let timer_3_sec_id = locked_tc.allocate().unwrap();
        locked_tc.set_time(timer_3_sec_id, 300);
        locked_tc.timers[timer_3_sec_id].data = 3;
        // 0.01s x 100 = 1s
        let timer_ticking_id = locked_tc.allocate().unwrap();
        locked_tc.set_time(timer_ticking_id, 100);
        locked_tc.timers[timer_ticking_id].data = 1;
        timer_ticking_id
    };
    asm::sti();

    loop {
        // FIFOバッファは割り込み時にロックされうるので、
        // 2重ロックを防ぐためにcliしてからロックしないといけない
        asm::cli();
        let fifo_buf_pop_result = fifo::GLOBAL_FIFO_BUF.lock().pop();
        asm::sti();

        {
            let mut sheet_control = SHEET_CONTROL.lock();
            let initial_column_position =
                sheet_control.sheets[test_sheet_id].initial_column_position;
            sheet_control.sheets[test_sheet_id].column_position = initial_column_position;
            sheet_control.sheets[test_sheet_id]
                .boxfill(Color::LightGrey, ((3, 23), (3 + 8 * 15, 23 + 16)));

            asm::cli();
            let timer_count = timer::TIMER_CONTROL.lock().count;
            asm::sti();

            write!(
                sheet_control.sheets[test_sheet_id],
                "Uptime:{:>08}",
                timer_count
            )
            .unwrap();
        }

        if let Ok(data) = fifo_buf_pop_result {
            use crate::alloc::string::ToString;
            use core::char::from_u32;
            match data {
                256..=511 => write!(
                    SHEET_CONTROL.lock().sheets[test_sheet_id],
                    "{}",
                    from_u32(data - 256).unwrap().to_string().as_str()
                )
                .unwrap(),
                512..=767 => crate::interrupts::MOUSE
                    .lock()
                    .process_packet((data - 512) as u8),
                10 => write!(
                    SHEET_CONTROL.lock().sheets[test_sheet_id],
                    "\n\n10 secs have passed",
                )
                .unwrap(),
                3 => {
                    write!(
                        SHEET_CONTROL.lock().sheets[test_sheet_id],
                        "\n3 secs have passed",
                    )
                    .unwrap();
                }
                1 | 0 => {
                    asm::cli();
                    if data == 0 {
                        timer::TIMER_CONTROL.lock().timers[timer_ticking_id].data = 1;
                    } else {
                        timer::TIMER_CONTROL.lock().timers[timer_ticking_id].data = 0;
                    }
                    timer::TIMER_CONTROL.lock().set_time(timer_ticking_id, 100);
                    asm::sti();
                    let mut sheet_control = SHEET_CONTROL.lock();
                    sheet_control.sheets[test_sheet_id].boxfill(
                        Color::LightGrey,
                        ((3, 23 + 16 * 3), (3 + 8 * 1, 23 + 16 * 4)),
                    );
                    if data == 0 {
                        write!(sheet_control.sheets[test_sheet_id], "\n\n\nx",).unwrap();
                    } else {
                        write!(sheet_control.sheets[test_sheet_id], "\n\n\ny",).unwrap();
                    }
                }
                _ => panic!("Unexpected value popped from timer fifo"),
            }
        }
        let mut sheet_control = SHEET_CONTROL.lock();
        let test_sheet_height = sheet_control.sheets[test_sheet_id].height as isize;
        // sheet_control.flush_printed_chars(Some(test_sheet_height));
        let test_sheet_area = sheet_control.sheets[test_sheet_id].area();
        sheet_control.refresh_screen(Some(test_sheet_area), Some(test_sheet_height));
    }
}
