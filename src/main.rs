#![feature(exit_status_error)]
#[macro_use]
extern crate argh;
extern crate env_logger;
extern crate h264_nal_paging;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate tokio;

use log::LevelFilter;

use crate::implem::camera::{CameraArgs, Mode};
use crate::implem::camera::image::ImageWrapper;
use crate::implem::camera::video::VideoWrapper;
use crate::model::cli::CliArgs;
use crate::service::image::listen_for_new_image_requests;
use crate::service::video::listen_for_new_video_sockets;
use crate::utils::am;

mod model;
mod implem;
mod service;
mod utils;

#[tokio::main]
async fn main() {
	let args = argh::from_env::<CliArgs>();

	env_logger::builder()
		.filter_level(LevelFilter::Debug)
		.init();

	let mut vargs = CameraArgs::from((args.camera_provider, Mode::Video));
	vargs.with_resolution(args.resolution)
		.with_rotation(args.rotation)
		.with_framerate(args.framerate);

	let vw = VideoWrapper::create(vargs).await;
	let iw = ImageWrapper::create(vw.clone());

	let img = args.start_listening_for_image().await;
	tokio::spawn(listen_for_new_image_requests(img, iw));

	let vid = args.start_listening_for_video().await;
	listen_for_new_video_sockets(vid, vw).await;
}

