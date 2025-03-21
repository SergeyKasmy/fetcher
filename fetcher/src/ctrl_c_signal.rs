use tokio::sync::watch;

#[derive(Clone, Debug)]
pub struct CtrlCSignalChannel(pub watch::Receiver<()>);

impl CtrlCSignalChannel {
	pub async fn signaled(&mut self) {
		self.0
			.changed()
			.await
			.expect("Sender should be running in a detached tokio task and never dropped");
	}
}
