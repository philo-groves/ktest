pub use linked_list_allocator::LockedHeap;
use x86_64::{structures::paging::{mapper::MapToError, Size4KiB}};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: u64 = 100 * 1024; // 100 KB

#[global_allocator]
#[cfg(feature = "allocator")]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_allocator_if_enabled(_physical_memory_offset: Option<u64>) -> Result<(), MapToError<Size4KiB>> {
    #[cfg(not(feature = "allocator"))]
    {
        Ok(())
    }

    #[cfg(feature = "allocator")]
    {
        use crate::memory::{frame, paging};
        use x86_64::{structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags}, VirtAddr};

        let page_range = {
            let heap_start = VirtAddr::new(HEAP_START as u64);
            let heap_end = heap_start + HEAP_SIZE - 1u64;

            let heap_start_page = Page::containing_address(heap_start);
            let heap_end_page = Page::containing_address(heap_end);

            Page::range_inclusive(heap_start_page, heap_end_page)
        };

        for page in page_range {
            // important: locks are dropped at the end of each iteration (locks should be as microscopic as possible)
            let mut mapper = paging::get_kernel_page_table(_physical_memory_offset).lock();
            let mut frame_allocator = frame::get_frame_allocator(_physical_memory_offset).lock();

            let frame = frame_allocator.allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            unsafe { mapper.map_to(page, frame, flags, &mut *frame_allocator)?.flush() };
        }

        unsafe { HEAP_ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE as usize); }
        Ok(())
    }
}
