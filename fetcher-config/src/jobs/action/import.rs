/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
	error::{FetcherConfigError, Result},
	jobs::external_data::{ExternalDataResult, ProvideExternalData},
};
use fetcher_core::{action::Action as CAction, read_filter::ReadFilter as CReadFilter};

use itertools::process_results;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Import(pub String);

impl Import {
	#[allow(clippy::needless_pass_by_value)]
	pub fn parse<RF, D>(
		self,
		rf: Option<Arc<RwLock<RF>>>,
		external: &D,
	) -> Result<Option<Vec<CAction>>>
	where
		RF: CReadFilter + 'static,
		D: ProvideExternalData + ?Sized,
	{
		match external.import(&self.0) {
			ExternalDataResult::Ok(x) => {
				let v =
					process_results(x.into_iter().map(|x| x.parse(rf.clone(), external)), |i| {
						i.flatten(/* option */).flatten(/* inner vec */).collect::<Vec<_>>()
					})?;

				if v.is_empty() { Ok(None) } else { Ok(Some(v)) }
			}
			ExternalDataResult::Unavailable => Err(FetcherConfigError::ImportingUnavailable),
			ExternalDataResult::Err(e) => Err(e.into()),
		}
	}
}
