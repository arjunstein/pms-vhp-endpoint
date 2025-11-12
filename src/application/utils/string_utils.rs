pub fn capitalize_first_word(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => {
                    f.to_uppercase().collect::<String>() + &c.collect::<String>().to_lowercase()
                }
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn get_formatted_name(name: &Option<String>, pass: &Option<String>) -> String {
    match name {
        Some(n) if !n.trim().is_empty() => capitalize_first_word(n.trim()),
        _ => {
            let fallback = pass.clone().unwrap_or_default();
            capitalize_first_word(fallback.trim())
        }
    }
}

pub fn clean_password(password: &str) -> String {
    password
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .trim()
        .to_lowercase()
}
