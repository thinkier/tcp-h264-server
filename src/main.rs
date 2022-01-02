#![feature(exit_status_error)]
#[macro_use]
extern crate argh;
extern crate env_logger;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate tokio;

use std::collections::HashMap;

use log::LevelFilter;
use tokio::sync::mpsc;

use crate::cli::CliArgs;
use crate::service::image::listen_for_new_image_requests;
use crate::service::read_h264::read_h264_stream;
use crate::service::video::listen_for_new_video_sockets;
use crate::service::write_h264::write_h264_stream;
use crate::utils::am;

mod cli;
mod h264;
mod service;
mod utils;

#[tokio::main]
async fn main() {
	let args = argh::from_env::<CliArgs>();

	env_logger::builder()
		.filter_level(LevelFilter::Info)
		.init();

	let (tx, rx) = mpsc::channel(60);
	let socks = am(HashMap::new());
	tokio::spawn(read_h264_stream(tx));
	tokio::spawn(write_h264_stream(rx, socks.clone()));

	let img = args.start_listening_for_image().await;
	tokio::spawn(listen_for_new_image_requests(img, socks.clone()));

	let vid = args.start_listening_for_video().await;
	listen_for_new_video_sockets(vid, socks).await;
}

