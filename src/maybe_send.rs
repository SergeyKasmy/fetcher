/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`MaybeSend`], [`MaybeSync`], and [`MaybeSendSync`] traits

/// This type constrains the type to be [`Send`] only when the "send" feature is enabled.
/// Otherwise it does nothing.
#[cfg(feature = "send")]
pub trait MaybeSend: Send {}

/// This type constrains the type to be [`Send`] only when the "send" feature is enabled.
/// Otherwise it does nothing.
#[cfg(not(feature = "send"))]
pub trait MaybeSend {}

/// This type constrains the type to be [`Sync`] only when the "send" feature is enabled.
/// Otherwise it does nothing.
#[cfg(feature = "send")]
pub trait MaybeSync: Sync {}

/// This type constrains the type to be [`Sync`] only when the "send" feature is enabled.
/// Otherwise it does nothing.
#[cfg(not(feature = "send"))]
pub trait MaybeSync {}

/// This type constrains the type to be [`Send`] and [`Sync`] only when the "send" feature is enabled.
/// Otherwise it does nothing.
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
