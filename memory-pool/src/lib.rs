use std::{alloc::Layout, fmt::Debug, mem::MaybeUninit, ptr::NonNull};

pub mod fixed;
pub mod leaking;

/// # Safety
/// * `allocate`関数が返すポインタは`layout`で指定されたサイズのメモリが確保されている
/// * `allocate`関数が返すポインタは`layout`で指定されたアライメントを満たしている
/// * `allocate`関数が返すポインタはユニークである
/// * `allocate`関数が返すポインタは`self`がドロップされるまで有効である
pub unsafe trait MemoryPoolAlloc {
    type Error;
    fn try_allocate(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error>;

    fn allocate(&self, layout: Layout) -> NonNull<u8>
    where
        Self::Error: Debug,
    {
        self.try_allocate(layout).unwrap()
    }
}

/// # Safety
/// * `try_get_mut_ptr`関数が返すポインタが適切にアラインメントされている
/// * `try_get_mut_ptr`関数が返すポインタが指すメモリは少なくとも`T`のサイズの領域が確保されている
/// * `try_get_mut_ptr`関数が返すポインタはユニークである
/// * `try_get_mut_ptr`関数が返すポインタは`self`がドロップされるまで有効である
pub unsafe trait MemoryPool<T> {
    type Error;
    fn try_get_mut_ptr(&self) -> Result<NonNull<T>, Self::Error>;
    fn get_mut_ptr(&self) -> NonNull<T>
    where
        Self::Error: Debug,
    {
        self.try_get_mut_ptr().unwrap()
    }

    fn try_get_uninit_mut(&self) -> Result<&mut MaybeUninit<T>, Self::Error> {
        unsafe {
            self.try_get_mut_ptr()
                .map(|ptr| &mut *(ptr.cast().as_ptr()))
        }
    }

    fn get_uninit_mut(&self) -> &mut MaybeUninit<T>
    where
        Self::Error: Debug,
    {
        self.try_get_uninit_mut().unwrap()
    }
}

unsafe impl<P, T> MemoryPool<T> for P
where
    P: MemoryPoolAlloc,
{
    type Error = P::Error;
    fn try_get_mut_ptr(&self) -> Result<NonNull<T>, Self::Error> {
        let layout = Layout::new::<T>();
        let ptr = self.try_allocate(layout)?;
        Ok(ptr.cast())
    }
}
