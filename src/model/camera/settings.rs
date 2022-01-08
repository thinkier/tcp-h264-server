#[derive(Clone, Copy)]
pub enum CameraProvider {
	Legacy,
	LibCamera,
}

#[derive(Clone, Copy)]
pub struct Resolution {
	pub width: usize,
	pub height: usize,
}

#[derive(Clone, Copy)]
pub enum Rotation {
	Normal,
	Clockwise90,
	UpsideDown,
	Anticlockwise90,
}
