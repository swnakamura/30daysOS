#![no_std]
#![no_main]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(lib::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
extern crate rlibc;

use core::panic::PanicInfo;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use bootloader::{entry_point, BootInfo};
use haribote2 as lib;
use lib::println;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    haribote2::init();

    use haribote2::{allocator, memory};
    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    boot_info
        .memory_map
        .iter()
        .map(|x| println!("{:?}", x))
        .for_each(drop);

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initalization failed");

    let heap_value = Box::new(61);
    println!("heap_value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..100 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_countered = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_countered.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_countered);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        // boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        use x86_64::structures::paging::{Mapper, Page, PhysFrame, Size4KiB};
        let phys: PhysFrame<Size4KiB> = mapper
            .translate_page(Page::containing_address(virt))
            .expect("translation failed");
        println!("{:?} -> {:?}", virt, phys);
    }

    // println!("Let's crash here");
    // let ptr = 0x20493e as *mut u32;
    // unsafe {
    //     let _x = *ptr;
    // }
    // println!("read worked");
    // unsafe {
    //     *ptr = 42;
    // }
    // println!("write worked");

    // // VGA initialization. Doesn't work correctly.
    // haribote2::vga_graphic::init_palette();
    // let mut screen = haribote2::vga_graphic::Screen::new();
    // screen.init();
    // let mut string_writer =
    //     haribote2::vga_graphic::ScreenStringWriter::new(&screen, 0, 0, haribote2::vga_graphic::Color::White);
    // use core::fmt::Write;
    // write!(string_writer, "TEST").unwrap();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    haribote2::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    haribote2::test_panic_handler(info)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    haribote2::hlt_loop()
}
