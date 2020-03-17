#![deny(warnings)]

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::convert::Infallible;
use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use clap::{App, Arg};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
static NOTFOUND: &[u8] = b"Not Found";
static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";

struct Config {
    docroot: PathBuf,
}

async fn handle_request(config: &Config, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let filename = config.docroot.join(&req.uri().path()[1..]);
    match filename.exists() {
        true => match filename.extension() != Some(OsStr::new("php")) {
            true => Ok(serve_static_file(&filename).await),
            _ => Ok(fastcgi_proxy().await),
        },
        _ => Ok(fastcgi_proxy().await),
    }
}

async fn fastcgi_proxy() -> Response<Body> {
    // TODO: use hyper client here to proxy to php-fpm spawned process
    return not_found();
}

async fn serve_static_file(filename: &Path) -> Response<Body> {
    //   TODO:
    //   - mime types when serving static files
    if let Ok(mut file) = File::open(filename).await {
        let mut buf = Vec::new();
        if let Ok(_) = file.read_to_end(&mut buf).await {
            return Response::new(buf.into());
        }

        let index_html = filename.join("index.html");
        let mut resolved_filename = match index_html.is_file() {
            true => index_html,
            _ => filename.to_path_buf(),
        };

        if resolved_filename == filename {
            let index_htm = filename.join("index.htm");
            resolved_filename = match index_htm.is_file() {
                true => index_htm,
                _ => filename.to_path_buf(),
            };
        }

        if resolved_filename == filename {
            return internal_server_error();
        }

        debug!(
            "Resolved {} into {}",
            filename.display(),
            resolved_filename.display()
        );

        if let Ok(mut file) = File::open(resolved_filename).await {
            if let Ok(_) = file.read_to_end(&mut buf).await {
                return Response::new(buf.into());
            }

            return internal_server_error();
        }

        return internal_server_error();
    }
    return not_found();
}

/// HTTP status code 404
fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}

/// HTTP status code 500
fn internal_server_error() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(INTERNAL_SERVER_ERROR.into())
        .unwrap()
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
pub async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    let app = App::new(APP_NAME)
        .version(VERSION)
        .about("Lightweight 🍃 fast ⚡ fastcgi php 🐘 proxy and static http server.")
        .arg(
            Arg::with_name("serve")
                .short("s")
                .long("serve")
                .takes_value(true)
                .default_value("127.0.0.1:3000")
                .value_name("ADDR")
                .help("Serve on address and port"),
        )
        .arg(
            Arg::with_name("docroot")
                .short("d")
                .long("document-root")
                .takes_value(true)
                .value_name("DIR")
                .default_value("./")
                .help("Document root of both static files and\nphp-fpm"),
        );

    let matches = app.get_matches();

    let addr = match matches.value_of("serve") {
        Some(v) => v.to_string().parse().unwrap(),
        None => "127.0.0.1:3000".to_string().parse().unwrap(),
    };

    let docroot = match matches.value_of("docroot") {
        Some(v) => v,
        None => "./",
    };
    let d = Path::new(docroot).to_owned();
    if !d.is_dir() {
        panic!("docroot {} is a non existing directory.", docroot);
    }
    let config = Arc::new(Config { docroot: d });
    let make_svc = make_service_fn(|_| {
        let onion1 = Arc::clone(&config);
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let onion2 = Arc::clone(&onion1);
                info!("{:?}", req);
                async move { handle_request(&*onion2, req).await }
            }))
        }
    });

    // TODO: spawn php-fpm master process
    let server = Server::bind(&addr).tcp_nodelay(true).serve(make_svc);

    info!("Listening on http://{}", addr);
    info!("Serving docroot {}", docroot);
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}
