//! Reference Capabilities (Pony-inspired)
//!
//! Type-level enforcement of data race freedom through compile-time
//! reference capability checking.
//!
//! # Capability Types
//! - `iso`: Isolated - unique read/write access, sendable
//! - `val`: Value - immutable, freely shareable and sendable
//! - `ref`: Reference - local read/write, NOT sendable
//! - `tag`: Tag - identity only, for actor references

use std::marker::PhantomData;

/// Reference capability marker trait
pub trait RefCap: Send + Sync + 'static {
    /// Whether this capability allows sending across threads
    const SENDABLE: bool;

    /// Whether this capability allows mutation
    const MUTABLE: bool;

    /// Whether this capability is unique (exclusive)
    const UNIQUE: bool;
}

/// Isolated capability - unique read/write, sendable
#[derive(Debug, Clone, Copy)]
pub struct Iso;

impl RefCap for Iso {
    const SENDABLE: bool = true;
    const MUTABLE: bool = true;
    const UNIQUE: bool = true;
}

/// Value capability - immutable, freely shareable
#[derive(Debug, Clone, Copy)]
pub struct Val;

impl RefCap for Val {
    const SENDABLE: bool = true;
    const MUTABLE: bool = false;
    const UNIQUE: bool = false;
}

/// Reference capability - local read/write, not sendable
#[derive(Debug, Clone, Copy)]
pub struct Ref;

impl RefCap for Ref {
    const SENDABLE: bool = false;
    const MUTABLE: bool = true;
    const UNIQUE: bool = false;
}

/// Tag capability - identity only, for actor references
#[derive(Debug, Clone, Copy)]
pub struct Tag;

impl RefCap for Tag {
    const SENDABLE: bool = true;
    const MUTABLE: bool = false;
    const UNIQUE: bool = false;
}

/// Marker trait for sendable capabilities (iso, val, tag)
pub trait SendCap: RefCap {}

impl SendCap for Iso {}
impl SendCap for Val {}
impl SendCap for Tag {}

/// Capability-tracked data
///
/// The type system enforces that only `Iso` and `Val` capabilities
/// can be sent across threads.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tracked<T, Cap: RefCap> {
    data: T,
    _cap: PhantomData<Cap>,
}

impl<T, Cap: RefCap> Tracked<T, Cap> {
    /// Create new capability-tracked data
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            data,
            _cap: PhantomData,
        }
    }

    /// Get immutable reference to data
    #[inline]
    pub fn get(&self) -> &T {
        &self.data
    }

    /// Get mutable reference (only if capability allows)
    #[inline]
    pub fn get_mut(&mut self) -> &mut T
    where
        Cap: RefCap,
    {
        debug_assert!(Cap::MUTABLE, "Capability does not allow mutation");
        &mut self.data
    }

    /// Unwrap the tracked data
    #[inline]
    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T: Clone, Cap: RefCap> Tracked<T, Cap> {
    /// Clone if not unique (Val capability)
    pub fn share(&self) -> Option<Self>
    where
        Cap: RefCap,
    {
        if !Cap::UNIQUE {
            Some(Self::new(self.data.clone()))
        } else {
            None
        }
    }
}

// Only sendable capabilities can be sent across threads
unsafe impl<T: Send, Cap: SendCap> Send for Tracked<T, Cap> {}
unsafe impl<T: Sync, Cap: SendCap> Sync for Tracked<T, Cap> {}

/// Convert from `Iso` to `Val` (consume unique, create immutable)
impl<T> From<Tracked<T, Iso>> for Tracked<T, Val> {
    fn from(iso: Tracked<T, Iso>) -> Self {
        Self::new(iso.into_inner())
    }
}

/// Capability assertion at compile time
#[macro_export]
macro_rules! assert_sendable {
    ($cap:ty) => {
        const _: () = {
            fn _assert_sendable<C: $crate::capabilities::SendCap>() {}
            _assert_sendable::<$cap>();
        };
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso_is_sendable() {
        assert!(Iso::SENDABLE);
        assert!(Iso::MUTABLE);
        assert!(Iso::UNIQUE);
    }

    #[test]
    fn test_val_is_sendable() {
        assert!(Val::SENDABLE);
        assert!(!Val::MUTABLE);
        assert!(!Val::UNIQUE);
    }

    #[test]
    fn test_ref_not_sendable() {
        assert!(!Ref::SENDABLE);
        assert!(Ref::MUTABLE);
        assert!(!Ref::UNIQUE);
    }

    #[test]
    fn test_tracked_iso() {
        let mut tracked = Tracked::<i32, Iso>::new(42);
        assert_eq!(*tracked.get(), 42);
        *tracked.get_mut() = 100;
        assert_eq!(*tracked.get(), 100);
    }

    #[test]
    fn test_iso_to_val_conversion() {
        let iso = Tracked::<String, Iso>::new("hello".to_string());
        let val = Tracked::<String, Val>::from(iso);
        assert_eq!(val.get(), "hello");
    }

    #[test]
    fn test_val_share() {
        let val = Tracked::<i32, Val>::new(42);
        let shared = val.share().unwrap();
        assert_eq!(shared.get(), val.get());
    }
}
