#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(maybe_debug_nightly, feature(specialization, ptr_metadata))]
#![allow(
    incomplete_features, // specialization is incomplete...
)]

use core::marker::PhantomData;
use core::fmt::{self, Debug, Formatter};
use core::ffi::c_void;

#[rustversion::nightly]
const NIGHTLY: bool = true;

#[rustversion::not(nightly)]
const NIGHTLY: bool = false;

#[rustversion::nightly]
mod backend {
    include!("nightly.rs");
}

#[rustversion::not(nightly)]
mod backend {
    include!("stable.rs");
}

/// A version of [`std::dbg!`](https://doc.rust-lang.org/std/macro.dbg.html) that works regardless
/// of whether or not `T` implements `Debug`
///
/// This requires the standard library to be present.
///
/// This macro is also aliased as `dbg!`,
/// so you may use a fully qualified `maybe_debug::dbg!()` if you so chose.
///
/// See also [maybe_debug] function.
///
/// ## Example
/// ```
/// use maybe_debug::maybe_dbg;
/// let a = vec![5, 4, 8];
/// assert_eq!(maybe_dbg!(a), vec![5, 4, 8]);
/// let a = vec![2, 4, 7];
/// // NOTE: Absolute path is useful for the 'dbg!' variant (and often clearer)
/// let (a, b, c) = maybe_debug::dbg!(a, format!("foooz"), 5u32);
/// 
/// let (a, b, _c): (Vec<i32>, String, u32) = maybe_debug::dbg!(a, b, c);
/// drop(a);
/// drop(b);
/// ```
#[macro_export]
macro_rules! maybe_dbg {
    () => (std::dbg!());
    ($val:expr $(,)?) => {
        {
            let val = $val;
            std::dbg!($crate::maybe_debug(&val));
            val
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::maybe_dbg!($val)),+ ,)
    };
}

pub use self::maybe_dbg as dbg;

/// Attempt to cast the specified value into `&dyn Debug`,
/// returning `None` if this fails.
///
/// This always returns `None` on the stable compiler.
///
/// Currently it is not possible to support casting unsized types.
#[inline]
pub fn cast_debug<'a, T: 'a>(val: &'a T) -> Option<&'a (dyn Debug + 'a)> {
    <T as backend::MaybeDebug>::cast_debug(val)
}

/// Attempt to cast the specified value into `&dyn Debug`,
/// falling back to a reasonable default on failure.
/// 
/// This unconditionally delegates to [MaybeDebug::fallback] on the stable compiler.
#[inline]
pub fn maybe_debug<T: ?Sized>(val: &T) -> MaybeDebug<'_> {
    <T as backend::MaybeDebug>::maybe_debug(val)
}

