pub fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

pub fn is_exact_match(a: &str, b: &str) -> bool {
    a == b
}

pub fn is_normalized_match(a: &str, b: &str) -> bool {
    normalize_name(a) == normalize_name(b)
}

pub fn is_alias_match(aliases: &[String], name: &str) -> bool {
    let normalized_name = normalize_name(name);
    aliases
        .iter()
        .any(|alias| normalize_name(alias) == normalized_name)
}
