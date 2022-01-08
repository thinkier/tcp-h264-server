use std::str::FromStr;
use crate::model::camera::settings::Resolution;

impl Default for Resolution {
	fn default() -> Self {
		Resolution {
			width: 1280,
			height: 720,
		}
	}
}

impl FromStr for Resolution {
	type Err = <usize as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (width, height) = s.split_once("x").expect("failed to split the resolution by the delimiter 'x'");

		let (width, height) = (width.parse()?, height.parse()?);

		Ok(Resolution { width, height })
	}
}