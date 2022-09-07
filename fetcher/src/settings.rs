/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod config;
pub mod data;
pub mod read_filter;

use fetcher_config::tasks::TaskSettings;

use color_eyre::Result;

const PREFIX: &str = "fetcher";

pub async fn get_task_settings() -> Result<TaskSettings> {
	tracing::trace!("Getting task settings");

	tracing::trace!("Getting twitter auth");
	let twitter_auth = data::twitter().await?;

	tracing::trace!("Getting google auth");
	let google_oauth2 = data::google_oauth2().await?;

	tracing::trace!("Getting email password");
	let email_password = data::email_password().await?;

	tracing::trace!("Getting telegram auth");
	let telegram = data::telegram().await?;

	tracing::trace!("Getting read filters");
	let read_filter = read_filter::get().await?;

	Ok(TaskSettings {
		twitter_auth,
		google_oauth2,
		email_password,
		telegram,
		read_filter,
	})

	// Ok(TaskSettings {
	// 	twitter_auth: data::twitter().await?,
	// 	google_oauth2: data::google_oauth2().await?,
	// 	email_password: data::email_password().await?,
	// 	telegram: data::telegram().await?,
	// 	read_filter: read_filter::get().await?,
	// })
}
