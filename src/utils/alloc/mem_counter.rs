use std::{
    alloc::{AllocError, GlobalAlloc, Layout},
    ptr::NonNull,
    slice,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

#[derive(Debug)]
struct MemCouterInner<G>
where
    G: GlobalAlloc,
{
    allocated: AtomicUsize,
    allocator: G,
}

// impl<G> Drop for MemCouterInner<G>
// where
//     G: GlobalAlloc,
// {
//     fn drop(&mut self) {
//         if self.allocated.load(Ordering::Relaxed) != 0 {
//             eprintln!("Memory leak detected");
//         }
//     }
// }

#[derive(Debug)]
pub struct MemCountAllocator<G>
where
    G: GlobalAlloc,
{
    inner: Arc<MemCouterInner<G>>,
}

impl<G> Clone for MemCountAllocator<G>
where
    G: GlobalAlloc,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<G> MemCountAllocator<G>
where
    G: GlobalAlloc,
{
    pub fn new(allocator: G) -> Self {
        Self {
            inner: Arc::new(MemCouterInner {
                allocated: AtomicUsize::new(0),
                allocator,
            }),
        }
    }

    pub fn allocated(&self) -> usize {
        self.inner.allocated.load(Ordering::Relaxed)
    }
}

unsafe impl<G> GlobalAlloc for MemCountAllocator<G>
where
    G: GlobalAlloc,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner
            .allocated
            .fetch_add(layout.size(), Ordering::Relaxed);

        unsafe { self.inner.allocator.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner
            .allocated
            .fetch_sub(layout.size(), Ordering::Relaxed);

        unsafe { self.inner.allocator.dealloc(ptr, layout) };
    }
}

unsafe impl<G> std::alloc::Allocator for MemCountAllocator<G>
where
    G: GlobalAlloc,
{
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let ptr = self.alloc(layout);

            match ptr.is_null() {
                true => Err(AllocError),
                false => Ok(NonNull::new_unchecked(slice::from_raw_parts_mut(
                    ptr,
                    layout.size(),
                ))),
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { self.dealloc(ptr.as_ptr(), layout) };
    }
}

#[cfg(test)]
mod test {
    use super::MemCountAllocator;

    #[test]
    fn alloc() {
        let allocator = MemCountAllocator::new(mimalloc_rust::GlobalMiMalloc);

        let mut vec =
            Vec::<i32, MemCountAllocator<mimalloc_rust::GlobalMiMalloc>>::with_capacity_in(
                10,
                allocator.clone(),
            );

        assert_eq!(vec.capacity(), 10);
        assert_eq!(allocator.allocated(), 10 * std::mem::size_of::<i32>());

        for i in 0..1000 {
            vec.push(i);
        }

        assert_eq!(
            allocator.allocated(),
            vec.capacity() * std::mem::size_of::<i32>()
        );

        drop(vec);

        assert_eq!(allocator.allocated(), 0);
    }
}
