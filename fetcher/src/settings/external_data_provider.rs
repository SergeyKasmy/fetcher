/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::extentions::IntoStdErrorExt;

use super::{config, context::StaticContext, data};
use fetcher_config::jobs::{
	action::Action as ActionConfig,
	external_data::{ExternalDataError, ExternalDataResult, ProvideExternalData},
	named::{JobName, TaskName},
	read_filter::Kind as ReadFilterKind,
};
use fetcher_core::{auth, read_filter::ReadFilter, task::entry_to_msg_map::EntryToMsgMap};

pub struct ExternalDataFromDataDir {
	pub cx: StaticContext,
}

impl ProvideExternalData for ExternalDataFromDataDir {
	type ReadFilter = Box<dyn ReadFilter>;

	fn google_oauth2(&self) -> ExternalDataResult<auth::Google> {
		data::google_oauth2::get(self.cx).into()
	}

	fn email_password(&self) -> ExternalDataResult<String> {
		data::email_password::get(self.cx).into()
	}

	fn telegram_bot_token(&self) -> ExternalDataResult<String> {
		data::telegram::get(self.cx).into()
	}

	fn discord_bot_token(&self) -> ExternalDataResult<String> {
		data::discord::get(self.cx).into()
	}

	fn read_filter(
		&self,
		job: &JobName,
		task: Option<&TaskName>,
		expected_rf: ReadFilterKind,
	) -> ExternalDataResult<Self::ReadFilter> {
		data::runtime_external_save::read_filter::get(job, task, expected_rf, self.cx).into()
	}

	fn entry_to_msg_map(
		&self,
		job: &JobName,
		task: Option<&TaskName>,
	) -> ExternalDataResult<EntryToMsgMap> {
		data::runtime_external_save::entry_to_msg_map::get(job, task, self.cx).into()
	}

	fn import(&self, name: &str) -> ExternalDataResult<Vec<ActionConfig>> {
		match config::actions::find(name, self.cx) {
			Ok(Some(x)) => ExternalDataResult::Ok(x),
			Ok(None) => ExternalDataResult::Err(ExternalDataError::ActionNotFound(name.to_owned())),
			Err(e) => ExternalDataResult::Err(ExternalDataError::ActionParsingError {
				name: name.to_owned(),
				err: e.into_std_error(),
			}),
		}
	}
}
