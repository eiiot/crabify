use anyhow::{Context, Result};
use rspotify::prelude::*;
use rspotify::{scopes, AuthCodePkceSpotify, Credentials, OAuth};
use std::path::PathBuf;

use crate::config::AppConfig;

const REDIRECT_URI: &str = "http://127.0.0.1:8888/callback";
const SCOPES: &[&str] = &[
    "user-read-playback-state",
    "user-modify-playback-state",
    "user-read-currently-playing",
    "user-library-read",
    "user-library-modify",
    "playlist-read-private",
    "playlist-read-collaborative",
];

fn token_cache_path() -> Result<PathBuf> {
    Ok(AppConfig::config_dir()?.join(".spotify_token_cache.json"))
}

pub async fn authenticate() -> Result<AuthCodePkceSpotify> {
    let config = AppConfig::load()?;

    let creds = Credentials::new_pkce(&config.client_id);

    let oauth = OAuth {
        redirect_uri: REDIRECT_URI.to_string(),
        scopes: scopes!(
            "user-read-playback-state",
            "user-modify-playback-state",
            "user-read-currently-playing",
            "user-library-read",
            "user-library-modify",
            "playlist-read-private",
            "playlist-read-collaborative"
        ),
        ..Default::default()
    };

    let config = rspotify::Config {
        token_cached: true,
        cache_path: token_cache_path()?,
        token_refreshing: true,
        ..Default::default()
    };

    let mut spotify = AuthCodePkceSpotify::with_config(creds, oauth.clone(), config);

    // Try to load cached token
    let token_path = token_cache_path()?;
    if token_path.exists() {
        // Attempt to read cached token
        let token_data = std::fs::read_to_string(&token_path).ok();
        if let Some(data) = token_data {
            if let Ok(token) = serde_json::from_str::<rspotify::Token>(&data) {
                *spotify.token.lock().await.unwrap() = Some(token);
                // Try a simple API call to verify the token works
                if spotify.current_user().await.is_ok() {
                    return Ok(spotify);
                }
                // Token is invalid, proceed with fresh auth
            }
        }
    }

    // Perform fresh OAuth PKCE flow
    let auth_url = spotify.get_authorize_url(None)?;

    // Start local HTTP server for callback
    let server = tiny_http::Server::http("127.0.0.1:8888")
        .map_err(|e| anyhow::anyhow!("Failed to start callback server: {}", e))?;

    // Open browser for authentication
    eprintln!("Opening browser for Spotify authentication...");
    if open::that(&auth_url).is_err() {
        eprintln!("Could not open browser. Please visit this URL manually:\n{}", auth_url);
    }

    // Wait for the callback
    let request = server
        .recv()
        .context("Failed to receive OAuth callback")?;

    let url = format!("http://127.0.0.1:8888{}", request.url());
    let parsed = url::Url::parse(&url).context("Failed to parse callback URL")?;

    let code = parsed
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string())
        .context("No authorization code in callback")?;

    // Send response to browser
    let response = tiny_http::Response::from_string(
        "<html><body><h1>Authentication successful!</h1><p>You can close this tab and return to the terminal.</p></body></html>"
    )
    .with_header("Content-Type: text/html".parse::<tiny_http::Header>().unwrap());
    let _ = request.respond(response);

    // Exchange code for token
    spotify.request_token(&code).await?;

    // Cache the token
    if let Some(token) = spotify.token.lock().await.unwrap().as_ref() {
        let token_json = serde_json::to_string_pretty(token)?;
        std::fs::write(token_cache_path()?, token_json)?;
    }

    Ok(spotify)
}
