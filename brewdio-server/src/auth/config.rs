/// Auth configuration loaded from environment variables.
#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub allow_signups: bool,
    pub google_client_id: Option<String>,
    pub admin_email: Option<String>,
    pub admin_password: Option<String>,
    /// When true, all auth checks are bypassed (WebSocket connections, etc.)
    pub no_auth: bool,
}

impl AuthConfig {
    /// Load auth configuration from environment variables.
    /// If BREWDIO_JWT_SECRET is not set, generates a random one and logs a warning.
    pub fn from_env() -> Self {
        let jwt_secret = match std::env::var("BREWDIO_JWT_SECRET") {
            Ok(s) if !s.is_empty() => s,
            _ => {
                let secret: String = (0..64)
                    .map(|_| {
                        let idx = rand_byte() % 62;
                        match idx {
                            0..=9 => (b'0' + idx) as char,
                            10..=35 => (b'a' + idx - 10) as char,
                            _ => (b'A' + idx - 36) as char,
                        }
                    })
                    .collect();
                eprintln!("[auth] WARNING: BREWDIO_JWT_SECRET not set, using random secret. Tokens will not survive server restart.");
                secret
            }
        };

        let allow_signups = std::env::var("BREWDIO_ALLOW_SIGNUPS")
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);

        let google_client_id = std::env::var("BREWDIO_GOOGLE_CLIENT_ID").ok().filter(|s| !s.is_empty());
        let admin_email = std::env::var("BREWDIO_ADMIN_EMAIL").ok().filter(|s| !s.is_empty());
        let admin_password = std::env::var("BREWDIO_ADMIN_PASSWORD").ok().filter(|s| !s.is_empty());

        Self {
            jwt_secret,
            allow_signups,
            google_client_id,
            admin_email,
            admin_password,
            no_auth: false,
        }
    }

    pub fn google_enabled(&self) -> bool {
        self.google_client_id.is_some()
    }
}

/// Simple random byte using getrandom (available on all platforms).
fn rand_byte() -> u8 {
    let mut buf = [0u8; 1];
    getrandom(&mut buf);
    buf[0]
}

fn getrandom(buf: &mut [u8]) {
    // Use std's random fill which delegates to OS randomness
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let state = RandomState::new();
    for chunk in buf.chunks_mut(8) {
        let val = state.build_hasher().finish().to_ne_bytes();
        let len = chunk.len().min(8);
        chunk[..len].copy_from_slice(&val[..len]);
    }
}
