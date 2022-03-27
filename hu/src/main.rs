#[macro_use]
extern crate lazy_static;

use bytes::BufMut;
use clap::Parser;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use uuid::Uuid;
use warp::multipart::{FormData, Part};
use warp::{Rejection, Filter, Reply};

use futures::TryStreamExt;

static FORM_HTML: &str = r###"
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

    #[clap(short, long, default_value = ".")]
    dir: String,
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
    static ref UUID: Uuid = Uuid::new_v4();
}

async fn render_html_code() -> Result<Box<dyn Reply>, Rejection> {
    Ok(Box::new(warp::reply::html(FORM_HTML)))
}

async fn upload_file(form: FormData) -> Result<Box<dyn Reply>, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;

    for p in parts {
        if p.name() == "file" {
            let filename = String::from(p.filename().unwrap());

            let value = p.stream()
                .try_fold(Vec::new(), |mut vec, data| {
                    vec.put(data);
                    async move { Ok(vec) }
                })
                .await
                .map_err(|e| {
                    eprintln!("reading file error: {}", e);
                    warp::reject::reject()
                })?;

            let filename = format!("{}/{}", ARGS.dir, filename);
            tokio::fs::write(&filename, value).await.map_err(|e| {
                eprint!("error writing file: {}", e);
                warp::reject::reject()
            })?;
        }
    }
    Ok(Box::new("success"))
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

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Unable to parse socket address");

    println!("http://{}/{}", addr, UUID.to_string());

    // GET router
    let render_html = warp::path(UUID.to_string()).and_then(render_html_code);
    let get_routers = warp::get().and(render_html);

    // POST router
    let upload = warp::path(UUID.to_string())
        .and(warp::multipart::form().max_length(5_000_000_000))
        .and_then(upload_file);
    let post_routers = warp::post().and(upload);

    let rouoter = get_routers.or(post_routers);

    warp::serve(rouoter).run(addr).await;
}
