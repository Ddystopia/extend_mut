#[cfg(feature = "std")]
#[cfg(not(test))]
use core::panic::AssertUnwindSafe;

#[cfg(not(feature = "std"))]
#[inline(always)]
pub fn abort_no_unwind(msg: &'static str) -> ! {
    struct DoublePanic(&'static str);
    impl Drop for DoublePanic {
        fn drop(&mut self) {
            panic!("{}", self.0);
        }
    }

    let _double_panic = DoublePanic(msg);
    // If panic=abort, `msg` will be directly delivered to the panic handler, no double panic.
    // If panic=unwind, we will force double panic. This is mostly not needed for no_std.
    panic!("{msg}");
}

#[cfg(feature = "std")]
#[inline(always)]
pub fn abort_no_unwind(msg: &'static str) -> ! {
    eprintln!("{}", msg);
    std::process::abort();
}

#[cfg(not(feature = "std"))]
#[inline(always)]
pub fn abort_on_unwind<T>(f: impl FnOnce() -> T) -> T {
    // If panic=abort, after panic `f` will go directly to the panic handler.
    // If panic=unwind, we will force double panic. This is mostly not needed for no_std.

    struct DoublePanic;
    impl Drop for DoublePanic {
        fn drop(&mut self) {
            panic!("ExtendMut: Function cannot unwind");
        }
    }

    let double_panic = DoublePanic;
    let ret = f();
    core::mem::forget(double_panic);
    ret
}

#[cfg(test)]
pub fn abort_on_unwind<T>(f: impl FnOnce() -> T) -> T {
    f()
}

#[cfg(feature = "std")]
#[cfg(not(test))]
#[inline(always)]
pub fn abort_on_unwind<T>(f: impl FnOnce() -> T) -> T {
    match std::panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(ret) => ret,
        // fixme: how can we print error? It is just `Box<dyn Any + Send>`.
        Err(_err) => abort_no_unwind("ExtendMut: Function cannot unwind"),
    }
}
