//! Turning what the user typed in the address bar into a URL.

/// Resolve address-bar input. Anything that looks like a URL or host is
/// opened directly, everything else is sent to the search engine.
/// `search_template` contains `{}` where the encoded query goes.
pub fn resolve_input(input: &str, search_template: &str) -> String {
    let input = input.trim();
    if input.is_empty() {
        return "zaphire://app/newtab".to_string();
    }
    if has_scheme(input) {
        return input.to_string();
    }
    if looks_like_host(input) {
        let local = input.starts_with("localhost") || input.starts_with("127.");
        return format!("{}://{input}", if local { "http" } else { "https" });
    }
    search_template.replace("{}", &encode_query(input))
}

fn has_scheme(s: &str) -> bool {
    const SCHEMES: [&str; 9] = [
        "http://",
        "https://",
        "file://",
        "chrome://",
        "chrome-extension://",
        "zaphire://",
        "about:",
        "data:",
        "view-source:",
    ];
    let lower = s.to_ascii_lowercase();
    SCHEMES.iter().any(|p| lower.starts_with(p))
}

fn looks_like_host(s: &str) -> bool {
    if s.contains(char::is_whitespace) {
        return false;
    }
    let host = s.split(['/', '?', '#']).next().unwrap_or("");
    let host_no_port = host.rsplit_once(':').map_or(host, |(h, p)| {
        if p.chars().all(|c| c.is_ascii_digit()) {
            h
        } else {
            host
        }
    });
    if host_no_port == "localhost" {
        return true;
    }
    let labels: Vec<&str> = host_no_port.split('.').collect();
    labels.len() >= 2
        && labels
            .iter()
            .all(|l| !l.is_empty() && l.chars().all(|c| c.is_alphanumeric() || c == '-'))
        && labels
            .last()
            .is_some_and(|tld| tld.chars().any(|c| c.is_alphabetic()) || labels.len() == 4)
}

fn encode_query(q: &str) -> String {
    let mut out = String::with_capacity(q.len() * 3);
    for b in q.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const S: &str = "https://duckduckgo.com/?q={}";

    #[test]
    fn urls_pass_through() {
        assert_eq!(
            resolve_input("https://example.com/a", S),
            "https://example.com/a"
        );
        assert_eq!(
            resolve_input("chrome://extensions", S),
            "chrome://extensions"
        );
    }

    #[test]
    fn hosts_get_https() {
        assert_eq!(resolve_input("example.com", S), "https://example.com");
        assert_eq!(
            resolve_input("sub.example.co.uk/path?x=1", S),
            "https://sub.example.co.uk/path?x=1"
        );
        assert_eq!(resolve_input("localhost:3000", S), "http://localhost:3000");
        assert_eq!(resolve_input("192.168.1.1", S), "https://192.168.1.1");
    }

    #[test]
    fn text_is_searched() {
        assert_eq!(
            resolve_input("rust cef browser", S),
            "https://duckduckgo.com/?q=rust+cef+browser"
        );
        assert_eq!(
            resolve_input("c++ & rust", S),
            "https://duckduckgo.com/?q=c%2B%2B+%26+rust"
        );
        assert_eq!(resolve_input("hello", S), "https://duckduckgo.com/?q=hello");
    }
}
