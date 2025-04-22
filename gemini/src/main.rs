use aws_lc_rs::signature::{ECDSA_P384_SHA384_ASN1_SIGNING, EcdsaKeyPair};
use eyre::Context;
use fend_core::FendResult;
use rcgen::{CertificateParams, DistinguishedName, Ia5String, KeyPair, PKCS_ECDSA_P384_SHA384};
use rustls::{
	ServerConfig,
	pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, pem::PemObject},
	server::{Acceptor, ClientHello},
};
use std::{
	ffi,
	net::SocketAddr,
	sync::{Arc, LazyLock},
	time::Duration,
};
use time::OffsetDateTime;
use tokio::{
	io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
	net::{TcpListener, lookup_host},
	sync::RwLock,
	time::Instant,
	try_join,
};
use tokio_rustls::LazyConfigAcceptor;
use tokio_util::sync::CancellationToken;
use url::Url;

struct Config {
	cert_and_key: Option<(ffi::OsString, ffi::OsString)>,
	port: u16,
	host: String,
}

fn cli_args() -> eyre::Result<Config> {
	let mut config = Config {
		cert_and_key: None,
		port: 1965,
		host: "::".into(),
	};
	let mut args = std::env::args_os().skip(1);
	let mut cert = None;
	let mut key = None;
	while let Some(arg) = args.next() {
		match arg.to_str() {
			Some("--cert") => {
				let path = args
					.next()
					.ok_or(eyre::eyre!("Please provide a path to a certificate file"))?;
				cert = Some(path);
			}
			Some("--key") => {
				let path = args
					.next()
					.ok_or(eyre::eyre!("Please provide a path to a key file"))?;
				key = Some(path);
			}
			Some("--port") => {
				config.port = args
					.next()
					.ok_or(eyre::eyre!("Please provide a port number"))?
					.to_str()
					.ok_or(eyre::eyre!("Please provide a valid port number"))?
					.parse()
					.map_err(|_| eyre::eyre!("Please provide a valid port number"))?;
			}
			Some("--host") => {
				config.host = args
					.next()
					.ok_or(eyre::eyre!("Please provide a hostname"))?
					.to_str()
					.ok_or(eyre::eyre!("Please provide a valid hostname"))?
					.to_string();
			}
			Some("-h" | "--help") => {
				eyre::bail!(
					"Usage:
fend-gemini [options]

Options:
    --cert <path>     Path to a certificate file (default: disable TLS)
    --key <path>      Path to a key file (default: disable TLS)
    --port <port>     Port to listen on (default: 1965)
    --host <host>     Hostname to listen on (default: \"::\", i.e. any address)"
				);
			}
			_ => {
				eyre::bail!("Unknown argument: {}", arg.to_string_lossy());
			}
		}
	}
	match (cert, key) {
		(Some(cert), Some(key)) => {
			config.cert_and_key = Some((cert, key));
		}
		(None, None) => {
			eprintln!(
				"warning: no certificate and key provided, using a temporary self-signed certificate"
			);
		}
		_ => {
			eyre::bail!("Please provide both a certificate and a key");
		}
	}
	Ok(config)
}

fn format_socket_addr(addr: &SocketAddr) -> String {
	let formatted = format!("gemini://{addr}");
	let Ok(mut url) = Url::parse(&formatted) else {
		return formatted;
	};
	if url.port() == Some(1965) {
		if url.set_port(None).is_err() {
			return formatted;
		};
	}
	url.to_string()
}

fn self_signed_cert() -> eyre::Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
	let keypair = EcdsaKeyPair::generate(&ECDSA_P384_SHA384_ASN1_SIGNING)?;
	let pkcs8 = keypair.to_pkcs8v1()?;
	let pkcs8 = PrivatePkcs8KeyDer::from(pkcs8.as_ref());
	let rcgen_keypair = KeyPair::from_pkcs8_der_and_sign_algo(&pkcs8, &PKCS_ECDSA_P384_SHA384)?;
	let mut params = CertificateParams::default();
	params.distinguished_name = DistinguishedName::new();
	params
		.subject_alt_names
		.push(rcgen::SanType::DnsName(Ia5String::try_from("localhost")?));
	params.not_before = OffsetDateTime::now_utc();
	params.not_after = OffsetDateTime::now_utc()
		.checked_add(time::Duration::minutes(2))
		.unwrap();
	let certificate = params.self_signed(&rcgen_keypair)?;
	Ok((certificate.der().clone(), pkcs8.clone_key().into()))
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let config = cli_args()?;
	if let Some((cert_path, key_path)) = &config.cert_and_key {
		choose_server_config(None, Some((cert_path, key_path))).await?;
	}
	let results: Vec<_> = lookup_host((config.host, config.port)).await?.collect();
	let cert_and_key = if let Some((cert, key)) = Box::leak(Box::new(config.cert_and_key)) {
		Some((cert.as_os_str(), key.as_os_str()))
	} else {
		None
	};
	let mut handles = vec![];
	for addr in results {
		let formatted_addr = format_socket_addr(&addr);
		let listener = match TcpListener::bind(addr).await {
			Ok(listener) => listener,
			Err(e) => {
				eprintln!("failed to listen on {formatted_addr}: {e}");
				continue;
			}
		};
		eprintln!("listening on {formatted_addr}");
		handles.push(tokio::spawn(async move {
			loop {
				let (stream, client_addr) = match listener.accept().await {
					Ok(r) => r,
					Err(e) => {
						eprintln!("failed to accept connection: {e}");
						continue;
					}
				};
				eprintln!("accepted connection from {client_addr}");
				tokio::spawn(async move {
					if let Err(err) = accept(client_addr, cert_and_key, stream).await {
						eprintln!("Error: {err}");
					}
				});
			}
		}));
	}
	for h in handles {
		h.await?;
	}
	Ok(())
}

