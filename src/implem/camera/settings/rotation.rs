use std::str::FromStr;
use crate::model::camera::settings::Rotation;

impl Default for Rotation {
	fn default() -> Self {
		Rotation::Normal
	}
}

impl FromStr for Rotation {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"0" => Rotation::Normal,
			"90" => Rotation::Clockwise90,
			"180" => Rotation::UpsideDown,
			"270" => Rotation::Anticlockwise90,
			_ => return Err(format!("unknown rotation {}", s))
		})
	}
}