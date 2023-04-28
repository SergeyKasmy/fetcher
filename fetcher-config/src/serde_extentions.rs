/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod tuple {
	use serde::{de::Visitor, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
	use std::{fmt, marker::PhantomData};

	#[allow(clippy::extra_unused_type_parameters)] // they are used in the where clause and in the function body
	pub fn serialize<'a, S, V, First, Second>(
		tuple: &'a V,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
		(First, Second): From<&'a V>,
		First: Serialize + 'a,
		Second: Serialize + 'a,
	{
		let (first, second) = tuple.into();
		let mut map = serializer.serialize_map(Some(1))?;
		map.serialize_entry(&first, &second)?;
		map.end()
	}

	pub fn deserialize<'de, D, V, First, Second>(deserializer: D) -> Result<V, D::Error>
	where
		D: Deserializer<'de>,
		V: From<(First, Second)>,
		First: Deserialize<'de>,
		Second: Deserialize<'de>,
	{
		let tuple: (First, Second) = deserializer.deserialize_map(TupleVisitor {
			_v: PhantomData,
			_t: PhantomData,
			_u: PhantomData,
		})?;

		Ok(tuple.into())
	}

	struct TupleVisitor<V, First, Second> {
		_v: PhantomData<V>,
		_t: PhantomData<First>,
		_u: PhantomData<Second>,
	}

	impl<'de, V, First, Second> Visitor<'de> for TupleVisitor<V, First, Second>
	where
		V: From<(First, Second)>,
		First: Deserialize<'de>,
		Second: Deserialize<'de>,
	{
		type Value = V;

		fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
			f.write_str("a map with a one element")
		}

		fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
		where
			A: serde::de::MapAccess<'de>,
		{
			let Some(tuple) = map.next_entry()? else {
				return Err(serde::de::Error::invalid_length(0, &self));
			};

			if map.next_entry::<First, Second>()?.is_some() {
				return Err(serde::de::Error::invalid_length(2, &self));
			}

			Ok(tuple.into())
		}
	}
}
