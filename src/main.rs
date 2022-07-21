#![deny(warnings, rust_2018_idioms)]

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about, version)]
struct Args {
    /// One or more addresses to listen on.
    #[clap(default_value = "0.0.0.0:8080")]
    addrs: Vec<std::net::SocketAddr>,
}

/// Runs a basic HTTP server that always returns 204 No Content.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let Args { addrs } = Args::parse();

    let (tx, rx) = tokio::sync::watch::channel(());
    let tasks = addrs
        .into_iter()
        .map(|addr| {
            let server = hyper::server::Server::try_bind(&addr)?
                // Allow weird clients (like netcat).
                .http1_half_close(true)
                // Prevent port scanners, etc, from holding connections open.
                .http1_header_read_timeout(std::time::Duration::from_secs(2))
                // Use a small buffer, since we don't really transfer much data.
                .http1_max_buf_size(8 * 1024)
                .tcp_nodelay(true);
            println!("Listening on {addr}");

            let mut rx = rx.clone();
            Ok::<_, hyper::Error>(tokio::spawn(
                server
                    .serve(hyper::service::make_service_fn(|_conn| async move {
                        Ok::<_, hyper::Error>(hyper::service::service_fn(|_req| async move {
                            Ok::<_, hyper::Error>(
                                hyper::Response::builder()
                                    .header(hyper::header::SERVER, format!("hokay/{VERSION}"))
                                    .status(hyper::StatusCode::NO_CONTENT)
                                    .body(hyper::Body::default())
                                    .unwrap(),
                            )
                        }))
                    }))
                    .with_graceful_shutdown(async move {
                        let _ = rx.changed().await;
                        println!("Closing {addr}");
                    }),
            ))
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
