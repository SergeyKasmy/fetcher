#[cfg(feature = "send")]
pub trait MaybeSend: Send {}

#[cfg(not(feature = "send"))]
pub trait MaybeSend {}

#[cfg(feature = "send")]
pub trait MaybeSync: Sync {}

#[cfg(not(feature = "send"))]
pub trait MaybeSync {}

pub trait MaybeSendSync: MaybeSend + MaybeSync {}

#[cfg(feature = "send")]
impl<T: Send> MaybeSend for T {}

#[cfg(not(feature = "send"))]
impl<T> MaybeSend for T {}

#[cfg(feature = "send")]
impl<T: Sync> MaybeSync for T {}

#[cfg(not(feature = "send"))]
impl<T> MaybeSync for T {}

impl<T: MaybeSend + MaybeSync> MaybeSendSync for T {}
