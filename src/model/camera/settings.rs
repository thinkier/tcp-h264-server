pub enum CameraProvider {
	Legacy,
	LibCamera,
}

pub struct Resolution {
	pub width: usize,
	pub height: usize,
}

pub enum Rotation {
	Normal,
	Clockwise90,
	UpsideDown,
	Anticlockwise90,
}
