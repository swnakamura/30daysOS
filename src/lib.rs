#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate rlibc;

use core::panic::PanicInfo;

pub mod interrupts;
pub mod serial;
pub mod vga_text;

pub mod gdt {
    use lazy_static::lazy_static;
    use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
    use x86_64::structures::tss::TaskStateSegment;
    use x86_64::VirtAddr;

    pub const DOUBLE_FAULT_DEFAULT_IST_INDEX: u16 = 0;

    lazy_static! {
        static ref TSS: TaskStateSegment = {
            let mut tss = TaskStateSegment::new();
            tss.interrupt_stack_table[DOUBLE_FAULT_DEFAULT_IST_INDEX as usize] = {
                const STACK_SIZE: usize = 4096 * 5;
                static mut STACK: [usize; STACK_SIZE] = [0; STACK_SIZE];

                let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                let stack_end = stack_start + STACK_SIZE;
                stack_end
            };
            tss
        };
    }

    lazy_static! {
        static ref GDT: (GlobalDescriptorTable, Selectors) = {
            let mut gdt = GlobalDescriptorTable::new();
            gdt.add_entry(Descriptor::kernel_code_segment());
            gdt.add_entry(Descriptor::tss_segment(&TSS));
            let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
            let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
            (
                gdt,
                Selectors {
                    code_selector,
                    tss_selector,
                },
            )
        };
    }

    struct Selectors {
        code_selector: SegmentSelector,
        tss_selector: SegmentSelector,
    }

    pub fn init() {
        use x86_64::instructions::segmentation::set_cs;
        use x86_64::instructions::tables::load_tss;
        GDT.0.load();
        unsafe {
            set_cs(GDT.1.code_selector);
            load_tss(GDT.1.tss_selector);
        }
    }
} /* gdt */

pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

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

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe {
        interrupts::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}
