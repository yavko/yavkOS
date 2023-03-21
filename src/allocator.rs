#[cfg(feature = "alloc-bump")]
pub mod bump;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[cfg_attr(feature = "alloc-lla", global_allocator)]
#[cfg(feature = "alloc-lla")]
#[cfg(not(feature = "alloc-bump"))]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

#[cfg_attr(feature = "alloc-bump", global_allocator)]
#[cfg(feature = "alloc-bump")]
#[cfg(not(feature = "alloc-galloc"))]
static ALLOCATOR: Locked<bump::BumpAllocator> = Locked::new(bump::BumpAllocator::new());

#[cfg_attr(feature = "alloc-galloc", global_allocator)]
#[cfg(feature = "alloc-galloc")]
static ALLOCATOR: good_memory_allocator::SpinLockedAllocator =
    good_memory_allocator::SpinLockedAllocator::empty();

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
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }
    unsafe {
        #[cfg(feature = "alloc-bump")]
        #[cfg(not(any(feature = "alloc-galloc", feature = "alloc-lla")))]
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
        #[cfg(feature = "alloc-lla")]
        #[cfg(not(any(feature = "alloc-galloc", feature = "alloc-bump")))]
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
        #[cfg(feature = "alloc-galloc")]
        ALLOCATOR.init(HEAP_START, HEAP_SIZE)
    }
    Ok(())
}

#[repr(transparent)]
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
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
