use std::collections::{HashMap, HashSet};
use std::time::Duration;

use h264_nal_paging::{H264NalUnit, H264Stream};
use tokio::process::Child;
use tokio::task::JoinHandle;
use tokio::time::{Instant, interval};

use crate::am;
use crate::implem::camera::CameraArgs;
use crate::utils::{Am, StreamsContainer, Writable};

struct VideoManager {
	child: Child,
	main_task_handle: JoinHandle<()>,
}

impl VideoManager {
	pub async fn destroy(mut self) {
		self.main_task_handle.abort();
		self.child.kill().await.unwrap();
	}

	pub async fn new(args: CameraArgs, streams: StreamsContainer, mon: Am<Instant>) -> Self {
		let mut child = args.spawn().unwrap();
		let stdout = child.stdout.take().unwrap();

		let mut stream = H264Stream::new(stdout);
		let mut known_addrs = HashSet::new();

		let mut seq_param: Option<H264NalUnit> = None;
		let mut pic_param: Option<H264NalUnit> = None;

		let task_handle = tokio::spawn(async move {
			while let Ok(nal) = stream.next().await {
				{
					*mon.lock().await = Instant::now();
				}

				{
					let mut streams = streams.lock().await;
					let mut write_err = Vec::with_capacity(0);

					match nal.unit_code {
						7 => {
							debug!("Writing Sequence Parameters {:?}",  nal.raw_bytes);
							seq_param = Some(nal);
							continue;
						}
						8 => {
							debug!("Writing Picture Parameters {:?}",  nal.raw_bytes);
							pic_param = Some(nal);
							continue;
						}
						// Full frame if you get my drift
						5 => {
							for (k, w) in streams.iter_mut()
								.filter(|(k, _)| !known_addrs.contains(*k))
								.collect::<Vec<_>>() {
								info!("Connected {}", k);

								known_addrs.insert(k.to_string());
								let mut buf = vec![];

								[&seq_param, &pic_param]
									.into_iter()
									.for_each(|x| x
										.iter()
										.map(|p| &p.raw_bytes)
										.for_each(|p| buf.extend(p)));
								buf.extend(&nal.raw_bytes);

								if w.write_all(&buf).await.is_err() {
									write_err.push(k.clone());
								}
							}
						}
						_ => {}
					}

					for (k, w) in streams.iter_mut().filter(|(k, _)| known_addrs.contains(*k)) {
						if w.write_all(&nal.raw_bytes).await.is_err() {
							write_err.push(k.clone());
						}
					}

					for k in write_err.into_iter().rev() {
						streams.remove(&k);
						known_addrs.remove(&k);
						info!("Disconnected {}", k);
					}
				}
			}
		});

		return VideoManager {
			child,
			main_task_handle: task_handle,
		};
	}
}

#[derive(Clone)]
pub struct VideoWrapper {
	mon: Am<Instant>,
	handle: Am<JoinHandle<()>>,
	streams: StreamsContainer,
}

impl VideoWrapper {
	pub async fn create(args: CameraArgs) -> Self {
		let video_manager: Am<Option<VideoManager>> = am(None);
		let streams = am(HashMap::new());
		let mon = am(Instant::now());

		let handle = {
			let streams = streams.clone();
			let mon = mon.clone();

			tokio::spawn(async move {
				let mut int = interval(Duration::from_millis(50));

				loop {
					int.tick().await;

					let elapsed = {
						let earlier = { Instant::clone(&*mon.lock().await) };
						Instant::now().duration_since(earlier)
					};
					if elapsed.as_secs() > 1 {
						if let Some(vm) = {
							video_manager.lock().await.take()
						} {
							info!("Destroying video session: timeout on receiving new bytes.");
							vm.destroy().await;
						}
					}

					{
						if { video_manager.lock().await.is_none() } && {
							streams.lock().await.len() > 0
						} {
							info!("Spawning new video session.");
							{
								let vm = VideoManager::new(
									args.clone(),
									streams.clone(),
									mon.clone(),
								).await;
								*video_manager.lock().await = Some(vm);
							}
							{
								*mon.lock().await = Instant::now();
							}
						}
					}
				}
			})
		};

		return VideoWrapper {
			mon,
			handle: am(handle),
			streams,
		};
	}

	pub async fn register(&self, addr: String, w: Writable) {
		self.streams.lock().await.insert(addr, w);
	}
}
