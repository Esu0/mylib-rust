use std::{alloc::Layout, fmt::Debug, mem::MaybeUninit, ptr::NonNull};

pub mod fixed;
pub mod leaking;

/// # Safety
/// * `allocate`関数が返すポインタは`layout`で指定されたサイズのメモリが確保されている
/// * `allocate`関数が返すポインタは`layout`で指定されたアライメントを満たしている
/// * `allocate`関数が返すポインタはユニークである
/// * `allocate`関数が返すポインタは`self`がドロップされるまで有効である
pub unsafe trait MemoryPool {
    type Error;
    fn try_allocate(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error>;

    fn allocate(&self, layout: Layout) -> NonNull<u8>
    where
        Self::Error: Debug,
    {
        self.try_allocate(layout).unwrap()
    }

    fn try_get_uninit_mut<T>(&self) -> Result<&mut MaybeUninit<T>, Self::Error> {
        unsafe {
            self.try_allocate(Layout::new::<MaybeUninit<T>>())
                .map(|ptr| &mut *ptr.cast().as_ptr())
        }
    }

    fn get_uninit_mut<T>(&self) -> &mut MaybeUninit<T>
    where
        Self::Error: Debug,
    {
        self.try_get_uninit_mut().unwrap()
    }

    fn try_get_mut_ptr<T>(&self) -> Result<NonNull<T>, Self::Error> {
        self.try_allocate(Layout::new::<T>()).map(|ptr| ptr.cast())
    }

    fn get_mut_ptr<T>(&self) -> NonNull<T>
    where
        Self::Error: Debug,
    {
        self.try_get_mut_ptr().unwrap()
    }
}
