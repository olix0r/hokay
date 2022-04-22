/// Runs a basic HTTP server that always returns 204 No Content.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = get_port_from_args()?;

    let (tx, rx) = tokio::sync::oneshot::channel();
    let mut task = tokio::spawn({
        let addr = std::net::SocketAddr::new([0, 0, 0, 0].into(), port);
        let server = hyper::server::Server::try_bind(&addr)?
            // Allow weird clients (like netcat).
            .http1_half_close(true)
            // Prevent port scanners, etc, from holding connections open.
            .http1_header_read_timeout(std::time::Duration::from_secs(2))
            // Use a small buffer, since we don't really transfer much data.
            .http1_max_buf_size(8 * 1024);

        server
            .serve(hyper::service::make_service_fn(|_conn| async {
                Ok::<_, hyper::Error>(hyper::service::service_fn(|_req| async {
                    Ok::<_, hyper::Error>(
                        hyper::Response::builder()
                            .header(hyper::header::SERVER, "hokay")
                            .status(hyper::StatusCode::NO_CONTENT)
                            .body(hyper::Body::default())
                            .unwrap(),
                    )
                }))
            }))
            .with_graceful_shutdown(async move {
                let _ = rx.await;
            })
    });

    let mut interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {
        res = (&mut task) => match res {
            Ok(res) => match res {
                Ok(()) => return Err("server exited unexpectedly".into()),
                Err(error) => return Err(format!("server exited unexpectedly: {error}").into()),
            },
            Err(error) => return Err(format!("server exited unexpectedly: {error}").into()),
        },
        _ = interrupt.recv() => {
            let _ = tx.send(());
            if let Err(error) = task.await {
                return Err(format!("server exited unexpectedly: {error}").into());
            }
        },
        _ = terminate.recv() => {
            let _ = tx.send(());
            if let Err(error) = task.await {
                return Err(format!("server exited unexpectedly: {error}").into());
            }
        },
    }

    Ok(())
}

fn get_port_from_args() -> Result<u16, Box<dyn std::error::Error>> {
    let mut args = std::env::args();

    match args.next() {
        Some(name) => {
            if let Some(p) = args.next() {
                if args.next().is_some() {
                    return Err(format!("Too many arguments\nUsage: {} [<port>]", name).into());
                }
                match p.parse::<u16>() {
                    Ok(port) => return Ok(port),
                    Err(error) => return Err(format!("Invalid port: {error}").into()),
                }
            }
        }
        None => {
            eprintln!("Process executed with no arguments");
        }
    }

    Ok(8080)
}
