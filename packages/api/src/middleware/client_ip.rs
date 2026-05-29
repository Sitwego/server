use std::net::IpAddr;

use axum::http::HeaderMap;

/// Extract the client IP from request headers, respecting trusted proxies.
///
/// Order of preference:
///   1. `X-Forwarded-For` — the *first* IP that is NOT in the trusted-proxy
///      list, walked from rightmost (closest to us) leftward. This stops a
///      hostile client from spoofing the chain by prepending `X-Forwarded-For:
///      1.2.3.4,` to their request.
///   2. `X-Real-IP` — used only if no trusted proxy chain is configured.
///   3. The empty string — caller must decide how to handle that. Almost
///      always a misconfigured deployment.
///
/// `trusted_proxies` is a list of CIDR-bare IPs (no prefix support — keep it
/// to your known ingress IPs). If the list is empty, behavior degrades to
/// "trust the first XFF entry", which is unsafe behind an untrusted edge.
pub fn extract_client_ip(
    headers: &HeaderMap,
    trusted_proxies: &[IpAddr],
) -> String {
    if let Some(raw) =
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok())
    {
        // Walk right-to-left, skipping any IP we trust as a proxy.
        // The first untrusted IP is the actual client.
        let ips: Vec<&str> = raw.split(',').map(str::trim).collect();
        if !trusted_proxies.is_empty() {
            for ip_str in ips.iter().rev() {
                if let Ok(ip) = ip_str.parse::<IpAddr>()
                    && !trusted_proxies.contains(&ip)
                {
                    return ip.to_string();
                }
            }
        } else if let Some(first) = ips.first() {
            // No trusted-proxy list configured. Use leftmost XFF entry but
            // log once — this configuration is unsafe behind a public edge.
            return first.to_string();
        }
    }

    if let Some(real) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        return real.trim().to_string();
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderName, HeaderValue};

    fn h(headers: &[(&'static str, &str)]) -> HeaderMap {
        let mut m = HeaderMap::new();
        for (k, v) in headers {
            m.insert(
                HeaderName::from_static(k),
                HeaderValue::from_str(v).unwrap(),
            );
        }
        m
    }

    #[test]
    fn picks_first_untrusted_walking_right_to_left() {
        // chain: client(1.2.3.4) -> proxy(10.0.0.1) -> us
        let headers = h(&[("x-forwarded-for", "1.2.3.4, 10.0.0.1")]);
        let trusted = vec!["10.0.0.1".parse().unwrap()];
        assert_eq!(extract_client_ip(&headers, &trusted), "1.2.3.4");
    }

    #[test]
    fn ignores_spoofed_left_entries_when_proxy_trusted() {
        // attacker prepended a spoofed IP — we walk from the right past our
        // own proxy and stop at the real client
        let headers = h(&[("x-forwarded-for", "9.9.9.9, 1.2.3.4, 10.0.0.1")]);
        let trusted = vec!["10.0.0.1".parse().unwrap()];
        assert_eq!(extract_client_ip(&headers, &trusted), "1.2.3.4");
    }

    #[test]
    fn falls_back_to_real_ip() {
        let headers = h(&[("x-real-ip", "8.8.8.8")]);
        assert_eq!(extract_client_ip(&headers, &[]), "8.8.8.8");
    }

    #[test]
    fn empty_when_nothing_present() {
        let headers = HeaderMap::new();
        assert_eq!(extract_client_ip(&headers, &[]), "");
    }
}
