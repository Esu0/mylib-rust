pub fn take_mut<T>(v: &mut T, change: impl FnOnce(T) -> T) {
    replace(v, |value| (change(value), ()))
}

pub fn replace<T, R>(v: &mut T, change: impl FnOnce(T) -> (T, R)) -> R {
    struct PanicGuard;
    impl Drop for PanicGuard {
        fn drop(&mut self) {
            std::process::abort()
        }
    }
    let guard = PanicGuard;
    let value = unsafe { std::ptr::read(v) };
    let (new_value, ret) = change(value);
    unsafe {
        std::ptr::write(v, new_value);
    }
    std::mem::forget(guard);
    ret
}
