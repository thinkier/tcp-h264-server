use std::io::Result as IoResult;
use std::process::Stdio;
use std::time::Duration;

use tokio::process::{Child, Command};

use crate::model::camera::settings::{CameraProvider, Resolution, Rotation};

pub mod image;
pub mod settings;
pub mod video;

#[derive(PartialEq)]
pub enum Mode {
	Image,
	Video,
}

#[derive(Clone)]
pub struct CameraArgs {
	cmd: &'static str,
	args: Vec<String>,
}

impl From<(CameraProvider, Mode)> for CameraArgs {
	fn from((p, m): (CameraProvider, Mode)) -> Self {
		let mut args = ["-o", "-"].into_iter()
			.map(str::to_string)
			.collect::<Vec<_>>();

		if m == Mode::Image {
			args.extend(["--encoding", "bmp"].into_iter().map(str::to_string))
		}

		CameraArgs {
			cmd: match (p, m) {
				(CameraProvider::Legacy, Mode::Image) => "raspistill",
				(CameraProvider::LibCamera, Mode::Image) => "raspivid",
				(CameraProvider::Legacy, Mode::Video) => "libcamera-vid",
				(CameraProvider::LibCamera, Mode::Video) => "libcamera-still"
			},
			args,
		}
	}
}

impl CameraArgs {
	pub fn with_resolution(&mut self, res: &Resolution) -> &mut Self {
		self.args.push("--width".to_string());
		self.args.push(res.width.to_string());
		self.args.push("--height".to_string());
		self.args.push(res.height.to_string());

		return self;
	}

	pub fn with_rotation(&mut self, rot: &Rotation) -> &mut Self {
		let args = match rot {
			Rotation::Clockwise90 => ["--rotation", "90"],
			Rotation::UpsideDown => ["--hflip", "--vflip"],
			Rotation::Anticlockwise90 => ["--rotation", "270"],
			_ => return self
		};

		self.args.extend(args.into_iter().map(str::to_string));

		return self;
	}

	pub fn with_shutter_speed(&mut self, duration: &Duration) -> &mut Self {
		self.args.push("--shutter".to_string());
		self.args.push(duration.as_micros().to_string());

		return self;
	}

	pub fn spawn(&self) -> IoResult<Child> {
		Command::new(self.cmd)
			.args(&self.args)
			.stdin(Stdio::null())
			.stdout(Stdio::piped())
			.stderr(Stdio::inherit())
			.spawn()
	}
}
