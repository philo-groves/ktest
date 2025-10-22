use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::{structures::paging::{FrameAllocator, PhysFrame, Size4KiB}, PhysAddr};

static FRAME_ALLOCATOR: OnceCell<Mutex<BumpFrameAllocator>> = OnceCell::uninit();

pub fn get_frame_allocator(boot_info: &'static bootloader_api::BootInfo) -> &'static Mutex<BumpFrameAllocator> {
    FRAME_ALLOCATOR.get_or_init(|| Mutex::new(init_frame_allocator(boot_info)))
}

fn init_frame_allocator(boot_info: &'static bootloader_api::BootInfo) -> BumpFrameAllocator {
    unsafe { BumpFrameAllocator::init(&boot_info.memory_regions) }
}

pub struct BumpFrameAllocator {
    memory_map: &'static MemoryRegions,
    next: usize
}

impl BumpFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryRegions) -> Self {
        BumpFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BumpFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
