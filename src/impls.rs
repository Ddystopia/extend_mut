/*!

This module contains implementations for helper traits

IntoExtendMutReturn:
No impl for IntoExtendMutReturn<(&mut T, &mut T), ()>
*/

use crate::{extend_mut, ExtendMut, IntoExtendMutReturn};

#[cfg(feature = "assume-non-forget")]
use crate::extend_mut_async;

// #![feature(generic_const_exprs)]
// trait NotZst: Sized {}
// impl<T> NotZst for T where [(); size_of::<T>() - 1]: Sized {}

macro_rules! impl_into_extend_mut {
    (unit: $head:ident,) => {
        unsafe impl<'a, $head: ?Sized> IntoExtendMutReturn<(&'a mut $head,), ()> for (&'a mut $head,) {
            #[inline(always)]
            fn into_extend_mut_return(self) -> ((&'a mut $head,), ()) { (self, ()) }
        }
    };
    (unit: $head:ident, $($param:ident,)*) => {
        unsafe impl<'a, $head: ?Sized, $($param,)*> IntoExtendMutReturn<(&'a mut $head, $(&'a mut $param,)*), ()>
            for (&'a mut $head, $(&'a mut $param,)*)
        {
            #[inline(always)]
            fn into_extend_mut_return(self) -> ((&'a mut $head, $(&'a mut $param,)*), ()) { (self, ()) }
        }
        impl_into_extend_mut!(unit: $($param,)*);
    };
    (any: $head:ident,) => {
        unsafe impl<'a, $head: ?Sized, R> IntoExtendMutReturn<(&'a mut $head,), R> for ((&'a mut $head,), R) {
            #[inline(always)]
            fn into_extend_mut_return(self) -> ((&'a mut $head,), R) { self }
        }
    };
    (any: $head:ident, $($param:ident,)*) => {
        unsafe impl<'a, $head: ?Sized, $($param: ?Sized,)* R> IntoExtendMutReturn<(&'a mut $head, $(&'a mut $param,)*), R>
            for ((&'a mut $head, $(&'a mut $param,)*), R)
        {
            #[inline(always)]
            fn into_extend_mut_return(self) -> ((&'a mut $head, $(&'a mut $param,)*), R) { self }
        }
        impl_into_extend_mut!(any: $($param,)*);
    };
}

macro_rules! impl_extend_mut_many {
    ($head:ident,) => {
        #[allow(non_snake_case)]
        impl<'a, 'b, $head: ?Sized + 'b> ExtendMut<'b> for (&'a mut $head,) {
            type Extended = (&'b mut $head,);
            #[inline(always)]
            fn extend_mut<R, ER: IntoExtendMutReturn<Self::Extended, R>>(
                self,
                f: impl FnOnce(Self::Extended) -> ER,
            ) -> R {
                extend_mut(self.0, #[inline(always)] |x| {
                    let ((x,), r) = f((x,)).into_extend_mut_return();
                    (x, r)
                })
            }
            #[cfg(feature = "assume-non-forget")]
            #[inline(always)]
            async fn extend_mut_async<R, ER: IntoExtendMutReturn<Self::Extended, R>>(
                self,
                f: impl AsyncFnOnce(Self::Extended) -> ER,
            ) -> R {
                extend_mut_async(self.0, #[inline(always)] async |x| {
                    let ((x,), r) = f((x,)).await.into_extend_mut_return();
                    (x, r)
                }).await
            }
        }
    };
    ($head:ident, $($param:ident,)*) => {
        #[allow(non_snake_case)]
        impl <'a, 'b, $head: ?Sized + 'b, $($param: ?Sized + 'b,)*> ExtendMut<'b> for (&'a mut $head, $(&'a mut $param,)*) {
            type Extended = (&'b mut $head, $(&'b mut $param,)*);
            #[inline(always)]
            fn extend_mut<R, ER: IntoExtendMutReturn<Self::Extended, R>>( self, f: impl FnOnce(Self::Extended) -> ER,) -> R {
                let (x, $($param,)*) = self;
                extend_mut(x, #[inline(always)] |x| {
                    ($($param,)*).extend_mut(#[inline(always)] |($($param,)*)| {
                        let ((x, $($param,)*), r) = f((x, $($param,)*)).into_extend_mut_return();
                        (($($param,)*), (x, r))
                    })
                })
            }
            #[cfg(feature = "assume-non-forget")]
            #[inline(always)]
            async fn extend_mut_async<R, ER: IntoExtendMutReturn<Self::Extended, R>>( self, f: impl AsyncFnOnce(Self::Extended) -> ER,) -> R {
                let (x, $($param,)*) = self;
                extend_mut_async(x, #[inline(always)] async |x| {
                    ($($param,)*).extend_mut_async(#[inline(always)] async |($($param,)*)| {
                        let ((x, $($param,)*), r) = f((x, $($param,)*)).await.into_extend_mut_return();
                        (($($param,)*), (x, r))
                    }).await
                }).await
            }
        }
        impl_extend_mut_many!($($param,)*);
    };
}

unsafe impl<'a, T: ?Sized, R> IntoExtendMutReturn<&'a mut T, R> for (&'a mut T, R) {
    #[inline(always)]
    fn into_extend_mut_return(self) -> (&'a mut T, R) {
        self
    }
}

unsafe impl<'a, T: ?Sized> IntoExtendMutReturn<&'a mut T, ()> for &'a mut T {
    #[inline(always)]
    fn into_extend_mut_return(self) -> (&'a mut T, ()) {
        (self, ())
    }
}

impl_into_extend_mut!(any: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,);
impl_into_extend_mut!(unit: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,);

impl<'a, 'b, T: ?Sized + 'b> ExtendMut<'b> for &'a mut T {
    type Extended = &'b mut T;
    #[inline(always)]
    fn extend_mut<R, ER: IntoExtendMutReturn<Self::Extended, R>>(
        self,
        f: impl FnOnce(Self::Extended) -> ER,
    ) -> R {
        extend_mut(self, f)
    }
    #[cfg(feature = "assume-non-forget")]
    #[inline(always)]
    async fn extend_mut_async<R, ER: IntoExtendMutReturn<Self::Extended, R>>(
        self,
        f: impl AsyncFnOnce(Self::Extended) -> ER,
    ) -> R {
        extend_mut_async(self, f).await
    }
}

impl<'b> ExtendMut<'b> for () {
    type Extended = ();
    #[inline(always)]
    fn extend_mut<R, ER: IntoExtendMutReturn<Self::Extended, R>>(
        self,
        f: impl FnOnce(Self::Extended) -> ER,
    ) -> R {
        f(()).into_extend_mut_return().1
    }
    #[cfg(feature = "assume-non-forget")]
    #[inline(always)]
    async fn extend_mut_async<R, ER: IntoExtendMutReturn<Self::Extended, R>>(
        self,
        f: impl AsyncFnOnce(Self::Extended) -> ER,
    ) -> R {
        f(()).await.into_extend_mut_return().1
    }
}

impl_extend_mut_many!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,);
