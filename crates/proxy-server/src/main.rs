use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::Arc,
    time::Duration,
};

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{
    body::Incoming,
    header::{HeaderName, HeaderValue, HOST, RETRY_AFTER},
    service::service_fn,
    Request, Response, StatusCode, Uri,
};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::{TokioExecutor, TokioIo},
    server::conn::auto,
};
use proxy_config::{load_and_validate, model::ProxyConfig};
use proxy_ratelimit::{Decision, Policy, RateLimiter};
use tokio::net::TcpListener;
use tokio_rustls::{
    rustls::{
        pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
        ServerConfig,
    },
    TlsAcceptor,
};

type BoxError = Box<dyn Error + Send + Sync>;
type ProxyBody = BoxBody<Bytes, BoxError>;

#[derive(Clone)]
struct ProxyState {
    hostname: String,
    web: SocketAddr,
    app: SocketAddr,
    target_header: HeaderName,
    client: Client<HttpConnector, ProxyBody>,
    limiter: Option<Arc<RateLimiter>>,
}

fn main() -> Result<(), BoxError> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "configs/edge.toml".to_owned());
    let config = load_and_validate(path)?;

    let mut runtime = tokio::runtime::Builder::new_multi_thread();
    runtime.enable_all();
    if config.proxy.worker_threads != 0 {
        runtime.worker_threads(config.proxy.worker_threads);
    }
    runtime.build()?.block_on(run(config))
}

async fn run(config: ProxyConfig) -> Result<(), BoxError> {
    let tls = load_tls(&config.tls.certificate, &config.tls.private_key)?;
    let listener = TcpListener::bind(&config.tls.listen).await?;
    let client = Client::builder(TokioExecutor::new()).build_http();
    let limiter = if config.rate_limit.enabled {
        let limit = &config.rate_limit;
        let policy = Policy {
            max_requests: limit.max_requests,
            window: Duration::from_secs(limit.window_seconds),
            strike_window: Duration::from_secs(limit.strike_window_seconds),
            strikes_before_block: limit.strikes_before_block,
            block: Duration::from_secs(limit.block_seconds),
            max_tracked_ips: limit.max_tracked_ips,
        };
        Some(Arc::new(RateLimiter::new(policy).map_err(|error| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, error)
        })?))
    } else {
        None
    };
    let state = ProxyState {
        hostname: config.tls.hostname.to_ascii_lowercase(),
        web: config.upstreams.web.parse()?,
        app: config.upstreams.app.parse()?,
        target_header: config.routing.target_header.parse()?,
        client,
        limiter,
    };
    eprintln!(
        "{} listening on {}",
        config.proxy.name,
        listener.local_addr()?
    );

    loop {
        let (stream, peer) = listener.accept().await?;
        let tls = tls.clone();
        let state = state.clone();
        tokio::spawn(async move {
            let stream = match tls.accept(stream).await {
                Ok(stream) => stream,
                Err(error) => {
                    eprintln!("TLS handshake from {peer} failed: {error}");
                    return;
                }
            };
            if stream
                .get_ref()
                .1
                .server_name()
                .is_some_and(|name| !name.eq_ignore_ascii_case(&state.hostname))
            {
                eprintln!("TLS SNI from {peer} does not match {}", state.hostname);
                return;
            }
            let service = service_fn(move |request| forward(request, peer.ip(), state.clone()));
            if let Err(error) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(TokioIo::new(stream), service)
                .await
            {
                eprintln!("client {peer}: {error}");
            }
        });
    }
}

fn load_tls(certificate: &str, private_key: &str) -> Result<TlsAcceptor, BoxError> {
    let certificates =
        CertificateDer::pem_file_iter(Path::new(certificate))?.collect::<Result<Vec<_>, _>>()?;
    if certificates.is_empty() {
        return Err("TLS certificate file is empty".into());
    }
    let key = PrivateKeyDer::from_pem_file(Path::new(private_key))?;
    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates, key)?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(TlsAcceptor::from(Arc::new(config)))
}

