use std::str::FromStr;
use crate::model::camera::settings::CameraProvider;

impl Default for CameraProvider {
	fn default() -> Self {
		CameraProvider::Legacy
	}
}

impl FromStr for CameraProvider {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.to_lowercase() == "libcamera" {
			return Ok(CameraProvider::LibCamera);
		} else if s.to_lowercase() == "legacy" || s.to_lowercase() == "raspi" {
			return Ok(CameraProvider::Legacy);
		}

		return Err(format!("unrecognized camera provider: {}", s));
	}
}
