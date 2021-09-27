use core::ffi::c_void;
use core::fmt::Debug;

pub type DebugVtable = core::convert::Infallible;

pub trait MaybeDebug {
    fn cast_debug(&self) -> Option<&'_ dyn Debug>;
    fn maybe_debug(&self) -> crate::MaybeDebug<'_>;
}
impl<T: ?Sized> MaybeDebug for T {
    #[inline]
    fn cast_debug(&self) -> Option<&'_ dyn Debug> {
        None
    }
    #[inline]
    fn maybe_debug(&self) -> crate::MaybeDebug<'_> {
        crate::MaybeDebug::fallback::<T>()
    }
}

pub fn retrieve_vtable(_d: &dyn Debug) -> DebugVtable {
    unreachable!()
}
pub unsafe fn from_vtable<'a>(_val: *const c_void, vtable: DebugVtable) -> &'a dyn Debug {
    match vtable {}
}
