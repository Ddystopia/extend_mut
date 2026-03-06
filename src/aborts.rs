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
    // Thanks to @robofinch for pointing out that `eprintln!` can unwind.
    let _ = std::panic::catch_unwind(|| eprintln!("{msg}"));
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
        Err(err) => {
            let string = if let Some(s) = err.downcast_ref::<&'static str>() {
                s
            } else if let Some(s) = err.downcast_ref::<String>() {
                s
            } else {
                ""
            };

            let _ = std::panic::catch_unwind(|| {
                if string.is_empty() {
                    eprintln!("ExtendMut: Function cannot unwind");
                } else {
                    eprintln!("ExtendMut: Function cannot unwind, but panicked with message: {string}");
                }
            });

            std::process::abort();
        }
    }
}
