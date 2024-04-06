# Fast Function Dispatch: Improving the performance of Rust's dynamic function calls

[![crates.io](https://img.shields.io/crates/v/ffd.svg)](https://crates.io/crates/ffd)
[![crates.io](https://docs.rs/ffd/badge.svg)](https://docs.rs/ffd)
[![License](https://img.shields.io/crates/l/ffd.svg)](https://github.com/zesterer/ffd)
[![actions-badge](https://github.com/zesterer/ffd/workflows/Rust/badge.svg?branch=master)](https://github.com/zesterer/ffd/actions)

A safe, pragmatic toolkit for high-performance virtual function calls.

This library provides alternatives to types like `Box<dyn Fn(...) -> _>` that are more performant in a range of
scenarios.

## Feature flags

- `nightly`: Implements `Fn` for `Func`, as well as allowing `Func::new` to accept multi-argument closures

## Why?

You'll often hear it said that Rust is packed full of zero-cost abstractions.

In spirit, this is often true! Many of Rust's fancy features do indeed compile down to machine code that's close enough
to what one might write by hand in a 'low-level' language like C that the differences are fairly meaningless.

Sadly, an exception to this rule is *function dispatch*.

Rust's strategy, upon seeing a trait like the following, and a corresponding `dyn` coercion, is to generate a
[vtable](https://doc.rust-lang.org/nomicon/exotic-sizes.html?highlight=vtable#dynamically-sized-types-dsts).

```ignore
trait MyTrait {
    fn do_something(&self);
    fn do_something_else(&self, x: i32);
}

struct MyStruct { a: i32 }

impl MyTrait for MyStruct {
    fn do_something(&self) { println!("{}", self.a); }
    fn do_something_else(&self, a: i32) { println!("{a}"); }
}
```

The vtable might look something like this:

```ignore
struct MyTraitVtable {
    // `*const ()` represents the `&self` argument of `do_something`
    do_something: fn(*const ()),
    do_something_else: fn(*const (), i32),
}

static MYSTRUCT_MYTRAIT_VTABLE: MyTraitVtable = MyTraitVtable {
    do_something: MyStruct::do_something as fn(_),
    do_something_else: MyStruct::do_something_else as fn(_, _),
};
```

By and large, this is a reasonable strategy: when the compiler sees `&dyn MyTrait`, it'll internally represent this as
wide pointer, somewhat akin to the following tuple:

```ignore
(*const (), *const MyTraitVtable)
```

The first field represents the pointer to the data, `&self`. The second field is the vtable, allowing us to look up
methods at runtime.

When calling a method on the trait object, the compiler will generate code that first dereferences the vtable pointer to
find the vtable, and then selects the field corresponding to the method being invoked. This field is a function pointer:
so we can now call this function pointer using the data pointer as its argument.

This works brilliantly for most traits.

Sadly, Rust also uses the same strategy for dispatching dynamic function calls: the
[`Fn` traits](https://doc.rust-lang.org/std/ops/trait.Fn.html) appear, to Rust, like any other trait. This is
unnecessarily inefficient! The `Fn` trait only has one very commonly invoked method, `Fn::call`: why should we need to
perform *double indirection*, jumping through two locations in memory, when we could just carry the `Fn::call` function
pointer around directly as the pointer metadata? Worse still, this double-indirection can severely pessimise the code
generation of both the caller and callee, trashing register state and requiring unnecessary stack operations.

99% of the time, this relatively tiny inefficiency is of no consequence. However, there
[are circumstances](https://en.wikipedia.org/wiki/Threaded_code) in which this overhead really starts to matter, and it
is for those circumstances that this library exists.

## Planned features

- Covering concurrency use-cases: `Send` and `Sync` functions
- Covering more of the `Fn` traits: `FnMut`, `FnOnce`, etc.
- Different representation strategies: drop function in pointer metadata instead?