async fn forward(
    mut request: Request<Incoming>,
    peer_ip: IpAddr,
    state: ProxyState,
) -> Result<Response<ProxyBody>, std::convert::Infallible> {
    let header_host = request.headers().get(HOST);
    let authority = request.uri().authority().map(|value| value.as_str());
    if header_host.is_some_and(|value| !matches_hostname(value.to_str().ok(), &state.hostname))
        || authority.is_some_and(|value| !matches_hostname(Some(value), &state.hostname))
        || (header_host.is_none() && authority.is_none())
    {
        return Ok(error_response(StatusCode::MISDIRECTED_REQUEST));
    }
    let host = header_host
        .and_then(|value| value.to_str().ok())
        .or(authority)
        .expect("hostname was checked")
        .to_owned();

    if let Some(limiter) = &state.limiter {
        match limiter.check(peer_ip) {
            Decision::Allowed => {}
            Decision::Limited { retry_after } | Decision::Blacklisted { retry_after } => {
                let mut response = error_response(StatusCode::TOO_MANY_REQUESTS);
                let seconds = retry_after
                    .as_secs()
                    .saturating_add(u64::from(retry_after.subsec_nanos() > 0))
                    .max(1);
                response.headers_mut().insert(
                    RETRY_AFTER,
                    HeaderValue::from_str(&seconds.to_string())
                        .expect("integer is a valid header value"),
                );
                return Ok(response);
            }
        }
    }

    let upstream = select_upstream(
        request.headers().get(&state.target_header),
        state.web,
        state.app,
    );
    let path = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");
    let uri: Uri = match format!("http://{upstream}{path}").parse() {
        Ok(uri) => uri,
        Err(_) => return Ok(error_response(StatusCode::BAD_REQUEST)),
    };
    *request.uri_mut() = uri;
    if !request.headers().contains_key(HOST) {
        request.headers_mut().insert(
            HOST,
            HeaderValue::from_str(&host).expect("validated hostname is a header value"),
        );
    }
    *request.version_mut() = hyper::Version::HTTP_11;
    request.headers_mut().insert(
        HeaderName::from_static("x-real-ip"),
        HeaderValue::from_str(&peer_ip.to_string()).expect("IP address is a valid header value"),
    );

    let request = request.map(|body| {
        body.map_err(|error| -> BoxError { Box::new(error) })
            .boxed()
    });
    match state.client.request(request).await {
        Ok(response) => Ok(response.map(|body| {
            body.map_err(|error| -> BoxError { Box::new(error) })
                .boxed()
        })),
        Err(error) => {
            eprintln!("upstream {upstream}: {error}");
            Ok(error_response(StatusCode::BAD_GATEWAY))
        }
    }
}

fn matches_hostname(host: Option<&str>, expected: &str) -> bool {
    host.map(|value| {
        value.eq_ignore_ascii_case(expected)
            || value.eq_ignore_ascii_case(&format!("{expected}:443"))
    })
    .unwrap_or(false)
}

fn select_upstream(target: Option<&HeaderValue>, web: SocketAddr, app: SocketAddr) -> SocketAddr {
    if target
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("App"))
    {
        app
    } else {
        web
    }
}

