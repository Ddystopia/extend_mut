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
  async context, if you will call it on every poll, instead of on future creation.
- **`extend_mut_async`**: An asynchronous function that allows extending the
  lifetime of a mutable reference in an `async` context. This function comes
  with important safety considerations.

## Why Use `extend_mut`?

Rust's borrow checker enforces strict lifetime rules to ensure memory safety.
However, there are scenarios where you may need to work around lifetime
limitations in a controlled way. `extend_mut` provides a way to extend the
lifetime of mutable references safely and correctly, without introducing
undefined behavior.

A commom use case is creating a temporary `&'static mut` reference to send
somewhere, give execution control to other function that expects `&'static mut`,
and then take back the control, take back `&'static mut` and recover original lifetime.

## Crate Attributes

- `#![no_std]` support: This crate is compatible with `#![no_std]` environments,
  making it suitable for embedded and constrained systems.

### Synchronous Example

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

## Safety Considerations

`extend_mut` is designed to be safe, while `extend_mut_async` is inherently
unsafe due to the lack of linear types in Rust. When using `extend_mut_async`,
ensure that the returned future is fully awaited before being dropped.


