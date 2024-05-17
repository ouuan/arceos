use crate::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;
use verified_allocator::{AllocError as VError, BitmapAllocator};

pub struct VerifiedBitmapAllocator(BitmapAllocator);

impl VerifiedBitmapAllocator {
    pub const fn new() -> Self {
        Self(BitmapAllocator::new())
    }
}

fn map_error(ve: VError) -> AllocError {
    match ve {
        VError::InvalidParam => AllocError::InvalidParam,
        VError::MemoryOverlap => AllocError::MemoryOverlap,
        VError::NoMemory => AllocError::NoMemory,
    }
}

impl BaseAllocator for VerifiedBitmapAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.0.unsafe_add_memory(start, size).unwrap();
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        match self.0.unsafe_add_memory(start, size) {
            Ok(()) => Ok(()),
            Err(ve) => Err(map_error(ve)),
        }
    }
}

impl ByteAllocator for VerifiedBitmapAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        match self.0.alloc(layout.size(), layout.align()) {
            Ok((addr, _)) => Ok(unsafe { NonNull::new_unchecked(addr as *mut _) }),
            Err(ve) => Err(map_error(ve)),
        }
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        self.0.unsafe_dealloc(pos.as_ptr() as _, layout.size());
    }

    fn available_bytes(&self) -> usize {
        self.0.available_bytes()
    }

    fn used_bytes(&self) -> usize {
        self.0.used_bytes()
    }

    fn total_bytes(&self) -> usize {
        self.0.total_bytes()
    }
}
