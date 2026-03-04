use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GoogleTokenInfo {
    pub sub: String,
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    pub aud: String,
}

/// Verify a Google ID token by calling Google's tokeninfo endpoint.
/// Returns the token info if valid and the audience matches our client ID.
pub async fn verify_google_token(
    id_token: &str,
    expected_client_id: &str,
) -> Result<GoogleTokenInfo, String> {
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        id_token
    );

    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to verify Google token: {}", e))?;

    if !resp.status().is_success() {
        return Err("Invalid Google ID token".to_string());
    }

    let info: GoogleTokenInfo = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Google token info: {}", e))?;

    if info.aud != expected_client_id {
        return Err("Google token audience mismatch".to_string());
    }

    Ok(info)
}