fn error_response(status: StatusCode) -> Response<ProxyBody> {
    Response::builder()
        .status(status)
        .body(
            Full::new(Bytes::new())
                .map_err(|never| match never {})
                .boxed(),
        )
        .expect("static response is valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{
        client::conn::{http1 as client_http1, http2 as client_http2},
        server::conn::{http1 as server_http1, http2 as server_http2},
    };
    use tokio_rustls::{
        rustls::{pki_types::ServerName, ClientConfig, RootCertStore},
        TlsConnector,
    };

    #[test]
    fn app_header_routes_to_app_and_absence_routes_to_web() {
        let web = "127.0.0.1:8081".parse().unwrap();
        let app = "127.0.0.1:8082".parse().unwrap();
        assert_eq!(select_upstream(None, web, app), web);
        assert_eq!(
            select_upstream(Some(&HeaderValue::from_static("App")), web, app),
            app
        );
        assert_eq!(
            select_upstream(Some(&HeaderValue::from_static("app")), web, app),
            app
        );
        assert_eq!(
            select_upstream(Some(&HeaderValue::from_static("Web")), web, app),
            web
        );
    }

    async fn upstream(label: &'static str) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let service = service_fn(move |request: Request<Incoming>| async move {
                let (parts, body) = request.into_parts();
                let body = body.collect().await.unwrap().to_bytes();
                let real_ip = parts.headers.get("x-real-ip").unwrap().to_str().unwrap();
                let custom = parts.headers.get("x-test").unwrap().to_str().unwrap();
                let target = parts
                    .headers
                    .get("x-client-type")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("none");
                let host = parts.headers.get(HOST).unwrap().to_str().unwrap();
                let answer = format!(
                    "{label}|{}|{host}|{real_ip}|{custom}|{target}|{}",
                    parts.uri,
                    String::from_utf8_lossy(&body)
                );
                Ok::<_, std::convert::Infallible>(Response::new(Full::new(Bytes::from(answer))))
            });
            server_http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service)
                .await
                .unwrap();
        });
        address
    }

    #[tokio::test]
    async fn forwards_header_body_and_real_ip_to_selected_upstream() {
        let web = upstream("web").await;
        let app = upstream("app").await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let proxy_address = listener.local_addr().unwrap();
        let state = ProxyState {
            hostname: "www.typenull.xyz".to_owned(),
            web,
            app,
            target_header: HeaderName::from_static("x-client-type"),
            client: Client::builder(TokioExecutor::new()).build_http(),
            limiter: None,
        };
        tokio::spawn(async move {
            for _ in 0..2 {
                let (stream, peer) = listener.accept().await.unwrap();
                let state = state.clone();
                tokio::spawn(async move {
                    server_http1::Builder::new()
                        .serve_connection(
                            TokioIo::new(stream),
                            service_fn(move |request| forward(request, peer.ip(), state.clone())),
                        )
                        .await
                        .unwrap();
                });
            }
        });

        for (target, expected) in [(None, "web"), (Some("App"), "app")] {
            let stream = tokio::net::TcpStream::connect(proxy_address).await.unwrap();
            let (mut sender, connection) =
                client_http1::handshake(TokioIo::new(stream)).await.unwrap();
            tokio::spawn(async move { connection.await.unwrap() });
            let mut request = Request::builder()
                .method("POST")
                .uri("/path?q=1")
                .header(HOST, "www.typenull.xyz")
                .header("x-test", "unchanged")
                .header("x-real-ip", "203.0.113.100");
            if let Some(value) = target {
                request = request.header("x-client-type", value);
            }
            let response = sender
                .send_request(
                    request
                        .body(Full::new(Bytes::from_static(b"payload")))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(
                body.as_ref(),
                format!(
                    "{expected}|/path?q=1|www.typenull.xyz|127.0.0.1|unchanged|{}|payload",
                    target.unwrap_or("none")
                )
                .as_bytes()
            );
        }
    }

    #[tokio::test]
    async fn http2_authority_is_forwarded_as_original_host() {
        let web = upstream("web").await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let state = ProxyState {
            hostname: "www.typenull.xyz".to_owned(),
            web,
            app: web,
            target_header: HeaderName::from_static("x-client-type"),
            client: Client::builder(TokioExecutor::new()).build_http(),
            limiter: None,
        };
        tokio::spawn(async move {
            let (stream, peer) = listener.accept().await.unwrap();
            server_http2::Builder::new(TokioExecutor::new())
                .serve_connection(
                    TokioIo::new(stream),
                    service_fn(move |request| forward(request, peer.ip(), state.clone())),
                )
                .await
                .unwrap();
        });
        let stream = tokio::net::TcpStream::connect(address).await.unwrap();
        let (mut sender, connection) = client_http2::Builder::new(TokioExecutor::new())
            .handshake(TokioIo::new(stream))
            .await
            .unwrap();
        tokio::spawn(async move { connection.await.unwrap() });
        let request = Request::builder()
            .uri("https://www.typenull.xyz/h2")
            .header("x-test", "unchanged")
            .body(Full::new(Bytes::new()))
            .unwrap();
        let response = sender.send_request(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            body.as_ref(),
            b"web|/h2|www.typenull.xyz|127.0.0.1|unchanged|none|"
        );
        let mismatched = Request::builder()
            .uri("https://other.example/h2")
            .header(HOST, "www.typenull.xyz")
            .body(Full::new(Bytes::new()))
            .unwrap();
        let response = sender.send_request(mismatched).await.unwrap();
        assert_eq!(response.status(), StatusCode::MISDIRECTED_REQUEST);
    }

    #[tokio::test]
    async fn over_limit_request_returns_429_and_retry_after() {
        let web = upstream("web").await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let limiter = Arc::new(
            RateLimiter::new(Policy {
                max_requests: 1,
                window: Duration::from_secs(10),
                strike_window: Duration::from_secs(60),
                strikes_before_block: 3,
                block: Duration::from_secs(10),
                max_tracked_ips: 64,
            })
            .unwrap(),
        );
        let state = ProxyState {
            hostname: "www.typenull.xyz".to_owned(),
            web,
            app: web,
            target_header: HeaderName::from_static("x-client-type"),
            client: Client::builder(TokioExecutor::new()).build_http(),
            limiter: Some(limiter),
        };
        tokio::spawn(async move {
            let (stream, peer) = listener.accept().await.unwrap();
            server_http1::Builder::new()
                .serve_connection(
                    TokioIo::new(stream),
                    service_fn(move |request| forward(request, peer.ip(), state.clone())),
                )
                .await
                .unwrap();
        });
        let stream = tokio::net::TcpStream::connect(address).await.unwrap();
        let (mut sender, connection) = client_http1::handshake(TokioIo::new(stream)).await.unwrap();
        tokio::spawn(async move { connection.await.unwrap() });
        for expected in [StatusCode::OK, StatusCode::TOO_MANY_REQUESTS] {
            let request = Request::builder()
                .uri("/")
                .header(HOST, "www.typenull.xyz")
                .header("x-test", "rate-limit-test")
                .body(Full::new(Bytes::new()))
                .unwrap();
            let response = sender.send_request(request).await.unwrap();
            assert_eq!(response.status(), expected);
            if expected == StatusCode::TOO_MANY_REQUESTS {
                assert_eq!(response.headers().get(RETRY_AFTER).unwrap(), "10");
            }
            response.into_body().collect().await.unwrap();
        }
    }

    #[tokio::test]
    async fn loads_mounted_pem_files_and_negotiates_tls() {
        let rcgen::CertifiedKey { cert, signing_key } =
            rcgen::generate_simple_self_signed(vec!["www.typenull.xyz".to_owned()]).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let certificate = directory.path().join("fullchain.pem");
        let private_key = directory.path().join("privkey.pem");
        std::fs::write(&certificate, cert.pem()).unwrap();
        std::fs::write(&private_key, signing_key.serialize_pem()).unwrap();

        let acceptor =
            load_tls(certificate.to_str().unwrap(), private_key.to_str().unwrap()).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            acceptor.accept(stream).await.unwrap();
        });

        let mut roots = RootCertStore::empty();
        roots.add(cert.der().clone()).unwrap();
        let mut config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        let connector = TlsConnector::from(Arc::new(config));
        let stream = tokio::net::TcpStream::connect(address).await.unwrap();
        let tls = connector
            .connect(ServerName::try_from("www.typenull.xyz").unwrap(), stream)
            .await
            .unwrap();
        assert_eq!(
            tls.get_ref().1.alpn_protocol(),
            Some(b"http/1.1".as_slice())
        );
        server.await.unwrap();
    }
}
