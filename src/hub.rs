use crate::app_config;
use google_drive3::hyper;
use google_drive3::hyper::client::HttpConnector;
use google_drive3::hyper_rustls::HttpsConnector;
use google_drive3::hyper_rustls::HttpsConnectorBuilder;
use google_drive3::oauth2;
use google_drive3::oauth2::authenticator::Authenticator;
use google_drive3::oauth2::authenticator_delegate::InstalledFlowDelegate;
use google_drive3::DriveHub;
use std::future::Future;
use std::io;
use std::ops::Deref;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

/// Callback invoked with the OAuth consent url that the user must visit.
///
/// The CLI prints it to stdout; a GUI can forward it to the frontend and open
/// a browser instead.
pub type UrlPresenter = Arc<dyn Fn(String) + Send + Sync>;

/// The OAuth scopes gdrive requests when adding an account.
pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/drive",
    "https://www.googleapis.com/auth/drive.metadata.readonly",
];

/// The local port that receives the OAuth redirect from Google.
pub const REDIRECT_PORT: u16 = 8085;

pub struct HubConfig {
    pub secret: oauth2::ApplicationSecret,
    pub tokens_path: PathBuf,
}

pub struct Hub(DriveHub<HttpsConnector<HttpConnector>>);

impl Deref for Hub {
    type Target = DriveHub<HttpsConnector<HttpConnector>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hub {
    pub async fn new(auth: Auth) -> Hub {
        let connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();

        let http_client = hyper::Client::builder().build(connector);

        Hub(google_drive3::DriveHub::new(http_client, auth.0))
    }
}

pub struct Auth(pub Authenticator<HttpsConnector<HttpConnector>>);

impl Deref for Auth {
    type Target = Authenticator<HttpsConnector<HttpConnector>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Auth {
    pub async fn new(
        config: &app_config::Secret,
        tokens_path: &PathBuf,
    ) -> Result<Auth, io::Error> {
        Auth::new_with_presenter(config, tokens_path, Arc::new(print_user_url)).await
    }

    pub async fn new_with_presenter(
        config: &app_config::Secret,
        tokens_path: &PathBuf,
        presenter: UrlPresenter,
    ) -> Result<Auth, io::Error> {
        let secret = oauth2_secret(config);
        let delegate = Box::new(AuthDelegate { presenter });

        let auth = oauth2::InstalledFlowAuthenticator::builder(
            secret,
            oauth2::InstalledFlowReturnMethod::HTTPPortRedirect(REDIRECT_PORT),
        )
        .persist_tokens_to_disk(tokens_path)
        .flow_delegate(delegate)
        .build()
        .await?;

        Ok(Auth(auth))
    }
}

fn oauth2_secret(config: &app_config::Secret) -> oauth2::ApplicationSecret {
    oauth2::ApplicationSecret {
        client_id: config.client_id.clone(),
        client_secret: config.client_secret.clone(),
        token_uri: String::from("https://oauth2.googleapis.com/token"),
        auth_uri: String::from("https://accounts.google.com/o/oauth2/auth"),
        redirect_uris: vec![String::from("urn:ietf:wg:oauth:2.0:oob")],
        project_id: None,
        client_email: None,
        auth_provider_x509_cert_url: Some(String::from(
            "https://www.googleapis.com/oauth2/v1/certs",
        )),
        client_x509_cert_url: None,
    }
}

struct AuthDelegate {
    presenter: UrlPresenter,
}

impl InstalledFlowDelegate for AuthDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        _need_code: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        let presenter = self.presenter.clone();
        let url = url.to_string();

        Box::pin(async move {
            presenter(url);
            Ok(String::new())
        })
    }
}

/// The default presenter used by the CLI: print the consent url to stdout.
pub fn print_user_url(url: String) {
    println!();
    println!();
    println!("Gdrive requires permissions to manage your files on Google Drive.");
    println!("Open the url in your browser and follow the instructions:");
    println!("{}", url);
}

/// Check whether the OAuth redirect port is available, so the caller can warn
/// the user before starting a flow that would fail.
pub fn redirect_port_is_free() -> bool {
    std::net::TcpListener::bind(("127.0.0.1", REDIRECT_PORT)).is_ok()
}
