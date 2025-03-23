use super::JobGroup;

pub struct DisabledJobGroup<J>(pub J);

impl<J> JobGroup for DisabledJobGroup<J> {
	async fn run_concurrently(&mut self) -> Vec<super::JobRunResult> {
		Vec::new()
	}

	async fn run_concurrently_interruptible(
		&mut self,
		_ctrl_c_signal_channel: crate::ctrl_c_signal::CtrlCSignalChannel,
	) -> Vec<super::JobRunResult> {
		Vec::new()
	}
}
