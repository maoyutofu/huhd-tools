#[macro_use]
extern crate lazy_static;

use clap::Parser;
use hyper::header::{HeaderValue, CONTENT_TYPE, CONTENT_DISPOSITION};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Result, StatusCode};
use std::env;
use std::path::Path;
use uuid::Uuid;

static NOTFOUND: &[u8] = b"Not Found";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 5567)]
    port: u16,

    #[clap(short, long)]
    file: String,
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
    static ref UUID: Uuid = Uuid::new_v4();
}

async fn download(req: Request<Body>) ->  Result<Response<Body>> {
    let uri_path = format!("/{}", UUID.to_string());
    if req.uri().path() == uri_path.as_str() {
        let file = ARGS.file.clone();
        return simple_file_send(&file).await;
    }
    
    Ok(not_found())
}


/// HTTP status code 404
fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}

async fn simple_file_send(filename: &str) -> Result<Response<Body>> {
    // Serve a file by asynchronously reading it by chunks using tokio-util crate.
    if let Ok(file) = File::open(filename).await {
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);

        let path_ = Path::new(filename);
        let filename = path_.file_name().unwrap().to_str().unwrap();
        let attachment_name = format!("attachment;fileName={}", filename);

        let mut resp = Response::new(body);
        resp.headers_mut().append(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
        resp.headers_mut().append(CONTENT_DISPOSITION, HeaderValue::from_bytes(attachment_name.as_bytes()).unwrap());
        return Ok(resp);
    }

    Ok(not_found())
}

#[tokio::main]
async fn main() {

    let file = ARGS.file.clone();
    let port = ARGS.port;

    let path_ = Path::new(&file);

    if !path_.exists() {
        eprintln!("error: {} does not exist", file);
        return;
    }

    if path_.is_dir() {
        eprintln!("error: {} must be a file", file);
        return;
    }

    let host = match env::var("HUHD_HOST") {
        Ok(host) => host,
        Err(_) => String::from("0.0.0.0"),
    };

    let make_service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(download)) });

    let addr = format!("{}:{}", host, port)
        .parse()
        .expect("Unable to parse socket address");
    let server = hyper::Server::bind(&addr).serve(make_service);
    println!("http://{}/{}", addr, UUID.to_string());
    if let Err(e) = server.await {
        eprintln!("error: {}", e);
    }
}
