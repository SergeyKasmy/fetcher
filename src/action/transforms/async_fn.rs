use std::{convert::Infallible, iter};

use crate::{
	action::transforms::result::{OptionUnwrapTransformResultExt, TransformedMessage},
	entry::Entry,
	maybe_send::{MaybeSend, MaybeSendSync},
};

use super::{Transform, error::TransformErrorKind, result::TransformedEntry};

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

pub trait IntoTransformedEntries {
	type Err: Into<TransformErrorKind>;

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
