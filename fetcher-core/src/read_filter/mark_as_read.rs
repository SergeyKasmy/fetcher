/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use async_trait::async_trait;

use crate::error::Error;

#[async_trait]
pub trait MarkAsRead {
	async fn mark_as_read(&mut self, id: &str) -> Result<(), Error>;
}