async fn accept(
	client_addr: SocketAddr,
	cert_and_key: Option<(&ffi::OsStr, &ffi::OsStr)>,
	stream: tokio::net::TcpStream,
) -> eyre::Result<()> {
	let start = LazyConfigAcceptor::new(Acceptor::default(), stream)
		.await
		.wrap_err_with(|| "TLS handshake failed")?;
	let client_hello = start.client_hello();
	let config = choose_server_config(Some(client_hello), cert_and_key).await?;
	let tls_stream = start.into_stream(config).await?;
	respond(client_addr, tls_stream).await
}

struct CachedConfig {
	config: Arc<ServerConfig>,
	creation: Instant,
}
static GLOBAL_CONFIG: LazyLock<RwLock<Option<CachedConfig>>> = LazyLock::new(|| RwLock::new(None));

async fn choose_server_config(
	_ch: Option<ClientHello<'_>>,
	cert_and_key: Option<(&ffi::OsStr, &ffi::OsStr)>,
) -> eyre::Result<Arc<ServerConfig>> {
	if let Some(CachedConfig { config, creation }) = GLOBAL_CONFIG.read().await.as_ref() {
		if creation.elapsed() < Duration::from_secs(60) {
			return Ok(config.clone());
		}
	}
	let mut write_guard = GLOBAL_CONFIG.write().await;
	async fn read_file(path: &ffi::OsStr) -> eyre::Result<Vec<u8>> {
		let mut data = vec![];
		tokio::fs::File::open(path)
			.await
			.wrap_err_with(|| format!("failed to open file {path:?}"))?
			.read_to_end(&mut data)
			.await
			.wrap_err_with(|| format!("failed to read file {path:?}"))?;
		Ok(data)
	}
	let (chain, key) = if let Some((cert_path, key_path)) = cert_and_key {
		let (cert, key) = try_join!(read_file(cert_path), read_file(key_path))?;
		let chain = CertificateDer::pem_slice_iter(&cert).collect::<Result<Vec<_>, _>>()?;
		let mut key = PrivateKeyDer::pem_slice_iter(&key).collect::<Vec<_>>();
		if key.len() != 1 {
			eyre::bail!(
				"Specified key file contains {} keys (1 key required)",
				key.len()
			);
		}
		let key = key.swap_remove(0)?;
		(chain, key)
	} else {
		let (cert, key) = self_signed_cert()?;
		(vec![cert], key)
	};
	let config = Arc::new(
		ServerConfig::builder()
			.with_no_client_auth()
			.with_single_cert(chain, key)?,
	);
	*write_guard = Some(CachedConfig {
		config: config.clone(),
		creation: Instant::now(),
	});
	Ok(config)
}

async fn respond(client_addr: SocketAddr, stream: impl AsyncRead + AsyncWrite) -> eyre::Result<()> {
	let (mut reader, mut writer) = tokio::io::split(stream);
	let mut buffer = Vec::with_capacity(1024);
	reader.read_buf(&mut buffer).await?;
	let input_string = String::from_utf8(buffer)?;
	let input_string = input_string.trim_end();
	eprintln!("received request from {client_addr}: {input_string}");
	let url = url::Url::parse(input_string)?;
	if url.scheme() != "gemini" {
		eprintln!("rejecting due to invalid URL scheme {:?}", url.scheme());
		writer.write_all(b"50 Invalid URL scheme").await?;
		return Ok(());
	}
	match url.query() {
		Some(query) => {
			let decoded = percent_encoding::percent_decode_str(query)
				.decode_utf8()?
				.into_owned();
			eprintln!("query: {decoded}");
			let response = evaluate_fend(&decoded).await.map_or_else(
				|e| format!("Error: {e}"),
				|r| format!("Result: {}", r.get_main_result()),
			);
			eprintln!("response: {response}");
			let response = format!(
				"20 text/gemini; charset=utf-8; lang=en\r\n# fend-gemini v{}\nInput: {}\n{}\n=> . Make another calculation",
				fend_core::get_version(),
				decoded,
				response,
			);
			writer.write_all(response.as_bytes()).await?;
		}
		None => {
			eprintln!("no query: returning prompt");
			writer.write_all(b"10 Enter a calculation\r\n").await?;
		}
	}
	Ok(())
}

struct Interrupt {
	token: CancellationToken,
}

impl fend_core::Interrupt for Interrupt {
	fn should_interrupt(&self) -> bool {
		self.token.is_cancelled()
	}
}

async fn evaluate_fend(input: &str) -> eyre::Result<FendResult> {
	let token = CancellationToken::new();
	let input = input.to_string();
	let token2 = token.child_token();
	let fend_task = tokio::task::spawn_blocking(move || {
		let mut ctx = fend_core::Context::new();
		ctx.set_random_u32_fn(rand::random);
		fend_core::evaluate_with_interrupt(&input, &mut ctx, &Interrupt { token: token2 })
			.map_err(|e| eyre::eyre!("{e}"))
	});
	let response = tokio::select! {
		res = fend_task => res??,
		_ = tokio::time::sleep(Duration::from_secs(10)) => {
			token.cancel();
			eyre::bail!("timed out after 10 seconds")
		}
	};
	Ok(response)
}
