use core::alloc::Layout;
use core::ptr::NonNull;
use linked_list_allocator::Heap;
use crate::{AllocError, AllocResult, BaseAllocator, ByteAllocator};

pub struct NewByteAllocator {
    inner: Heap,
}

impl NewByteAllocator {
    pub const fn new() -> Self {
        Self {
            inner: Heap::empty(),
        }
    }
}

impl BaseAllocator for NewByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        unsafe { self.inner.init(start as *mut u8, size); }
    }

    fn add_memory(&mut self, _start: usize, size: usize) -> AllocResult {
        unsafe { self.inner.extend(size); }
        Ok(())
    }
}

impl ByteAllocator for NewByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        self.inner
            .allocate_first_fit(layout)
            .map_err(|_| AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        unsafe { self.inner.deallocate(pos, layout); }
    }

    fn total_bytes(&self) -> usize {
        self.inner.size()
    }

    fn used_bytes(&self) -> usize {
        self.inner.used()
    }

    fn available_bytes(&self) -> usize {
        self.inner.free()
    }
}