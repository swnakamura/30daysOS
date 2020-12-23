use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

/// a wrapper around spin::Mutex to permit trait immplmentations
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Self {
            inner: spin::Mutex::new(inner),
        }
    }
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

pub mod fixed_size_block;

pub mod bump {
    use alloc::alloc::{GlobalAlloc, Layout};
    use core::ptr;
    pub struct BumpAllocator {
        heap_start: usize,
        heap_end: usize,
        next: usize,
        allocations: usize,
    }

    impl BumpAllocator {
        pub const fn new() -> Self {
            Self {
                heap_start: 0,
                heap_end: 0,
                next: 0,
                allocations: 0,
            }
        }
        pub fn init(&mut self, heap_start: usize, heap_size: usize) {
            self.heap_start = heap_start;
            self.heap_end = heap_start + heap_size;
            self.next = heap_start;
        }
    }
    use super::{align_up, Locked};

    unsafe impl GlobalAlloc for Locked<BumpAllocator> {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let mut bump = self.lock();
            let alloc_start = align_up(bump.next, layout.align());
            let alloc_end = match alloc_start.checked_add(layout.size()) {
                Some(val) => val,
                None => return ptr::null_mut(),
            };
            if alloc_end > bump.heap_end {
                ptr::null_mut()
            } else {
                bump.next = alloc_end;
                bump.allocations += 1;
                alloc_start as *mut u8
            }
        }
        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
            let mut bump = self.lock();
            bump.allocations -= 1;
            if bump.allocations == 0 {
                bump.next = bump.heap_start;
            }
        }
    }
}

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 1000 * 1024;

use fixed_size_block::FixedSizeBlockAllocator;
#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());
// static ALLOCATOR: Locked<bump::BumpAllocator> = Locked::new(bump::BumpAllocator::new());

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    Ok(())
}
