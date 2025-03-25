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

/// Returns when the CtrlC channel signals that Ctrl-C has been pressed.
/// If ctrlc_chan is None, then it never returns
pub(crate) async fn ctrlc_signaled(ctrlc_chan: Option<&mut CtrlCSignalChannel>) {
	match ctrlc_chan {
		Some(ctrlc_chan) => ctrlc_chan.signaled().await,
		None => std::future::pending().await,
	}
}
