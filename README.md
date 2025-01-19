# extend_mut

[![docs.rs](https://docs.rs/extend_mut/badge.svg)](https://docs.rs/extend_mut)
[![crates.io](https://img.shields.io/crates/v/extend_mut.svg)](https://crates.io/crates/extend_mut)

<!-- [![crates.io](https://img.shields.io/crates/d/extend_mut.svg)](https://crates.io/crates/extend_mut) -->

`extend_mut` is a `#![no_std]` Rust crate that provides safe and unsafe
utilities to extend the lifetime of an exclusive mutable reference (`&mut`). It
includes both synchronous and asynchronous methods for achieving this, with a
focus on correctness and safety guarantees around mutable reference lifetimes.

## API

- **`extend_mut`**: A synchronous function that safely extends the lifetime of a
  mutable reference using a sync closure. Note that you can still use this in
  async context, if you will call it on every poll, instead of on future
  creation.
- **`extend_mut_async`**: An asynchronous function that allows extending the
  lifetime of a mutable reference in an `async` context. This function comes
  with important safety considerations.
- **`ExtendMut`**: A trait for `expr.extend_mut` syntax.

## Usage

Following snippet demonstrates the usage of `extend_mut` and `ExtendMut` trait:

```rust
let (mut t1, mut t2, mut t3, mut t4) = (1, 2, 3, 4);

let () = extend_mut(&mut t1, |t1: &'static mut u8| t1);
let "hi" = extend_mut(&mut t1, |t1| (t1, "hi")) else { panic!() };
let () = extend_mut(&mut t1, |t1| (t1, ()));

let () = t1.extend_mut(|t1| t1);
let () = (&mut t1).extend_mut(|t1: &'static mut u8| t1);
let "hi" = t1.extend_mut(|t1| (t1, "hi")) else { panic!() };
let "hi" = (&mut t1).extend_mut(|t1| (t1, "hi")) else { panic!() };

let () = (t1, t2).extend_mut(|it: &'static mut (u8, u8)| it);
let () = (&mut t1, &mut t2).extend_mut(|it: (&'static mut u8, &'static mut u8)| it);
let () = (&mut (t1, t2)).extend_mut(|it: &'static mut (u8, u8)| it);
let "hi" = (t1, t2).extend_mut(|it| (it, "hi")) else { panic!() };
let "hi" = (&mut t1, &mut t2).extend_mut(|it| (it, "hi")) else { panic!() };
let "hi" = (&mut (t1, t2)).extend_mut(|it| (it, "hi")) else { panic!() };

let () = (t1, t2, t3).extend_mut(|it: &'static mut (u8, u8, u8)| it);
let () = (&mut t1, &mut t2, &mut t3).extend_mut(|it: (&'static mut u8, &'static mut u8, &mut u8)| it);
let "hi" = (t1, t2, t3).extend_mut(|it| (it, "hi")) else { panic!() };
let "hi" = (&mut t1, &mut t2, &mut t3).extend_mut(|it| (it, "hi")) else { panic!() };

let () = (t1, t2, t3, t4).extend_mut(|it: &'static mut (u8, u8, u8, u8)| it);
let () = (&mut t1, &mut t2, &mut t3, &mut t4).extend_mut(|it: (&mut u8, &mut u8, &mut u8, &mut u8)| it);
let "hi" = (t1, t2, t3, t4).extend_mut(|it| (it, "hi")) else { panic!() };
let "hi" = (&mut t1, &mut t2, &mut t3, &mut t4).extend_mut(|it| (it, "hi")) else { panic!() };

let () = <_>::extend_mut(&mut (t1, t2, t3, t4), |it: &'static mut (u8, u8, u8, u8)| it);
let () = <_>::extend_mut((&mut t1, &mut t2, &mut t3, &mut t4), |it| it);
```

## Why Use `extend_mut`?

Rust's borrow checker enforces strict lifetime rules to ensure memory safety.
However, there are scenarios where you may need to temporarily bypass these
strict lifetime constraints in a controlled and safe manner. `extend_mut` offers
a way to extend the lifetime of mutable references without introducing undefined
behavior, allowing greater flexibility in certain advanced use cases.

One common scenario involves temporarily creating a `&'static mut` reference to
pass to a function or API that requires it, handing over control during the
function call, and then reclaiming control and restoring the original reference
with its proper lifetime. This approach avoids the need to introduce additional
wrappers or abstractions like `RefCell`.

`extend_mut` can also serve as an alternative to
[`StaticCell`](https://docs.rs/static_cell/latest/static_cell/struct.StaticCell.html#).
The advantage is that you can obtain a `&'static mut T` for types that are not
easily named, such as the result of an `async fn` or an `async {}` block,
without requiring a type alias impl trait (TAIT). The tradeoff, however, is that
the value of `T` will be allocated on the stack rather than in a static
linker-allocated region.

## Crate Attributes

- `#![no_std]` support: This crate is compatible with `#![no_std]` environments,
  making it suitable for embedded and constrained systems.

## Synchronous Example

```rust
use extend_mut::extend_mut;

fn main() {
    let mut x = 5;

    fn modify_static(x: &'static mut i32) -> &'static mut i32 {
        *x += 1;
        x
    }

    extend_mut(&mut x, |x| modify_static(x));
    assert_eq!(x, 6);

    extend_mut(&mut x, modify_static);
    assert_eq!(x, 7);

    let result = extend_mut(&mut x, |x| (modify_static(x), 42));

    assert_eq!(result, 42);
    assert_eq!(x, 8);
}
```

## Asynchronous Example

```rust
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
        Poll::Pending => unreachable!(), // `want_static` will not return pending
    }
};

assert_eq!(ret, 8);
```

## Safety Considerations

`extend_mut` is designed to be safe, while `extend_mut_async` is inherently
unsafe due to the lack of linear types in Rust. When using `extend_mut_async`,
ensure that the returned future is fully awaited before being dropped.
