#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FrontMatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub slug: Option<String>,
    pub template: Option<String>,
    pub locale: Option<String>,
}

pub(crate) fn split_front_matter(source: &str) -> (FrontMatter, String) {
    let normalized = source.strip_prefix('\u{feff}').unwrap_or(source);
    if !normalized.starts_with("---\n") && !normalized.starts_with("---\r\n") {
        return (FrontMatter::default(), normalized.to_string());
    }

    let body_start = if normalized.starts_with("---\r\n") {
        5
    } else {
        4
    };
    let rest = &normalized[body_start..];
    let Some((front, body)) = find_front_matter_end(rest) else {
        return (FrontMatter::default(), normalized.to_string());
    };

    (parse_front_matter(front), body.to_string())
}

fn find_front_matter_end(rest: &str) -> Option<(&str, &str)> {
    for marker in ["\n---\n", "\r\n---\r\n", "\n---\r\n", "\r\n---\n"] {
        if let Some(index) = rest.find(marker) {
            let front = &rest[..index];
            let body = &rest[index + marker.len()..];
            return Some((front, body));
        }
    }
    None
}

fn parse_front_matter(source: &str) -> FrontMatter {
    let mut front = FrontMatter::default();
    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = clean_scalar(value.trim());
        match key.trim() {
            "title" => front.title = Some(value),
            "description" => front.description = Some(value),
            "slug" => front.slug = Some(value),
            "template" => front.template = Some(value),
            "locale" | "lang" | "language" => front.locale = Some(value),
            _ => {}
        }
    }
    front
}

fn clean_scalar(value: &str) -> String {
    value
        .trim_matches(|ch| ch == '"' || ch == '\'')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_front_matter_fields() {
        let (front, body) =
            split_front_matter("---\ntitle: Intro\ndescription: 'Start here'\n---\n# Intro");
        assert_eq!(front.title.as_deref(), Some("Intro"));
        assert_eq!(front.description.as_deref(), Some("Start here"));
        assert_eq!(body, "# Intro");
    }
}