/// Optional presense of [Debug] information (equivalent to `Option<&dyn Debug>`)
///
/// The main difference from the equivalent `Option`
/// is that it prints a reasonable fallback (the type's name).
/// 
/// In other words `maybe_debug(NotDebug)` gives `"NotDebug { ... }`
/// instead of just printing `None`.
///
/// You can *always* retrieve the original type name
/// of the value, regardless of whether the `Debug` implementation
/// is `Some` or `None`.
///
/// The type name is the same one given by [core::any::type_name].
///
/// The specific variants of this struct are considered an implementation detail.
#[non_exhaustive]
#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub enum MaybeDebug<'a> {
    #[doc(hidden)]
    DynTrait {
        type_name: &'static str,
        val: &'a (dyn Debug + 'a)
    },
    #[doc(hidden)]
    TypeName {
        type_name: &'static str
    },
    #[doc(hidden)]
    Slice(MaybeDebugSlice<'a>),
    #[doc(hidden)]
    Str(&'a str),
}
impl<'a> MaybeDebug<'a> {
    /// Check if this type contians an underlying `Debug` implementation.
    ///
    /// Using the `Option<&dyn Debug>` analogy, this would be equivalent to `is_some`
    #[inline]
    pub fn has_debug_info(&self) -> bool {
        match *self {
            MaybeDebug::Str(_) | MaybeDebug::DynTrait { .. } => true,
            MaybeDebug::TypeName { .. } => false,
            MaybeDebug::Slice(ref s) => s.has_debug_info(),
        }
    }
    /// Check if this type is using a fallback implementation
    /// (based on the type name) (ie `!self.has_debug_info()`)
    /// 
    /// Note this returns `true` even for `MaybeDebug::fallback_slice`,
    /// despite the fact that does include length information.
    /// 
    /// Using the `Option<&dyn Debug>` analogy, this would be equivalent
    /// to `is_none`.
    #[inline]
    pub fn is_fallback(&self) -> bool {
        !self.has_debug_info()
    }
    /// Check if this type is known to be a slice.
    ///
    /// This function has false negatives, but no false positives.
    ///
    /// For example, if `MaybeDebug::fallback` is used, then this function
    /// will return `false` even if `fallback` is given a slice.
    #[inline]
    pub fn is_known_slice(&self) -> bool {
        matches!(*self, MaybeDebug::Slice { .. })
    }
    /// If the original value was a slice,
    /// return its length.
    /// 
    /// Returns `None` if the original value was not known to be a slice.
    ///
    /// Just like [MaybeDebug::is_known_slice],
    /// this may have false negatives on whether or not something is a slice.
    /// However, if it returns `Some`, then the length is guarenteed to be correct.
    #[inline]
    pub fn original_slice_len(&self) -> Option<usize> {
        match *self {
            MaybeDebug::Slice(ref s) => Some(s.original_len()),
            _ => None
        }
    }
    /// Return the underling type name,
    /// as given by `core::any::type_name`
    ///
    /// This function always succeeds.
    #[inline]
    pub fn type_name(&self) -> &'static str {
        match *self {
            MaybeDebug::DynTrait { type_name, .. } |
            MaybeDebug::TypeName { type_name } => type_name,
            MaybeDebug::Slice(ref s) => s.type_name,
            MaybeDebug::Str(_) => core::any::type_name::<str>(),
        }
    }
    /// Construct a "passthrough" 'MaybeDebug',
    /// that will simply delegate to the underlying implementation.
    ///
    /// On nightly, this is equivalent to [maybe_debug].
    /// However, on stable, `maybe_debug` cannot specialize
    /// and will unconditionally call [MaybeDebug::fallback].
    ///
    /// Therefore, this may be useful on stable rust
    /// if you need to create a 'MaybeDebug' value from
    /// a type that is unconditionally known to implement `Debug`.
    #[inline]
    pub fn passthrough<T: Debug + 'a>(val: &'a T) -> Self {
        MaybeDebug::DynTrait {
            val: val as &'a (dyn Debug + 'a),
            type_name: core::any::type_name::<T>()
        }
    }
    /// Construct a "passthrough" `MaybeDebug`,
    /// that will simply delegate to `str`'s `Debug` implementation.
    ///
    /// Note that a regular `passthrough` cannot work because `str: !Sized`
    #[inline]
    pub fn passthrough_str(val: &'a str) -> Self {
        MaybeDebug::Str(val)
    }
    /// Construct a "passthrough" `MaybeDebug`
    /// that will simply Debug each individual value in the slice.
    ///
    /// On nightly, this is equivalent to [maybe_debug].
    /// However, on stable, `maybe_debug` cannot specialize
    /// and will unconditionally call [MaybeDebug::fallback].
    ///
    /// Even worse, it cannot even detect the fact the type is a slice.
    ///
    /// NOTE (for stable only): Until [RFC #2580](https://github.com/rust-lang/rfcs/blob/master/text/2580-ptr-meta.md)
    /// is stabilized, this function will unconditionally
    /// delegate to [MaybeDebug::fallback_slice] on the *stable compiler*.
    /// This is still better than `maybe_debug` (or a plain `fallback`),
    /// since it also prints the length. Users using nightly can ignore this warning.
    ///
    /// See also [MaybeDebug::passthrough] and [MaybeDebug::fallback_slice].
    #[inline]
    pub fn passthrough_slice<T: Debug + 'a>(val: &'a [T]) -> Self {
        if NIGHTLY {
            MaybeDebug::Slice(MaybeDebugSlice::from_slice(val))
        } else {
            /*
             * Currently no way to construct a vtable on stable
             * TODO: Fix once RFC #2580 is stabilized
             */
            MaybeDebug::fallback_slice(val)
        }
    }
    /// Unconditionally use the "fallback" implementation for the specified slice.
    ///
    /// The current implementation simply prints the name
    /// and the slice length.
    ///
    /// On stable this actually gives *more* information than [maybe_debug].
    /// This is because `maybe_debug` cannot specialize to slices,
    /// so it simply prints the type name without any length information.
    ///
    /// On nightly, this is strictly worse than [maybe_debug],
    /// because it cannot specialize to the case that `[T]: Debug`.
    #[inline]
    pub fn fallback_slice<T>(val: &'a [T]) -> Self {
        MaybeDebug::Slice(unsafe { MaybeDebugSlice::with_vtable(val, None) })
    }
    /// A fallback implementation that unconditionally prints the type name.
    ///
    /// For example `fallback<Option<usize>>()`
    /// would print `"std::option::Option<usize> { ... }"`
    ///
    /// On stable, [maybe_debug] unconditionally delegates to this function.
    /// On nightly, this is used if `T` is `!Debug` (and isn't a slice).
    ///
    /// However on nightly, if `T` is `Debug` it is strictly worse than [maybe_debug]
    /// because it cannot take advantage of any `Debug` implementation for `T`.
    ///
    /// There is really almost no scenario in which you want to use this function.
    /// [core::any::type_name] expresses the intent of printing the type name more clearly
    /// and `maybe_debug` is able to take advantage of specialization  on the nightly compiler.
    ///
    /// It is included only for completeness.
    /// NOTE: It doesn't actually require a value for `T`,
    /// since it only uses its type name.
    ///
    /// See also [Self::fallback_slice] which also prints length information (which may actually be useful)
    #[inline]
    pub fn fallback<T: ?Sized>() -> Self {
        MaybeDebug::TypeName {
            type_name: core::any::type_name::<T>()
        }
    }
}
impl<'a> Debug for MaybeDebug<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            MaybeDebug::DynTrait { val, .. } => val.fmt(f),
            MaybeDebug::TypeName { type_name } => {
                f.debug_struct(type_name)
                    .finish_non_exhaustive()
            },
            MaybeDebug::Slice(ref s) => Debug::fmt(s, f),
            MaybeDebug::Str(s) => Debug::fmt(s, f),
        }
    }
}

