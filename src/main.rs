#![no_std]
#![no_main]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(lib::test_runner)]

extern crate alloc;
extern crate rlibc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
use haribote as lib;
use lib::{println, serial_println};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    haribote::init();

    use haribote::allocator;
    use haribote::memory;
    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    {
        serial_println!("{}", "#".repeat(100));
        serial_println!("Displaying memory regions...");
        boot_info
            .memory_map
            .iter()
            .for_each(|x| serial_println!("{:?}", x));
        serial_println!("{}", "#".repeat(100));

        lib::exit_qemu(lib::QemuExitCode::Success);
    }

    haribote::vga_graphic::graphic_mode();

    println!("It did not crash!");

    haribote::kernel_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    haribote::test_panic_handler(info)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use vga::writers::{Text80x25, TextWriter};
    let textmode = Text80x25::new();
    textmode.set_mode();
    println!("{}", info);
    haribote::hlt_loop();
}
