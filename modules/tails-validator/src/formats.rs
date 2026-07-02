pub fn is_valid_email(s: &str) -> bool {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return false;
    }
    let domain_parts: Vec<&str> = domain.split('.').collect();
    if domain_parts.iter().any(|p| p.is_empty()) {
        return false;
    }
    local
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '+' || c == '-')
        && domain
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
}

pub fn is_valid_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

pub fn is_valid_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    [8, 4, 4, 4, 12]
        .iter()
        .zip(parts.iter())
        .all(|(&len, &part)| part.len() == len && part.chars().all(|c| c.is_ascii_hexdigit()))
}

pub fn is_valid_datetime(s: &str) -> bool {
    if s.len() < 10 {
        return false;
    }
    let parts: Vec<&str> = s[..10].split('-').collect();
    parts.len() == 3 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

pub fn is_valid_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok())
}

pub fn is_valid_ipv6(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 3 || parts.len() > 8 {
        return false;
    }
    parts
        .iter()
        .all(|p| p.is_empty() || (p.len() <= 4 && p.chars().all(|c| c.is_ascii_hexdigit())))
}

pub fn is_valid_phone(s: &str) -> bool {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() >= 7 && digits.len() <= 15
}

pub fn is_valid_base64(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        && s.len().is_multiple_of(4)
}
