use h2::client;
use hyper::Uri;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use structopt::StructOpt;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio_rustls::rustls::{ClientConfig, RootCertStore, ProtocolVersion};
use tokio_rustls::TlsConnector;
use tracing::{debug, error, info};
use tracing_subscriber;

#[derive(StructOpt, Debug)]
#[structopt(name = "http2_client")]
struct Opt {
    #[structopt(short, long, default_value = "https://example.com")]
    url: String,

    #[structopt(short, long, default_value = "10")]
    timeout: u64,

    #[structopt(long)]
    headers: Vec<String>,

    #[structopt(long)]
    output: Option<String>,

    #[structopt(long)]
    body: Option<String>,

    #[structopt(short, long)]
    verbose: bool,

    #[structopt(short, long, default_value = "3")]
    retries: u8,

    #[structopt(long)]
    follow_redirects: bool,

    #[structopt(short, long, default_value = "1")]
    concurrency: usize,

    #[structopt(short, long, default_value = "0")]
    delay: u64,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    if let Err(e) = tracing_subscriber::fmt::try_init() {
        eprintln!("Failed to initialize logger: {:?}", e);
    }

    let opt = Opt::from_args();

    // Reinitialize the logger with a debug level if verbose is enabled
    if opt.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    // Parse the provided URL
    let uri = match opt.url.parse::<Uri>() {
        Ok(uri) => uri,
        Err(e) => {
            error!("Failed to parse URI: {}", e);
            return;
        }
    };

    let authority = match uri.authority() {
        Some(authority) => authority,
        None => {
            error!("URI does not have authority");
            return;
        }
    };
    let domain = authority.host();

    // Configure TLS
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        tokio_rustls::rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_protocol_versions(&[ProtocolVersion::TLSv1_2, ProtocolVersion::TLSv1_3])
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let tcp = match TcpStream::connect((authority.host(), 443)).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to connect to server: {}", e);
            return;
        }
    };
    let tls = match connector.connect(domain.try_into().unwrap(), tcp).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to perform TLS handshake: {}", e);
            return;
        }
    };

    let (mut client, connection) = match client::handshake(tls).await {
        Ok((client, connection)) => (client, connection),
        Err(e) => {
            error!("Failed to perform HTTP/2 handshake: {}", e);
            return;
        }
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Error in connection: {:?}", e);
        }
    });

    let semaphore = Arc::new(Semaphore::new(opt.concurrency));
    let stream_counter = Arc::new(AtomicU32::new(1));
    let path = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

    for _ in 0..opt.retries {
        let semaphore = Arc::clone(&semaphore);
        let stream_counter = Arc::clone(&stream_counter);
        let headers = opt.headers.clone();
        let path = path.to_string();

        let permit = semaphore.acquire().await.unwrap();
        tokio::spawn(async move {
            let stream_id = stream_counter.fetch_add(2, Ordering::SeqCst);
            let mut request = client::Request::builder()
                .method("GET")
                .uri(&opt.url)
                .version(h2::Version::HTTP_2)
                .header("User-Agent", "h2-client/0.1");

            for header in headers {
                let parts: Vec<&str> = header.splitn(2, ':').collect();
                if parts.len() == 2 {
                    request = request.header(parts[0].trim(), parts[1].trim());
                }
            }

            let (response, mut send_stream) = client.send_request(request.body(()).unwrap(), true).unwrap();

            debug!("[{}] Sent HEADERS on stream {}", stream_id, stream_id);

            tokio::time::sleep(Duration::from_millis(opt.delay)).await;

            send_stream.send_reset(h2::Reason::CANCEL);
            debug!("[{}] Sent RST_STREAM on stream {}", stream_id, stream_id);

            match response.await {
                Ok(res) => {
                    info!("[{}] Response: {}", stream_id, res.status());
                    let mut body = res.into_body();

                    while let Some(chunk) = body.data().await {
                        match chunk {
                            Ok(data) => {
                                debug!("[{}] Body chunk: {:?}", stream_id, data);
                            }
                            Err(e) => {
                                error!("[{}] Error reading body chunk: {:?}", stream_id, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("[{}] Error awaiting response: {:?}", stream_id, e);
                }
            }

            drop(permit);
            tokio::time::sleep(Duration::from_millis(opt.delay)).await;
        })
        .await
        .unwrap();
    }
}
