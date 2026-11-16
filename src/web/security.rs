use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{collections::BTreeSet, net::IpAddr, sync::Arc};

#[derive(Clone)]
pub struct SecurityConfig {
    token: Option<String>,
    hosts: BTreeSet<String>,
    port: u16,
    console_port: u16,
    admin_enabled: bool,
}

impl SecurityConfig {
    pub fn new(
        bind: IpAddr,
        port: u16,
        token: Option<String>,
        hosts: Vec<String>,
        admin_enabled: bool,
    ) -> Result<Self, String> {
        if port == 0 || port == u16::MAX {
            return Err("Port must be between 1 and 65534".to_owned());
        }
        if token.as_ref().is_some_and(|value| {
            value.len() < 32
                || value.len() > 256
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
        }) {
            return Err("AIRUST_API_TOKEN must contain 32 to 256 ASCII letters, digits, hyphens, or underscores".to_owned());
        }
        if !bind.is_loopback() && token.is_none() {
            return Err("Non-loopback binding requires AIRUST_API_TOKEN".to_owned());
        }
        let mut allowed = BTreeSet::from([
            "localhost".to_owned(),
            "127.0.0.1".to_owned(),
            "::1".to_owned(),
            bind.to_string(),
        ]);
        for host in hosts {
            let host = host.trim().to_ascii_lowercase();
            if host.is_empty() || host.contains(['/', '@', '*', '\\']) {
                return Err("AIRUST_ALLOWED_HOSTS must contain explicit hostnames".to_owned());
            }
            let authority: axum::http::uri::Authority =
                host.parse().map_err(|_| "Invalid allowed hostname")?;
            if authority.port_u16().is_some() {
                return Err("AIRUST_ALLOWED_HOSTS entries must omit ports".to_owned());
            }
            allowed.insert(authority.host().trim_matches(['[', ']']).to_owned());
        }
        Ok(Self {
            token,
            hosts: allowed,
            port,
            console_port: port + 1,
            admin_enabled,
        })
    }

    pub fn from_environment(bind: IpAddr, port: u16) -> Result<Self, String> {
        let token = std::env::var("AIRUST_API_TOKEN").ok();
        let hosts = std::env::var("AIRUST_ALLOWED_HOSTS")
            .unwrap_or_default()
            .split(',')
            .filter(|host| !host.trim().is_empty())
            .map(str::to_owned)
            .collect();
        Self::new(
            bind,
            port,
            token,
            hosts,
            std::env::var("AIRUST_ENABLE_ADMIN").as_deref() == Ok("1"),
        )
    }

    pub fn authorize(
        &self,
        headers: &HeaderMap,
        path: &str,
        console: bool,
    ) -> Result<(), StatusCode> {
        let authority: axum::http::uri::Authority = headers
            .get(header::HOST)
            .and_then(|value| value.to_str().ok())
            .ok_or(StatusCode::BAD_REQUEST)?
            .parse()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let host = authority
            .host()
            .trim_matches(['[', ']'])
            .to_ascii_lowercase();
        let expected_port = if console {
            self.console_port
        } else {
            self.port
        };
        if !self.hosts.contains(&host) || authority.port_u16().unwrap_or(80) != expected_port {
            return Err(StatusCode::FORBIDDEN);
        }
        if let Some(origin) = headers.get(header::ORIGIN) {
            let origin = url::Url::parse(origin.to_str().map_err(|_| StatusCode::FORBIDDEN)?)
                .map_err(|_| StatusCode::FORBIDDEN)?;
            if !matches!(origin.scheme(), "http" | "https")
                || !origin.username().is_empty()
                || origin.password().is_some()
                || origin
                    .host_str()
                    .map(|value| value.trim_matches(['[', ']']).to_ascii_lowercase())
                    != Some(host)
                || !matches!(origin.port_or_known_default(), Some(port) if port == self.port || port == self.console_port)
            {
                return Err(StatusCode::FORBIDDEN);
            }
        }
        if headers
            .get("sec-fetch-site")
            .is_some_and(|value| value == "cross-site")
        {
            return Err(StatusCode::FORBIDDEN);
        }
        if !self.admin_enabled
            && (path.starts_with("/api/files")
                || path == "/api/knowledge/load"
                || path == "/api/knowledge/save")
        {
            return Err(StatusCode::FORBIDDEN);
        }
        if let Some(expected) = &self.token {
            if path.starts_with("/api/") || console {
                let bearer = headers
                    .get(header::AUTHORIZATION)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.strip_prefix("Bearer "));
                let websocket = headers
                    .get(header::SEC_WEBSOCKET_PROTOCOL)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| {
                        value
                            .split(',')
                            .find_map(|protocol| protocol.trim().strip_prefix("token."))
                    });
                let supplied = bearer
                    .or(if console { websocket } else { None })
                    .ok_or(StatusCode::UNAUTHORIZED)?;
                if !constant_time_equal(supplied.as_bytes(), expected.as_bytes()) {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
        Ok(())
    }
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

pub async fn protect(
    State((config, console)): State<(Arc<SecurityConfig>, bool)>,
    request: Request,
    next: Next,
) -> Response {
    if let Err(status) = config.authorize(request.headers(), request.uri().path(), console) {
        return (status, "Request rejected by server access policy").into_response();
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-content-type-options", "nosniff".parse().unwrap());
    response
        .headers_mut()
        .insert("referrer-policy", "no-referrer".parse().unwrap());
    response
        .headers_mut()
        .insert("x-frame-options", "DENY".parse().unwrap());
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}
