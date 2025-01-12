#![cfg_attr(not(test), no_std)]

/*!

This crate provides a safe way to extend the lifetime of a exclusive reference.

[`extend_mut`] allows for safe extension of the lifetime of a exclusive reference
with a blocking closure.

[`extend_mut_async`] is similar to [`extend_mut`], but it is async and requires
a linear type be safe - but Rust does not have linear types yet, so it is unsafe.

*/

use core::{
    future::Future,
    pin::Pin,
    ptr,
    task::{Context, Poll},
};

// With `panic=abort` it will directly go to panic handler without unwind.
// With `panic=unwind` it will painc-in-drop, which will cause panic_nounwind.
fn abort_no_unwind(msg: &'static str) -> ! {
    struct DoublePanic(&'static str);

    impl Drop for DoublePanic {
        fn drop(&mut self) {
            panic!("{}", self.0);
        }
    }

    let _double_panic = DoublePanic(msg);
    panic!("{msg}");
}

// SAFETY:
//     if `'a` is >= `'b`, then is is safe by [extend_mut_proof_for_smaller] proof.
//     if `f` will diverge, `'a` will be `'static`, which is valid.
//     if `f` will return `&'b mut T` back, then `'a` will be large enough to fit this call.
//         That way, `&'b mut T` will not exist for `'b`, but only for `'a`.
//
//     if `f` stored `&'b mut T`, then
//         if `f` diverged, it is fine, because `'a` becomes `'static`.
//         else `f` must return `&'b mut T` different from the one it stored.
//           we verify it by an assertion.
//     else we know that `f` did not store the reference we gave it, so it is sound.

/// Extends the lifetime of a mutable reference. Note that `f` must return the same reference
/// that was passed to it, otherwise it will abort the process.
pub fn extend_mut<'a, 'b, T: 'b, R>(
    mut_ref: &'a mut T,
    f: impl FnOnce(&'b mut T) -> (&'b mut T, R),
) -> R {
    let ptr = ptr::from_mut(mut_ref);
    let (extended, next) = f(unsafe { &mut *ptr });
    if ptr != ptr::from_mut(extended) {
        abort_no_unwind("ExtendMut: Pointer changed");
    }

    next
}

pin_project_lite::pin_project! {
    /// Future returned by returned by [extend_mut_async].
    /// Consult it's documentation for more information and safety requirements.
    pub struct ExtendMutFuture<'a, T, Fut, R> {
        ptr: *mut T,
        marker: core::marker::PhantomData<(&'a mut T, R)>,
        #[pin]
        future: Fut,
        // Instead of having that bool, we might make `ptr` null.
        ready: bool,
    }

    impl<'a, T, Fut, R> PinnedDrop for ExtendMutFuture<'a, T, Fut, R> {
        fn drop(this: Pin<&mut Self>) {
            if !*this.project().ready {
                abort_no_unwind("Cannot drop ExtendMutFuture before it yields Poll::Ready");
            }
        }
    }
}

impl<'a, T, Fut, R> Future for ExtendMutFuture<'a, T, Fut, R>
where
    Fut: Future<Output = (&'a mut T, R)>,
{
    type Output = R;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let ptr = *this.ptr;

        if *this.ready {
            return Poll::Pending;
        }

        match this.future.poll(cx) {
            Poll::Ready((extended, ret)) => {
                if ptr == ptr::from_mut(extended) {
                    *this.ready = true;
                    Poll::Ready(ret)
                } else {
                    abort_no_unwind("ExtendMut: Pointer changed")
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Async version of [`extend_mut`]. You should not drop the future returned by [`extend_mut_async`]
/// until it yields [`Poll::Ready`] - if you do, it will abort the process. This function is *not*
/// cancel-safe.
///
/// If polled after yielding [`Poll::Ready`], it will always return [`Poll::Pending`].
///
/// # Safety
///
/// Shortly - do not cancel returned future.
///
/// You must not skip abortion on dropping the future returned by [`extend_mut_async`]
/// by any means, including [forget](core::mem::forget), [`ManuallyDrop`](core::mem::ManuallyDrop) etc. Otherwise,
/// borrow checker will allow you to use `mut_ref` while it might be used by `f`, which will
/// be undefined behavior.
pub unsafe fn extend_mut_async<'a, 'b, T: 'b, F, Fut, R>(
    mut_ref: &'a mut T,
    f: F,
) -> ExtendMutFuture<'b, T, Fut, R>
where
    Fut: Future<Output = (&'b mut T, R)>,
    F: FnOnce(&'b mut T) -> Fut,
{
    let ptr = ptr::from_mut(mut_ref);
    let future = f(unsafe { &mut *ptr });

    ExtendMutFuture {
        ptr,
        marker: core::marker::PhantomData,
        future,
        ready: false,
    }
}

#[allow(dead_code)]
fn extend_mut_proof_for_smaller<'a: 'b, 'b, T: 'b, R>(
    mut_ref: &'a mut T,
    f: impl FnOnce(&'b mut T) -> (&'b mut T, R),
) -> R {
    f(mut_ref).1
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extend_mut() {
        let mut x = 5;

        fn want_static(x: &'static mut i32) -> &'static mut i32 {
            assert_eq!(*x, 5);
            *x += 1;
            *x += 1;
            x
        }

        let r = extend_mut(&mut x, |x| (want_static(x), 6));
        assert_eq!(r, 6);
        assert_eq!(x, 7);
    }

    #[test]
    fn test_extend_mut_async_immediate() {
        use core::pin::pin;
        use core::task::{Context, Poll, Waker};

        let mut x = 5;
        async fn want_static(x: &'static mut i32) -> &'static mut i32 {
            assert_eq!(*x, 5);
            x
        }

        let fut = unsafe { extend_mut_async(&mut x, async |x| (want_static(x).await, 8)) };
        let mut fut = pin!(fut);
        let ret = loop {
            match fut.as_mut().poll(&mut Context::from_waker(&Waker::noop())) {
                Poll::Ready(ret) => break ret,
                Poll::Pending => panic!(),
            }
        };

        assert_eq!(ret, 8);
    }

    #[test]
    fn test_extend_mut_async_yielding() {
        use core::pin::pin;
        use core::task::{Context, Poll, Waker};

        let mut x = 5;

        async fn want_static(x: &'static mut i32) -> &'static mut i32 {
            let mut i = 0;

            let yield_fn = core::future::poll_fn(|cx| {
                *x += 1;

                if i == 20 {
                    return Poll::Ready(());
                } else {
                    i += 1;
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
            });

            yield_fn.await;

            x
        }

        let fut = unsafe { extend_mut_async(&mut x, async |x| (want_static(x).await, 8)) };
        let mut fut = pin!(fut);
        let ret = loop {
            match fut.as_mut().poll(&mut Context::from_waker(&Waker::noop())) {
                Poll::Ready(ret) => break ret,
                Poll::Pending => continue,
            }
        };

        assert_eq!(ret, 8);
        assert_eq!(x, 26);
    }
}
