#![cfg(feature = "web")]

use airust::web::{
    bots::crawler::{public_address, public_url},
    security::SecurityConfig,
};

#[test]
fn patched_url_origin_does_not_recurse_through_nested_blob_schemes() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let value = format!("{}https://example.com/", "blob:".repeat(10_000));
            let parsed = url::Url::parse(&value).unwrap();
            assert!(matches!(parsed.origin(), url::Origin::Opaque(_)));
            let ordinary = url::Url::parse("blob:https://example.com/item").unwrap();
            assert!(matches!(ordinary.origin(), url::Origin::Tuple(..)));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn nested_blob_inputs_are_rejected_by_web_origin_and_crawler_policies() {
    let value = format!("{}https://example.com/", "blob:".repeat(10_000));
    assert!(public_url(&value).is_err());
    let config =
        SecurityConfig::new("127.0.0.1".parse().unwrap(), 7070, None, vec![], false).unwrap();
    assert_eq!(
        config.authorize(
            &headers("localhost:7070", Some(&value), None),
            "/api/query",
            false
        ),
        Err(StatusCode::FORBIDDEN)
    );
}
use axum::http::{HeaderMap, HeaderValue, StatusCode};

fn headers(host: &str, origin: Option<&str>, token: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("host", HeaderValue::from_str(host).unwrap());
    if let Some(origin) = origin {
        headers.insert("origin", HeaderValue::from_str(origin).unwrap());
    }
    if let Some(token) = token {
        headers.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
    }
    headers
}

#[test]
fn remote_bind_requires_valid_authentication_configuration() {
    assert!(SecurityConfig::new("0.0.0.0".parse().unwrap(), 7070, None, vec![], false).is_err());
    assert!(SecurityConfig::new(
        "127.0.0.1".parse().unwrap(),
        7070,
        Some("short".to_owned()),
        vec![],
        false
    )
    .is_err());
    assert!(SecurityConfig::new("127.0.0.1".parse().unwrap(), 65535, None, vec![], false).is_err());
    assert!(SecurityConfig::new(
        "127.0.0.1".parse().unwrap(),
        7070,
        None,
        vec!["*".to_owned()],
        false
    )
    .is_err());
}

#[test]
fn host_origin_and_fetch_site_checks_reject_rebinding_and_browser_cross_site_access() {
    let config =
        SecurityConfig::new("127.0.0.1".parse().unwrap(), 7070, None, vec![], false).unwrap();
    assert!(config
        .authorize(
            &headers("localhost:7070", Some("http://localhost:7070"), None),
            "/api/query",
            false
        )
        .is_ok());
    assert_eq!(
        config.authorize(
            &headers("attacker.example:7070", None, None),
            "/api/query",
            false
        ),
        Err(StatusCode::FORBIDDEN)
    );
    assert_eq!(
        config.authorize(
            &headers("localhost:7070", Some("https://attacker.example"), None),
            "/api/query",
            false
        ),
        Err(StatusCode::FORBIDDEN)
    );
    assert_eq!(
        config.authorize(
            &headers("localhost:7070", Some("null"), None),
            "/api/query",
            false
        ),
        Err(StatusCode::FORBIDDEN)
    );
    assert_eq!(
        config.authorize(
            &headers("localhost:7070", Some("http://localhost:9999"), None),
            "/api/query",
            false
        ),
        Err(StatusCode::FORBIDDEN)
    );
    let mut cross_site = headers("localhost:7070", None, None);
    cross_site.insert("sec-fetch-site", HeaderValue::from_static("cross-site"));
    assert_eq!(
        config.authorize(&cross_site, "/api/query", false),
        Err(StatusCode::FORBIDDEN)
    );
}

#[test]
fn bearer_authentication_covers_api_and_websocket_without_query_string_secrets() {
    let token = "a_secure_session_token_0123456789abcdef";
    let config = SecurityConfig::new(
        "0.0.0.0".parse().unwrap(),
        7070,
        Some(token.to_owned()),
        vec![],
        false,
    )
    .unwrap();
    assert_eq!(
        config.authorize(&headers("localhost:7070", None, None), "/api/status", false),
        Err(StatusCode::UNAUTHORIZED)
    );
    assert_eq!(
        config.authorize(
            &headers("localhost:7070", None, Some("incorrect")),
            "/api/status",
            false
        ),
        Err(StatusCode::UNAUTHORIZED)
    );
    assert!(config
        .authorize(
            &headers("localhost:7070", None, Some(token)),
            "/api/status",
            false
        )
        .is_ok());
    let mut websocket = headers("localhost:7071", Some("http://localhost:7070"), None);
    websocket.insert(
        "sec-websocket-protocol",
        HeaderValue::from_str(&format!("airust, token.{token}")).unwrap(),
    );
    assert!(config.authorize(&websocket, "/ws/console", true).is_ok());
    assert_eq!(
        config.authorize(
            &headers(
                "localhost:7071",
                Some("https://attacker.example"),
                Some(token)
            ),
            "/ws/console",
            true
        ),
        Err(StatusCode::FORBIDDEN)
    );
}

#[test]
fn administrative_apis_require_explicit_opt_in() {
    let config =
        SecurityConfig::new("127.0.0.1".parse().unwrap(), 7070, None, vec![], false).unwrap();
    for path in [
        "/api/files",
        "/api/files/write",
        "/api/files/db/execute",
        "/api/knowledge/load",
        "/api/knowledge/save",
    ] {
        assert_eq!(
            config.authorize(&headers("localhost:7070", None, None), path, false),
            Err(StatusCode::FORBIDDEN)
        );
    }
    let enabled =
        SecurityConfig::new("127.0.0.1".parse().unwrap(), 7070, None, vec![], true).unwrap();
    assert!(enabled
        .authorize(&headers("localhost:7070", None, None), "/api/files", false)
        .is_ok());
}

#[test]
fn crawler_rejects_private_addresses_and_unsafe_url_schemes() {
    for address in [
        "127.0.0.1",
        "10.0.0.1",
        "169.254.169.254",
        "100.100.100.200",
        "192.168.1.1",
        "198.18.0.1",
        "0.0.0.0",
        "255.255.255.255",
        "::1",
        "fe80::1",
        "fc00::1",
        "::ffff:127.0.0.1",
        "2001:db8::1",
    ] {
        assert!(!public_address(address.parse().unwrap()), "{address}");
    }
    for address in ["1.1.1.1", "8.8.8.8", "2606:4700:4700::1111"] {
        assert!(public_address(address.parse().unwrap()), "{address}");
    }
    for url in [
        "file:///etc/passwd",
        "ftp://example.com",
        "http://localhost/",
        "http://service.localhost/",
        "http://127.0.0.1/",
        "http://[::1]/",
        "https://user:password@example.com/",
    ] {
        assert!(public_url(url).is_err(), "{url}");
    }
    assert!(public_url("https://example.com/path").is_ok());
}
