/// This type constrains the type to be [`Send`] only when the "send" feature is enabled.
/// Otherwise it does nothing.
#[cfg(feature = "send")]
pub trait MaybeSend: Send {}

#[cfg(not(feature = "send"))]
pub trait MaybeSend {}

/// This type constrains the type to be [`Sync`] only when the "send" feature is enabled.
/// Otherwise it does nothing.
#[cfg(feature = "send")]
pub trait MaybeSync: Sync {}

#[cfg(not(feature = "send"))]
pub trait MaybeSync {}

pub trait MaybeSendSync: MaybeSend + MaybeSync {}

#[cfg(feature = "send")]
impl<T: Send + ?Sized> MaybeSend for T {}

#[cfg(not(feature = "send"))]
impl<T> MaybeSend for T {}

#[cfg(feature = "send")]
impl<T: Sync + ?Sized> MaybeSync for T {}

#[cfg(not(feature = "send"))]
impl<T> MaybeSync for T {}

impl<T: MaybeSend + MaybeSync + ?Sized> MaybeSendSync for T {}
