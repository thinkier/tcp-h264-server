use tokio::io::stdin;
use tokio::sync::mpsc::Sender;

use crate::h264::{H264NalUnit, H264Stream};

pub async fn read_h264_stream(consumer: Sender<H264NalUnit>) {
	let mut stream = H264Stream::new(stdin());

	// TODO Spawn a raspivid / libcamera-vid process
	info!("Capturing H.264 video from stdin");

	while let Ok(frame) = stream.next().await {
		let _ = consumer.send(frame).await;
	}
}