/// A slice of elements which may or may not implement `Debug`
///
/// `MaybeDebugSlice` is to `&[T]` as `MaybeDebug` is to `T`
/// 
/// This is considered an implementation detail.
#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct MaybeDebugSlice<'a> {
    elements: *const c_void,
    len: usize,
    type_size: usize,
    type_name: &'static str,
    debug_vtable: Option<backend::DebugVtable>,
    marker: PhantomData<&'a ()>
}
impl<'a> MaybeDebugSlice<'a> {
    /// Check if this type actually has any debug information,
    /// returning `false` if this will just print the length.
    #[inline]
    fn has_debug_info(&self) -> bool {
        self.debug_vtable.is_some()
    }
    /// Wrap the specified slice to implement `Debug`
    #[inline]
    pub(crate) fn from_slice<T: 'a>(elements: &'a [T]) -> Self {
        let debug_vtable = if elements.is_empty() {
            None
        } else {
            crate::cast_debug::<T>(&elements[0]).map(backend::retrieve_vtable)
        };
        unsafe { Self::with_vtable(elements, debug_vtable) }
    }
    #[inline]
    unsafe fn with_vtable<T: 'a>(elements: &'a [T], debug_vtable: Option<backend::DebugVtable>) -> Self {
        MaybeDebugSlice {
            elements: elements.as_ptr() as *const _,
            type_size: core::mem::size_of::<T>(),
            len: elements.len(),
            type_name: core::any::type_name::<T>(),
            debug_vtable, marker: PhantomData
        }
    }
    /// Get the length of the original slice
    ///
    /// This is available regardless of whether
    /// or not the type actually implements debug.
    #[inline]
    fn original_len(&self) -> usize {
        self.len
    }
}
impl Debug for MaybeDebugSlice<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.len == 0 {
            f.debug_list().finish()
        } else if let Some(vtable) = self.debug_vtable {
            let mut elements = self.elements;
            let mut debugger = f.debug_list();
            unsafe {
                for _ in 0..self.len {
                    debugger.entry(backend::from_vtable(
                        elements, vtable
                    ));
                    elements = (elements as *const u8).add(self.type_size).cast();
                }
            }
            debugger.finish()
        } else {
            write!(f, "[{} of {}]", self.len, self.type_name)
        }
    }
}
