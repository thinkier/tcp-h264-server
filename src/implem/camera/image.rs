use std::process::Stdio;

use tokio::process::Command;

use crate::{CameraArgs, VideoWrapper};
use crate::utils::Writable;

#[derive(Clone)]
pub struct ImageWrapper {
	args: CameraArgs,
	vw: VideoWrapper,
}

impl ImageWrapper {
	pub fn create(args: CameraArgs, vw: VideoWrapper) -> Self {
		ImageWrapper {
			args,
			vw,
		}
	}

	pub async fn take_snapshot(&self, addr: String) -> (Vec<u8>, &'static str) {
		if self.vw.is_active().await {
			(self.take_snapshot_from_video(addr).await, "image/jpeg")
		} else {
			(self.take_snapshot_from_cmd(addr).await, "image/bmp")
		}
	}

	pub async fn take_snapshot_from_video(&self, addr: String) -> Vec<u8> {
		let mut child = Command::new("ffmpeg")
			.args(&[
				"-i", "-",
				"-vf", "select=gte(n\\,5)",
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

	pub async fn take_snapshot_from_cmd(&self, addr: String) -> Vec<u8> {
		info!("Non-transcoding snapshot requested by {}",addr);
		let child = self.args.spawn().unwrap();

		let buf = child.wait_with_output().await.unwrap().stdout;
		info!("Snapshot completed for {}",addr);
		return buf;
	}
}