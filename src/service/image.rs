use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::header::HeaderName;
use hyper::http::HeaderValue;
use hyper::server::Builder;
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::service::{make_service_fn, service_fn};

use crate::ImageWrapper;

#[derive(Clone)]
struct HyperCtx {
	iw: ImageWrapper,
}

pub async fn listen_for_new_image_requests(server: Builder<AddrIncoming>, iw: ImageWrapper) {
	let ctx = HyperCtx { iw };

	let make_service = make_service_fn(move |conn: &AddrStream| {
		let ctx = ctx.clone();
		let addr = conn.remote_addr();

		async move {
			Ok::<_, Infallible>(service_fn(move |req| {
				handle(ctx.clone(), addr, req)
			}))
		}
	});

	server
		.serve(make_service)
		.await
		.unwrap();
}

async fn handle(ctx: HyperCtx, addr: SocketAddr, req: Request<Body>) -> Result<Response<Body>, Infallible> {
	if req.method() != Method::GET || req.uri() != "/" {
		return bad_request();
	}

	let bytes= ctx.iw.take_snapshot_from_video(format!("[snapshot {}]", addr)).await;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(HeaderName::from_static("content-type"), HeaderValue::from_static("image/jpeg"))
		.body(Body::from(bytes))
		.unwrap())
}

fn bad_request() -> Result<Response<Body>, Infallible> {
	Ok(Response::builder()
		.status(StatusCode::BAD_REQUEST)
		.body(Body::empty())
		.unwrap())
}
