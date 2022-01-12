use std::process::Stdio;

use tokio::process::Command;

use crate::VideoWrapper;
use crate::utils::Writable;

#[derive(Clone)]
pub struct ImageWrapper {
	vw: VideoWrapper,
}

impl ImageWrapper {
	pub fn create(vw: VideoWrapper) -> Self {
		ImageWrapper {
			vw,
		}
	}

	pub async fn take_snapshot_from_video(&self, addr: String) -> Vec<u8> {
		let mut child = Command::new("ffmpeg")
			.args(&[
				"-i", "-",
				"-vframes", "1",
				"-f", "image2",
				"pipe:"
			])
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.stderr(Stdio::null())
			.spawn()
			.unwrap();

		if let Some(stdin) = child.stdin.take() {
			self.vw.register(addr, Writable::ChildStdin(stdin)).await;
		}

		return child.wait_with_output().await.unwrap().stdout;
	}
}