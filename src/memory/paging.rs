use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::{structures::paging::{OffsetPageTable, PageTable}, VirtAddr};

static KERNEL_PAGE_TABLE: OnceCell<Mutex<OffsetPageTable<'static>>> = OnceCell::uninit();

pub fn get_kernel_page_table(physical_memory_offset: Option<u64>) -> &'static Mutex<OffsetPageTable<'static>> {
    KERNEL_PAGE_TABLE.get_or_init(|| Mutex::new(init_kernel_page_table(physical_memory_offset)))
}

fn init_kernel_page_table(physical_memory_offset: Option<u64>) -> OffsetPageTable<'static> {
    let physical_memory_offset = VirtAddr::new(physical_memory_offset
        .expect("BootInfo is missing physical_memory_offset"));

    unsafe {
        let level_4_table = find_level_4_table_from_bootloader(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

fn find_level_4_table_from_bootloader(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}
