#[macro_use]
extern crate lazy_static;

use clap::Parser;
use hyper::header::{HeaderValue, CONTENT_TYPE, CONTENT_DISPOSITION};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Result, StatusCode};
use std::env;
use std::path::Path;
use uuid::Uuid;

static NOTFOUND: &[u8] = b"Not Found";

static FORM_HTML: &[u8] = br###"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body>
    <form action="" method="post" enctype="multipart/form-data">
        <input type="file" name="file" />
        <button type="submit">Upload</button>
    </form>
</body>
</html>
"###;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 5567)]
    port: u16,

    #[clap(short, long, default_value = "./")]
    dir: String,
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
    static ref UUID: Uuid = Uuid::new_v4();
}

async fn upload(req: Request<Body>) ->  Result<Response<Body>> {
    let uri_path = format!("/{}", UUID.to_string());

    if req.uri().path() == uri_path.as_str() && req.method() == &Method::GET {
        return Ok(render_form());
    } else if req.uri().path() == uri_path.as_str() && req.method() == &Method::POST {
        return Ok(not_found());
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

fn render_form() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .body(FORM_HTML.into())
        .unwrap()
}

#[tokio::main]
async fn main() {

    let dir = ARGS.dir.clone();
    let port = ARGS.port;

    let path_ = Path::new(&dir);

    if !path_.exists() {
        eprintln!("error: {} does not exist", dir);
        return;
    }

    if path_.is_file() {
        eprintln!("error: {} must be a directory", dir);
        return;
    }

    let host = match env::var("HUHD_HOST") {
        Ok(host) => host,
        Err(_) => String::from("0.0.0.0"),
    };

    let make_service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(upload)) });

    let addr = format!("{}:{}", host, port)
        .parse()
        .expect("Unable to parse socket address");
    let server = hyper::Server::bind(&addr).serve(make_service);
    println!("http://{}/{}", addr, UUID.to_string());
    if let Err(e) = server.await {
        eprintln!("error: {}", e);
    }
}
