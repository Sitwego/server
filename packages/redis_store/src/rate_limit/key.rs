pub use crate::rate_limit::policy::KeyScope;

/// A short numeric identifier for a route. The middleware maps `MatchedPath`
/// → `RouteId` at startup, so keys stay bounded even if attackers stuff
/// garbage into the URL (e.g. `/rides/<million distinct ids>`).
pub type RouteId = u32;

/// Normalize an Axum `matched_path` into a stable, low-cardinality string
/// suitable as part of a Redis key.
///
/// Axum already returns the route template (`"/api/rides/{ride_id}/fare"`),
/// so the work here is mostly defensive — strip placeholders down to a known
/// shape and trim leading slashes. Anything inside `{...}` collapses to `*`
/// to prevent surprises if a future route uses `{id:regex}` style.
pub fn normalize_route(matched_path: &str) -> String {
    let mut out = String::with_capacity(matched_path.len());
    let mut in_param = false;
    for ch in matched_path.chars() {
        match ch {
            '{' => {
                in_param = true;
                out.push('*');
            }
            '}' => in_param = false,
            c if in_param => {
                // discard characters inside {param}
                let _ = c;
            }
            '/' if out.is_empty() => {} // strip leading slash
            c => out.push(c),
        }
    }
    out
}

/// Build a fully-qualified Redis key for a rate-limit bucket.
///
/// Layout: `rl:{algo_tag}:{scope_tag}:{identifier}:{route}`
///
/// `algo_tag` separates buckets so switching algorithm on a route doesn't
/// collide with stale data from the old algorithm.
pub fn build_key(
    algo_tag: &str,
    scope_tag: &str,
    identifier: &str,
    route_id: &str,
) -> String {
    // Pre-size: tags are short, identifier dominates.
    let mut k = String::with_capacity(
        4 + algo_tag.len()
            + scope_tag.len()
            + identifier.len()
            + route_id.len(),
    );
    k.push_str("rl:");
    k.push_str(algo_tag);
    k.push(':');
    k.push_str(scope_tag);
    k.push(':');
    k.push_str(identifier);
    k.push(':');
    k.push_str(route_id);
    k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_leading_slash_and_placeholders() {
        assert_eq!(
            normalize_route("/api/rides/{ride_id}/fare"),
            "api/rides/*/fare"
        );
    }

    #[test]
    fn normalize_handles_multiple_placeholders() {
        assert_eq!(
            normalize_route("/api/rides/{ride_id}/fare/components/{key}"),
            "api/rides/*/fare/components/*"
        );
    }

    #[test]
    fn normalize_passes_through_no_placeholders() {
        assert_eq!(normalize_route("/login"), "login");
    }

    #[test]
    fn build_key_layout() {
        let k = build_key("tb", "user", "01HX...", "api/rides/*/fare");
        assert_eq!(k, "rl:tb:user:01HX...:api/rides/*/fare");
    }
}
