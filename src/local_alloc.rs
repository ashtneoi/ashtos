use core::alloc::Layout;
use core::convert::TryInto;
use core::ops::{Deref, DerefMut, Drop};
use core::mem;

// TODO: Find an existing trait for this on crates.io if it exists.

// TODO: Is there any reason for this to be so unsafe?
/// EXPERIMENTAL
pub unsafe trait LocalAllocator {
    // TODO: `None` is supremely unhelpful.
    unsafe fn alloc_raw(&self, layout: Layout) -> Option<*mut u8>;
    unsafe fn dealloc_raw(&self, ptr: *mut u8, layout: Layout);
}

pub struct LocalBox<'a, T> {
    allocator: &'a mut dyn LocalAllocator,
    inner: *mut T,
}

impl<'a, T> Deref for LocalBox<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl<'a, T> DerefMut for LocalBox<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

impl<'a, T> Drop for LocalBox<'a, T> {
    fn drop(&mut self) {
        let layout = unsafe {
            // Valid because `Layout::from_size_align()` returned `Ok` earlier.
            Layout::from_size_align_unchecked(
                mem::size_of::<T>(),
                mem::align_of::<T>(),
            )
        };
        unsafe {
            self.allocator.dealloc_raw(self.inner as *mut u8, layout);
        }
    }
}

pub struct SingleLocalAllocator<'a> {
    mem: &'a mut [u8],
}

impl<'a> SingleLocalAllocator<'a> {
    // TODO: `AsRef`?
    pub fn new(mem: &'a mut [u8]) -> SingleLocalAllocator<'a> {
        Self { mem }
    }

    // TODO: `Result<_, ()>` isn't very useful.
    /// EXPERIMENTAL
    pub fn alloc<'b, T>(&'b mut self, t: T) -> Result<LocalBox<'b, T>, ()> {
        let layout = match Layout::from_size_align(
                mem::size_of::<T>(),
                mem::align_of::<T>(),
        ) {
            Ok(x) => x,
            Err(_) => return Err(()),
        };
        let mut local_box = unsafe {
            // Valid because `Layout::from_size_align()` checked all the
            // requirements assumed by `LocalAllocator::alloc_raw()`.
            match self.alloc_raw(layout) {
                Some(x) => LocalBox { allocator: self, inner: x as *mut T },
                None => return Err(()),
            }
        };
        *local_box = t;
        Ok(local_box)
    }

    // TODO: Maybe we can allocate many of the same type? Shouldn't be much
    // extra work, just a new param and some more trait impls.
}

unsafe impl<'a> LocalAllocator for SingleLocalAllocator<'a> {
    unsafe fn alloc_raw(&self, layout: Layout) -> Option<*mut u8> {
        let unaligned_base = self.mem as *const [u8] as *mut u8;
        let offset = unaligned_base.align_offset(layout.align());
        let offset_isize: isize = match offset.try_into() {
            Ok(x) => x,
            Err(_) => return None,
        };
        if offset + layout.size() <= self.mem.len() {
            Some(unaligned_base.wrapping_offset(offset_isize))
        } else {
            None
        }
    }

    unsafe fn dealloc_raw(&self, _ptr: *mut u8, _layout: Layout) { }
}
