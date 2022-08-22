use async_trait::async_trait;

use crate::error::Error;

#[async_trait]
pub trait MarkAsRead {
	async fn mark_as_read(&mut self, id: &str) -> Result<(), Error>;
}
