
use core::fmt::Debug;
use core::ffi::c_void;

pub trait MaybeDebug {
    fn cast_debug(&self) -> Option<&'_ dyn Debug>;
    fn maybe_debug(&self) -> crate::MaybeDebug<'_>;
}

pub type DebugVtable = core::ptr::DynMetadata<dyn Debug>;

impl<T: ?Sized> MaybeDebug for T {
    #[inline]
    default fn cast_debug(&self) -> Option<&'_ dyn Debug> {
        None
    }
    #[inline]
    default fn maybe_debug(&self) -> crate::MaybeDebug<'_> {
        crate::MaybeDebug::fallback::<T>()
    }
}
impl<T: Debug> MaybeDebug for T {
    #[inline]
    fn cast_debug(&self) -> Option<&'_ dyn Debug> {
        Some(self as &dyn Debug)
    }
    #[inline]
    fn maybe_debug(&self) -> crate::MaybeDebug<'_> {
        crate::MaybeDebug::passthrough(self)
    }
}
impl<T> MaybeDebug for [T] {
    #[inline]
    fn maybe_debug(&self) -> crate::MaybeDebug<'_> {
        crate::MaybeDebug::Slice(crate::MaybeDebugSlice::from_slice(self))
    }
}
impl MaybeDebug for str {
    #[inline]
    fn maybe_debug(&self) -> crate::MaybeDebug<'_>  {
        crate::MaybeDebug::passthrough_str(self)
    }
}

#[inline]
pub fn retrieve_vtable(d: &dyn Debug) -> DebugVtable {
    // NOTE: Transmute is necessary because of the lifetime mismatch..
    unsafe { core::mem::transmute::<
        core::ptr::DynMetadata<_>,
        core::ptr::DynMetadata<_>,
    >(core::ptr::metadata(d)) }
}

#[inline]
pub unsafe fn from_vtable<'a>(val: *const c_void, metadata: DebugVtable) -> &'a dyn Debug {
    &*core::ptr::from_raw_parts(val as *const (), metadata)
}
