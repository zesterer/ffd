#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg), deny(rustdoc::all))]
#![cfg_attr(feature = "nightly", feature(unboxed_closures, tuple_trait, fn_traits))]
#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::{marker::PhantomData, mem::MaybeUninit};

/// A boxed function.
///
/// You can see this as semantically equivalent to a `Box<dyn Fn<T, Output = O>>`.
///
/// With the `nightly` feature enabled, this type implements [`Fn`].
pub struct Func<'a, T, O> {
    f: unsafe fn(*const (), T) -> O,
    data: *mut (),
    phantom: PhantomData<&'a ()>,
}

#[repr(C)]
struct WithDrop<F> {
    drop_fn: unsafe fn(*mut ()),
    f: F,
}

impl<'a, T, O> Drop for Func<'a, T, O> {
    fn drop(&mut self) {
        unsafe {
            let drop_fn = core::mem::transmute::<_, &WithDrop<MaybeUninit<()>>>(self.data).drop_fn;
            drop_fn(self.data)
        }
    }
}

impl<'a, T, O> Func<'a, T, O> {
    /// Create a new [`Func`] encapsulating the provided function, `f`.
    ///
    /// When the [`Func`] is dropped, the encapsulated function will be too.
    #[cfg(any(not(feature = "nightly"), docsrs))]
    pub fn new<F: Fn(T) -> O + 'a>(f: F) -> Self {
        #[inline(always)]
        unsafe fn invoke<F: Fn(T) -> O, T, O>(data: *const (), args: T) -> O {
            let wd = core::mem::transmute::<_, &WithDrop<F>>(data);
            (wd.f)(args)
        }

        unsafe fn do_drop<F>(x: *mut F) {
            drop(Box::<WithDrop<F>>::from_raw(x as _))
        }

        Self {
            f: invoke::<F, T, O>,
            data: Box::into_raw(Box::new(WithDrop {
                drop_fn: unsafe { core::mem::transmute(do_drop::<F> as unsafe fn(_)) },
                f,
            })) as _,
            phantom: PhantomData,
        }
    }

    /// Call the encapsulated function with the given argument.
    pub fn call_(&self, arg: T) -> O {
        unsafe { (self.f)(self.data, arg) }
    }
}

#[cfg(feature = "nightly")]
mod nightly {
    use super::*;

    use core::marker::Tuple;

    impl<'a, T: Tuple, O> Func<'a, T, O> {
        /// Create a new [`Func`] encapsulating the provided function, `f`.
        ///
        /// When the [`Func`] is dropped, the encapsulated function will be too.
        ///
        /// Note that the given function can accept an arbitrary set of arguments.
        pub fn new<F: Fn<T, Output = O> + 'a>(f: F) -> Self {
            #[inline(always)]
            unsafe fn invoke<F: Fn<T, Output = O>, T: Tuple, O>(data: *const (), args: T) -> O {
                let wd = core::mem::transmute::<_, &WithDrop<F>>(data);
                wd.f.call(args)
            }

            unsafe fn do_drop<F>(x: *mut F) {
                drop(Box::<WithDrop<F>>::from_raw(x as _))
            }

            Self {
                f: invoke::<F, T, O>,
                data: Box::into_raw(Box::new(WithDrop {
                    drop_fn: unsafe { core::mem::transmute(do_drop::<F> as unsafe fn(_)) },
                    f,
                })) as _,
                phantom: PhantomData,
            }
        }
    }

    impl<'a, T: Tuple, O> FnOnce<T> for Func<'a, T, O> {
        type Output = O;
        #[inline(always)]
        extern "rust-call" fn call_once(self, args: T) -> O {
            unsafe { (self.f)(self.data, args) }
        }
    }
    impl<'a, T: Tuple, O> FnMut<T> for Func<'a, T, O> {
        #[inline(always)]
        extern "rust-call" fn call_mut(&mut self, args: T) -> O {
            unsafe { (self.f)(self.data, args) }
        }
    }
    impl<'a, T: Tuple, O> Fn<T> for Func<'a, T, O> {
        #[inline(always)]
        extern "rust-call" fn call(&self, args: T) -> O {
            unsafe { (self.f)(self.data, args) }
        }
    }
}
