#![deny(warnings, rust_2018_idioms)]

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about, version)]
struct Args {
    /// One or more addresses to listen on.
    #[clap(default_value = "0.0.0.0:8080")]
    addrs: Vec<std::net::SocketAddr>,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runs a basic HTTP server that always returns 204 No Content.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args { addrs } = Args::parse();

    let (tx, rx) = tokio::sync::watch::channel(());
    let tasks = addrs
        .into_iter()
        .map(|addr| {
            let lis = std::net::TcpListener::bind(addr)?;
            lis.set_nonblocking(true)?;
            Ok::<_, std::io::Error>(tokio::spawn(serve_addr(lis, rx.clone())))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut servers = futures_util::future::try_join_all(tasks);

    // When SIGINT or SIGTERM is received, notify servers to
    let mut interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {
        res = (&mut servers) => match res {
            Ok(ress) => match ress.into_iter().collect::<Result<Vec<_>, _>>() {
                Ok(_) => return Err("server exited unexpectedly".into()),
                Err(error) => return Err(format!("server exited unexpectedly: {error}").into()),
            },
            Err(error) => return Err(format!("server exited unexpectedly: {error}").into()),
        },
        _ = interrupt.recv() => {
            let _ = tx.send(());
            if let Err(error) = servers.await {
                return Err(format!("server exited unexpectedly: {error}").into());
            }
        },
        _ = terminate.recv() => {
            let _ = tx.send(());
            if let Err(error) = servers.await {
                return Err(format!("server exited unexpectedly: {error}").into());
            }
        },
    }

    Ok(())
}

async fn serve_addr(
    lis: std::net::TcpListener,
    mut rx: tokio::sync::watch::Receiver<()>,
) -> std::io::Result<()> {
    let lis = tokio::net::TcpListener::from_std(lis)?;
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();

    loop {
        tokio::select! {
            res = lis.accept() => {
                let (io, _) = res?;
                io.set_nodelay(true)?;
                tokio::spawn(graceful.watch(serve_conn(io)));
            }
            _ = rx.changed() => {
                eprintln!("Shutting down server");
                graceful.shutdown().await;
                return Ok(());
            }
        }
    }
}

fn serve_conn(
    io: tokio::net::TcpStream,
) -> hyper::server::conn::http1::Connection<hyper_util::rt::TokioIo<tokio::net::TcpStream>, HokaySvc>
{
    hyper::server::conn::http1::Builder::new()
        // Allow weird clients (like netcat).
        .half_close(true)
        // Prevent port scanners, etc, from holding connections open.
        .header_read_timeout(std::time::Duration::from_secs(2))
        .timer(hyper_util::rt::TokioTimer::default())
        // Use a small buffer, since we don't really transfer much data.
        .max_buf_size(8 * 1024)
        .serve_connection(hyper_util::rt::TokioIo::new(io), HokaySvc)
}

struct HokaySvc;

impl hyper::service::Service<hyper::Request<hyper::body::Incoming>> for HokaySvc {
    type Response = hyper::Response<http_body_util::Empty<bytes::Bytes>>;
    type Error = hyper::Error;
    type Future = futures_util::future::Ready<Result<Self::Response, Self::Error>>;

    fn call(&self, _req: hyper::Request<hyper::body::Incoming>) -> Self::Future {
        futures_util::future::ready(Ok(hyper::Response::builder()
            .header(hyper::header::SERVER, format!("hokay/{VERSION}"))
            .status(hyper::StatusCode::NO_CONTENT)
            .body(http_body_util::Empty::<bytes::Bytes>::default())
            .unwrap()))
    }
}
