pub fn is_cmd(text: &str, prefix: &str) -> bool {
    !prefix.is_empty() && text.starts_with(prefix)
}
