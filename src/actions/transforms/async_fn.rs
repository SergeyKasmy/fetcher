/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains an implementation of [`Transform`] for async closures
//! returning [`Entry`], [`TransformedEntry`], [`Result`] of Entries, and [`Vec`] of Entries

use std::{convert::Infallible, iter};

use crate::{
	actions::transforms::result::{OptionUnwrapTransformResultExt, TransformedMessage},
	entry::Entry,
	maybe_send::{MaybeSend, MaybeSendSync},
};

use super::{Transform, error::TransformErrorKind, result::TransformedEntry};

///  Use [`transform_fn`](`crate::actions::transform_fn`) to improve type inference
impl<F, T, Fut> Transform for F
where
	F: Fn(Entry) -> Fut + MaybeSendSync,
	Fut: Future<Output = T> + MaybeSend,
	T: IntoTransformedEntries,
{
	type Err = T::Err;

	async fn transform_entry(&self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		let entries = (self)(entry).await;

		Ok(entries.into_transformed_entries()?.into_iter().collect())
	}
}

/// Conversion into transformed entries
pub trait IntoTransformedEntries {
	/// Error that may be returned. Return [`Infallible`](`std::convert::Infallible`) if it never errors
	type Err: Into<TransformErrorKind>;

	/// Converts self into an iterator of [`TransformedEntries`](`TransformedEntry`)
	fn into_transformed_entries(
		self,
	) -> Result<impl IntoIterator<Item = TransformedEntry>, Self::Err>;
}

impl IntoTransformedEntries for Entry {
	type Err = Infallible;

	#[expect(refining_impl_trait)]
	fn into_transformed_entries(self) -> Result<iter::Once<TransformedEntry>, Self::Err> {
		Ok(iter::once(TransformedEntry {
			id: self.id.unwrap_or_empty(),
			reply_to: self.reply_to.unwrap_or_empty(),
			raw_contents: self.raw_contents.unwrap_or_empty(),
			msg: TransformedMessage {
				title: self.msg.title.unwrap_or_empty(),
				body: self.msg.body.unwrap_or_empty(),
				link: self.msg.link.unwrap_or_empty(),
				media: self.msg.media.unwrap_or_empty(),
			},
		}))
	}
}

impl IntoTransformedEntries for TransformedEntry {
	type Err = Infallible;

	#[expect(refining_impl_trait)]
	fn into_transformed_entries(self) -> Result<iter::Once<TransformedEntry>, Self::Err> {
		Ok(iter::once(self))
	}
}

impl<T> IntoTransformedEntries for Vec<T>
where
	T: IntoTransformedEntries<Err = Infallible>,
{
	type Err = Infallible;

	fn into_transformed_entries(
		self,
	) -> Result<impl IntoIterator<Item = TransformedEntry>, Self::Err> {
		Ok(self.into_iter().flat_map(|entries| {
			let Ok(entries) = entries.into_transformed_entries();
			entries
		}))
	}
}

impl<E> IntoTransformedEntries for Result<Entry, E>
where
	E: Into<TransformErrorKind>,
{
	type Err = E;

	#[expect(refining_impl_trait)]
	fn into_transformed_entries(
		self,
	) -> Result<iter::Once<TransformedEntry>, <Self as IntoTransformedEntries>::Err> {
		self.map(|e| {
			let Ok(entries) = e.into_transformed_entries();
			entries
		})
	}
}

impl<E> IntoTransformedEntries for Result<TransformedEntry, E>
where
	E: Into<TransformErrorKind>,
{
	type Err = E;

	#[expect(refining_impl_trait)]
	fn into_transformed_entries(
		self,
	) -> Result<iter::Once<TransformedEntry>, <Self as IntoTransformedEntries>::Err> {
		self.map(|e| {
			let Ok(entries) = e.into_transformed_entries();
			entries
		})
	}
}

impl<T, E> IntoTransformedEntries for Result<Vec<T>, E>
where
	T: IntoTransformedEntries<Err = Infallible>,
	E: Into<TransformErrorKind>,
{
	type Err = E;

	fn into_transformed_entries(
		self,
	) -> Result<impl IntoIterator<Item = TransformedEntry>, <Self as IntoTransformedEntries>::Err>
	{
		self.map(|v| {
			v.into_iter().flat_map(|entries| {
				let Ok(entries) = entries.into_transformed_entries();
				entries
			})
		})
	}
}

/*
impl<T> IntoTransformedEntries for T
where
	T: IntoIterator<Item = TransformedEntry>,
{
	#[expect(refining_impl_trait)]
	fn into_transformed_entries(self) -> T {
		self
	}
}
*/
