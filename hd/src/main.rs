#[macro_use]
extern crate lazy_static;

use clap::Parser;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use uuid::Uuid;
use warp::Filter;

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

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Unable to parse socket address");

    println!("http://{}/{}", addr, UUID.to_string());


    let path_ = Path::new(file.as_str());
    let filename = path_.file_name().unwrap().to_str().unwrap();
    let attachment_name = format!("attachment;fileName={}", filename);

    let download_router =
        warp::path(UUID.to_string())
            .and(warp::fs::file(file))
            .with(warp::reply::with::header(
                "Content-Type",
                "application/octet-stream",
            ))
            .with(warp::reply::with::header(
                "Content-Disposition",
                attachment_name,
            ));

    let routers = warp::any().and(download_router);
    warp::serve(routers).run(addr).await
}
