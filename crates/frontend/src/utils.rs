use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};

use crate::api::API_BASE;

/// Sanitize untrusted or Markdown-rendered HTML using a strict tag, attribute,
/// and URL allowlist to prevent XSS without pulling `ammonia`/`html5ever` into
/// WASM.
pub fn sanitize_html(raw: &str) -> String {
    if raw.is_empty() {
        return String::new();
    }

    let bytes = raw.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(raw.len());
    let mut open_stack: Vec<&'static str> = Vec::new();
    let mut drop_state: Option<(&'static str, usize)> = None;
    let mut i = 0usize;

    while i < len {
        if let Some((drop_tag, depth)) = drop_state {
            if is_rawtext_drop_tag(drop_tag) {
                if let Some((close_start, after_close)) = find_rawtext_closing_tag(raw, i, drop_tag)
                {
                    let _ = close_start;
                    i = after_close;
                    drop_state = None;
                } else {
                    break;
                }
                continue;
            }

            let Some(rel_lt) = raw[i..].find('<') else {
                break;
            };
            let lt_pos = i + rel_lt;
            if raw[lt_pos..].starts_with("<!--") {
                i = match raw[lt_pos + 4..].find("-->") {
                    Some(end_rel) => lt_pos + 4 + end_rel + 3,
                    None => len,
                };
                continue;
            }
            if let Some(parsed) = parse_html_tag(raw, lt_pos) {
                let tag_lower = parsed.name.to_ascii_lowercase();
                if tag_lower == drop_tag {
                    if parsed.is_closing {
                        if depth <= 1 {
                            drop_state = None;
                        } else {
                            drop_state = Some((drop_tag, depth - 1));
                        }
                    } else if !parsed.self_closing {
                        drop_state = Some((drop_tag, depth + 1));
                    }
                }
                i = parsed.next_index;
            } else {
                i = lt_pos + 1;
            }
            continue;
        }

        let Some(rel_lt) = raw[i..].find('<') else {
            append_sanitized_text_segment(&mut out, &raw[i..]);
            break;
        };
        let lt_pos = i + rel_lt;
        if lt_pos > i {
            append_sanitized_text_segment(&mut out, &raw[i..lt_pos]);
        }

        if raw[lt_pos..].starts_with("<!--") {
            i = match raw[lt_pos + 4..].find("-->") {
                Some(end_rel) => lt_pos + 4 + end_rel + 3,
                None => len,
            };
            continue;
        }

        if raw[lt_pos..].starts_with("<!") || raw[lt_pos..].starts_with("<?") {
            i = match raw[lt_pos + 2..].find('>') {
                Some(end_rel) => lt_pos + 2 + end_rel + 1,
                None => len,
            };
            continue;
        }

        let Some(parsed) = parse_html_tag(raw, lt_pos) else {
            out.push_str("&lt;");
            i = lt_pos + 1;
            continue;
        };

        let tag_lower = parsed.name.to_ascii_lowercase();
        if let Some(drop_tag) = canonical_drop_content_tag(&tag_lower) {
            if !parsed.is_closing && (!parsed.self_closing || is_rawtext_drop_tag(drop_tag)) {
                drop_state = Some((drop_tag, 1));
            }
            i = parsed.next_index;
            continue;
        }

        let Some(allowed_tag) = canonical_allowed_tag(&tag_lower) else {
            i = parsed.next_index;
            continue;
        };

        if parsed.is_closing {
            if !is_void_tag(allowed_tag) {
                if let Some(pos) = open_stack.iter().rposition(|&open| open == allowed_tag) {
                    while open_stack.len() > pos {
                        if let Some(popped) = open_stack.pop() {
                            out.push_str("</");
                            out.push_str(popped);
                            out.push('>');
                        }
                    }
                }
            }
            i = parsed.next_index;
            continue;
        }

        if emit_sanitized_opening_tag(&mut out, allowed_tag, &parsed.attrs)
            && !is_void_tag(allowed_tag)
        {
            if parsed.self_closing {
                out.push_str("</");
                out.push_str(allowed_tag);
                out.push('>');
            } else {
                open_stack.push(allowed_tag);
            }
        }

        i = parsed.next_index;
    }

    while let Some(unclosed) = open_stack.pop() {
        out.push_str("</");
        out.push_str(unclosed);
        out.push('>');
    }

    out
}

struct ParsedHtmlTag<'a> {
    name: &'a str,
    is_closing: bool,
    self_closing: bool,
    attrs: Vec<(&'a str, Option<&'a str>)>,
    next_index: usize,
}

fn parse_html_tag(raw: &str, lt_pos: usize) -> Option<ParsedHtmlTag<'_>> {
    let bytes = raw.as_bytes();
    let len = bytes.len();
    let mut idx = lt_pos + 1;
    if idx >= len {
        return None;
    }

    let is_closing = if bytes[idx] == b'/' {
        idx += 1;
        true
    } else {
        false
    };

    if idx >= len || !bytes[idx].is_ascii_alphabetic() {
        return None;
    }

    let name_start = idx;
    while idx < len && (bytes[idx].is_ascii_alphanumeric() || bytes[idx] == b'-') {
        idx += 1;
    }
    let name = &raw[name_start..idx];

    let mut attrs = Vec::new();
    let mut self_closing = false;

    loop {
        while idx < len && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= len {
            return None;
        }
        if bytes[idx] == b'>' {
            return Some(ParsedHtmlTag {
                name,
                is_closing,
                self_closing,
                attrs,
                next_index: idx + 1,
            });
        }
        if bytes[idx] == b'/' {
            self_closing = true;
            idx += 1;
            continue;
        }

        let attr_start = idx;
        while idx < len
            && !bytes[idx].is_ascii_whitespace()
            && bytes[idx] != b'='
            && bytes[idx] != b'>'
            && bytes[idx] != b'/'
        {
            idx += 1;
        }
        let attr_name = &raw[attr_start..idx];
        if attr_name.is_empty() {
            idx += 1;
            continue;
        }

        while idx < len && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx < len && bytes[idx] == b'=' {
            idx += 1;
            while idx < len && bytes[idx].is_ascii_whitespace() {
                idx += 1;
            }
            if idx >= len {
                return None;
            }
            if bytes[idx] == b'"' || bytes[idx] == b'\'' {
                let quote = bytes[idx];
                idx += 1;
                let val_start = idx;
                while idx < len && bytes[idx] != quote {
                    idx += 1;
                }
                if idx >= len {
                    return None;
                }
                let val = &raw[val_start..idx];
                idx += 1;
                if !is_closing {
                    attrs.push((attr_name, Some(val)));
                }
            } else {
                let val_start = idx;
                while idx < len && !bytes[idx].is_ascii_whitespace() && bytes[idx] != b'>' {
                    idx += 1;
                }
                let val = &raw[val_start..idx];
                if !is_closing {
                    attrs.push((attr_name, Some(val)));
                }
            }
        } else if !is_closing {
            attrs.push((attr_name, None));
        }
    }
}

fn find_rawtext_closing_tag(raw: &str, start: usize, tag: &str) -> Option<(usize, usize)> {
    let bytes = raw.as_bytes();
    let len = bytes.len();
    let tag_bytes = tag.as_bytes();
    let mut cursor = start;

    while let Some(rel) = raw[cursor..].find("</") {
        let pos = cursor + rel;
        let name_start = pos + 2;
        let name_end = name_start + tag_bytes.len();
        if name_end < len && bytes[name_start..name_end].eq_ignore_ascii_case(tag_bytes) {
            let next_byte = bytes[name_end];
            if next_byte == b'>' || next_byte == b'/' || next_byte.is_ascii_whitespace() {
                if let Some(gt_rel) = raw[name_end..].find('>') {
                    return Some((pos, name_end + gt_rel + 1));
                }
                return None;
            }
        }
        cursor = pos + 2;
    }
    None
}

fn is_rawtext_drop_tag(tag: &str) -> bool {
    matches!(
        tag,
        "script"
            | "style"
            | "textarea"
            | "title"
            | "xmp"
            | "plaintext"
            | "iframe"
            | "noembed"
            | "noframes"
    )
}

fn canonical_drop_content_tag(tag_lower: &str) -> Option<&'static str> {
    match tag_lower {
        "script" => Some("script"),
        "style" => Some("style"),
        "iframe" => Some("iframe"),
        "object" => Some("object"),
        "embed" => Some("embed"),
        "applet" => Some("applet"),
        "svg" => Some("svg"),
        "math" => Some("math"),
        "form" => Some("form"),
        "noscript" => Some("noscript"),
        "template" => Some("template"),
        "textarea" => Some("textarea"),
        "title" => Some("title"),
        "xmp" => Some("xmp"),
        "plaintext" => Some("plaintext"),
        "head" => Some("head"),
        "frameset" => Some("frameset"),
        "frame" => Some("frame"),
        "noembed" => Some("noembed"),
        "noframes" => Some("noframes"),
        "base" => Some("base"),
        "link" => Some("link"),
        "meta" => Some("meta"),
        _ => None,
    }
}

fn canonical_allowed_tag(tag_lower: &str) -> Option<&'static str> {
    match tag_lower {
        "p" => Some("p"),
        "br" => Some("br"),
        "hr" => Some("hr"),
        "h1" => Some("h1"),
        "h2" => Some("h2"),
        "h3" => Some("h3"),
        "h4" => Some("h4"),
        "h5" => Some("h5"),
        "h6" => Some("h6"),
        "strong" => Some("strong"),
        "b" => Some("b"),
        "em" => Some("em"),
        "i" => Some("i"),
        "del" => Some("del"),
        "s" => Some("s"),
        "ins" => Some("ins"),
        "mark" => Some("mark"),
        "code" => Some("code"),
        "pre" => Some("pre"),
        "blockquote" => Some("blockquote"),
        "ul" => Some("ul"),
        "ol" => Some("ol"),
        "li" => Some("li"),
        "dl" => Some("dl"),
        "dt" => Some("dt"),
        "dd" => Some("dd"),
        "table" => Some("table"),
        "thead" => Some("thead"),
        "tbody" => Some("tbody"),
        "tfoot" => Some("tfoot"),
        "tr" => Some("tr"),
        "th" => Some("th"),
        "td" => Some("td"),
        "a" => Some("a"),
        "img" => Some("img"),
        "div" => Some("div"),
        "span" => Some("span"),
        "section" => Some("section"),
        "sup" => Some("sup"),
        "sub" => Some("sub"),
        "kbd" => Some("kbd"),
        "details" => Some("details"),
        "summary" => Some("summary"),
        "input" => Some("input"),
        _ => None,
    }
}

fn is_void_tag(tag: &str) -> bool {
    matches!(tag, "br" | "hr" | "img" | "input")
}

fn emit_sanitized_opening_tag(
    out: &mut String,
    tag: &'static str,
    attrs: &[(&str, Option<&str>)],
) -> bool {
    if tag == "input" {
        let is_checkbox = attrs.iter().any(|(name, value)| {
            name.eq_ignore_ascii_case("type")
                && value
                    .map(|v| {
                        decode_html_entities(v)
                            .trim()
                            .eq_ignore_ascii_case("checkbox")
                    })
                    .unwrap_or(false)
        });
        if !is_checkbox {
            return false;
        }

        let mut is_checked = false;
        let mut is_disabled = false;
        let mut safe_id: Option<String> = None;
        let mut safe_class: Option<String> = None;
        for (name, value) in attrs {
            if name.eq_ignore_ascii_case("checked") {
                is_checked = true;
            } else if name.eq_ignore_ascii_case("disabled") {
                is_disabled = true;
            } else if name.eq_ignore_ascii_case("id") {
                if let Some(raw_val) = value {
                    let decoded = decode_html_entities(raw_val);
                    let trimmed = decoded.trim();
                    if !trimmed.is_empty() {
                        safe_id = Some(trimmed.to_string());
                    }
                }
            } else if name.eq_ignore_ascii_case("class") {
                if let Some(raw_val) = value {
                    let decoded = decode_html_entities(raw_val);
                    let trimmed = decoded.trim();
                    if !trimmed.is_empty() {
                        safe_class = Some(trimmed.to_string());
                    }
                }
            }
        }

        out.push_str("<input type=\"checkbox\"");
        if is_checked {
            out.push_str(" checked=\"\"");
        }
        if is_disabled {
            out.push_str(" disabled=\"\"");
        }
        if let Some(id_val) = safe_id {
            push_escaped_attr(out, "id", &id_val);
        }
        if let Some(class_val) = safe_class {
            push_escaped_attr(out, "class", &class_val);
        }
        out.push('>');
        return true;
    }

    let mut sanitized_attrs: Vec<(&'static str, String)> = Vec::new();
    let mut seen_names: Vec<&'static str> = Vec::new();
    let mut has_valid_img_src = false;

    for (raw_name, raw_value) in attrs {
        let lower = raw_name.to_ascii_lowercase();
        if lower.starts_with("on") {
            continue;
        }

        let decoded_val = raw_value.map(decode_html_entities).unwrap_or_default();
        let trimmed_val = decoded_val.trim();

        let accepted: Option<(&'static str, String)> = match lower.as_str() {
            "id" if !trimmed_val.is_empty() => Some(("id", trimmed_val.to_string())),
            "class" if !trimmed_val.is_empty() => Some(("class", trimmed_val.to_string())),
            "title" => Some(("title", decoded_val)),
            "aria-hidden" if matches!(trimmed_val, "true" | "false") => {
                Some(("aria-hidden", trimmed_val.to_string()))
            },
            "href" if tag == "a" => sanitize_url(&decoded_val, false).map(|url| ("href", url)),
            "src" if tag == "img" => sanitize_url(&decoded_val, true).map(|url| {
                has_valid_img_src = true;
                ("src", url)
            }),
            "alt" if tag == "img" => Some(("alt", decoded_val)),
            "loading" if tag == "img" => {
                let mode = trimmed_val.to_ascii_lowercase();
                if matches!(mode.as_str(), "lazy" | "eager") {
                    Some(("loading", mode))
                } else {
                    None
                }
            },
            "width" if tag == "img" && is_safe_dimension(trimmed_val) => {
                Some(("width", trimmed_val.to_string()))
            },
            "height" if tag == "img" && is_safe_dimension(trimmed_val) => {
                Some(("height", trimmed_val.to_string()))
            },
            "colspan" if matches!(tag, "th" | "td") => trimmed_val
                .parse::<u16>()
                .ok()
                .filter(|&n| (1..=1000).contains(&n))
                .map(|n| ("colspan", n.to_string())),
            "rowspan" if matches!(tag, "th" | "td") => trimmed_val
                .parse::<u16>()
                .ok()
                .filter(|&n| n <= 1000)
                .map(|n| ("rowspan", n.to_string())),
            "align" if matches!(tag, "th" | "td" | "tr" | "table") => {
                let align = trimmed_val.to_ascii_lowercase();
                if matches!(align.as_str(), "left" | "center" | "right" | "justify") {
                    Some(("align", align))
                } else {
                    None
                }
            },
            "start" if tag == "ol" => trimmed_val
                .parse::<i32>()
                .ok()
                .map(|n| ("start", n.to_string())),
            "open" if tag == "details" => Some(("open", String::new())),
            _ => None,
        };

        if let Some((canon_name, clean_val)) = accepted {
            if !seen_names.contains(&canon_name) {
                seen_names.push(canon_name);
                sanitized_attrs.push((canon_name, clean_val));
            }
        }
    }

    if tag == "img" && !has_valid_img_src {
        return false;
    }

    out.push('<');
    out.push_str(tag);
    for (attr_name, attr_val) in &sanitized_attrs {
        push_escaped_attr(out, attr_name, attr_val);
    }
    if tag == "a" {
        push_escaped_attr(out, "rel", "noopener noreferrer");
    }
    out.push('>');
    true
}

fn is_safe_dimension(value: &str) -> bool {
    if value.is_empty() || value.len() > 16 {
        return false;
    }
    let digits = value.strip_suffix('%').unwrap_or(value);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

fn sanitize_url(decoded_value: &str, allow_data_image: bool) -> Option<String> {
    let cleaned: String = decoded_value
        .chars()
        .filter(|ch| !ch.is_control())
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return None;
    }

    let squashed_lower: String = trimmed
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect();

    if squashed_lower.starts_with("//") || squashed_lower.starts_with("\\\\") {
        return None;
    }

    if trimmed.starts_with('#') {
        return Some(trimmed.to_string());
    }

    if let Some(rest) = trimmed.strip_prefix('/') {
        if rest.starts_with('/') || rest.starts_with('\\') {
            return None;
        }
        return Some(trimmed.to_string());
    }

    if trimmed.starts_with("./") || trimmed.starts_with("../") {
        return Some(trimmed.to_string());
    }

    if squashed_lower.starts_with("http://")
        || squashed_lower.starts_with("https://")
        || squashed_lower.starts_with("mailto:")
    {
        return Some(trimmed.to_string());
    }

    if allow_data_image && is_safe_raster_data_image_url(&squashed_lower) {
        return Some(trimmed.to_string());
    }

    None
}

fn is_safe_raster_data_image_url(squashed_lower: &str) -> bool {
    const SAFE_PREFIXES: &[&str] = &[
        "data:image/png;base64,",
        "data:image/jpeg;base64,",
        "data:image/jpg;base64,",
        "data:image/gif;base64,",
        "data:image/webp;base64,",
        "data:image/avif;base64,",
        "data:image/bmp;base64,",
        "data:image/x-icon;base64,",
        "data:image/vnd.microsoft.icon;base64,",
    ];

    let Some(payload) = SAFE_PREFIXES
        .iter()
        .find_map(|prefix| squashed_lower.strip_prefix(prefix))
    else {
        return false;
    };

    !payload.is_empty()
        && payload
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'='))
}

fn push_escaped_attr(out: &mut String, name: &str, value: &str) {
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

fn append_sanitized_text_segment(out: &mut String, segment: &str) {
    let bytes = segment.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'>' => {
                out.push_str("&gt;");
                i += 1;
            },
            b'&' => {
                if let Some(entity_len) = valid_html_entity_len(&segment[i..]) {
                    out.push_str(&segment[i..i + entity_len]);
                    i += entity_len;
                } else {
                    out.push_str("&amp;");
                    i += 1;
                }
            },
            _ => {
                let ch = segment[i..].chars().next().unwrap_or('\u{FFFD}');
                out.push(ch);
                i += ch.len_utf8();
            },
        }
    }
}

fn valid_html_entity_len(slice: &str) -> Option<usize> {
    let semi = slice.get(1..)?.find(';')? + 1;
    if semi > 34 {
        return None;
    }
    let body = &slice[1..semi];
    if body.is_empty() {
        return None;
    }
    if let Some(num) = body.strip_prefix('#') {
        let valid = if let Some(hex) = num.strip_prefix('x').or_else(|| num.strip_prefix('X')) {
            !hex.is_empty() && hex.bytes().all(|b| b.is_ascii_hexdigit())
        } else {
            !num.is_empty() && num.bytes().all(|b| b.is_ascii_digit())
        };
        if valid {
            return Some(semi + 1);
        }
        return None;
    }
    if body.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Some(semi + 1);
    }
    None
}

fn decode_html_entities(input: &str) -> String {
    if !input.contains('&') {
        return input.to_string();
    }
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;
    while i < input.len() {
        let Some(rel_amp) = input[i..].find('&') else {
            out.push_str(&input[i..]);
            break;
        };
        let amp_pos = i + rel_amp;
        out.push_str(&input[i..amp_pos]);
        let rest = &input[amp_pos + 1..];
        let max_scan = rest.len().min(32);
        let entity_end = rest[..max_scan].find(|ch: char| {
            ch == ';' || ch.is_whitespace() || ch == '&' || ch == '<' || ch == '"'
        });
        let (token, consumed) = match entity_end {
            Some(idx) if rest.as_bytes()[idx] == b';' => (&rest[..idx], 1 + idx + 1),
            Some(idx) => (&rest[..idx], 1 + idx),
            None => (&rest[..max_scan], 1 + max_scan),
        };
        if let Some(decoded) = decode_single_entity(token) {
            out.push(decoded);
            i = amp_pos + consumed;
        } else {
            out.push('&');
            i = amp_pos + 1;
        }
    }
    out
}

fn decode_single_entity(token: &str) -> Option<char> {
    if let Some(num) = token.strip_prefix('#') {
        let code = if let Some(hex) = num.strip_prefix('x').or_else(|| num.strip_prefix('X')) {
            u32::from_str_radix(hex, 16).ok()?
        } else {
            num.parse::<u32>().ok()?
        };
        return char::from_u32(code);
    }
    match token.to_ascii_lowercase().as_str() {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "colon" => Some(':'),
        "sol" => Some('/'),
        "bsol" => Some('\\'),
        "tab" => Some('\t'),
        "newline" => Some('\n'),
        "nbsp" => Some('\u{00A0}'),
        _ => None,
    }
}

/// Convert image path to API endpoint if it's a relative path
pub fn image_url(path: &str) -> String {
    let normalized = path.trim();

    if normalized.starts_with("http://")
        || normalized.starts_with("https://")
        || normalized.starts_with("data:")
    {
        normalized.to_string()
    } else if normalized.starts_with("images/") {
        let filename = normalized.strip_prefix("images/").unwrap_or(normalized);
        format!("{}/images/{}", API_BASE, filename)
    } else if normalized.starts_with("/api/images/") {
        format!("{}{}", API_BASE.trim_end_matches("/api"), normalized)
    } else {
        normalized.to_string()
    }
}

/// Convert Markdown content into HTML with common extensions enabled.
/// Also transforms relative image paths to API endpoints.
pub fn markdown_to_html(content: &str) -> String {
    if content.trim().is_empty() {
        return String::new();
    }

    // Protect display-math blocks before markdown parsing so `=` lines inside
    // formulas are not interpreted as Setext headings.
    let normalized_content = protect_display_math_blocks(content);
    let normalized_content = normalize_standalone_break_markers(&normalized_content);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(&normalized_content, options);

    // Transform image paths
    let transformed_parser = parser.map(|event| match event {
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            // Check if image path is relative (starts with "images/")
            let new_url = CowStr::from(image_url(&dest_url));
            Event::Start(Tag::Image {
                link_type,
                dest_url: new_url,
                title,
                id,
            })
        },
        _ => event,
    });

    let mut html_output = String::new();
    html::push_html(&mut html_output, transformed_parser);
    sanitize_html(&html_output)
}

fn normalize_standalone_break_markers(content: &str) -> String {
    let mut result = String::new();
    let mut in_fenced_code = false;
    let mut active_fence = "";

    for line in content.lines() {
        let trimmed = line.trim();

        let fence_marker = if trimmed.starts_with("```") {
            Some("```")
        } else if trimmed.starts_with("~~~") {
            Some("~~~")
        } else {
            None
        };

        if let Some(marker) = fence_marker {
            if in_fenced_code && active_fence == marker {
                in_fenced_code = false;
                active_fence = "";
            } else if !in_fenced_code {
                in_fenced_code = true;
                active_fence = marker;
            }

            result.push_str(line);
            result.push('\n');
            continue;
        }

        if !in_fenced_code && matches!(trimmed, "<br>" | "<br/>" | "<br />") {
            result.push('\n');
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

#[derive(Clone, Debug)]
struct MarkdownExportBase {
    origin: String,
    page_dir_path: String,
}

/// Convert relative markdown resource paths into absolute URLs for external
/// export/copy.
///
/// This is used by "View Raw Markdown" and "Copy/Export Markdown" to keep links
/// and image resources valid after markdown is copied out of the site context.
pub fn markdown_for_external_export(markdown: &str) -> String {
    let Some(window) = web_sys::window() else {
        return markdown.to_string();
    };
    let location = window.location();
    let page_href = match location.href() {
        Ok(value) => value,
        Err(_) => return markdown.to_string(),
    };
    let origin = location.origin().ok();
    markdown_for_external_export_with_base(markdown, &page_href, origin.as_deref())
}

fn markdown_for_external_export_with_base(
    markdown: &str,
    page_href: &str,
    origin_hint: Option<&str>,
) -> String {
    let Some(base) = build_markdown_export_base(page_href, origin_hint) else {
        return markdown.to_string();
    };

    let with_inline = rewrite_inline_markdown_links(markdown, &base);
    let with_references = rewrite_reference_markdown_links(&with_inline, &base);
    rewrite_html_src_href_attributes(&with_references, &base)
}

fn build_markdown_export_base(
    page_href: &str,
    origin_hint: Option<&str>,
) -> Option<MarkdownExportBase> {
    let without_fragment = page_href
        .trim()
        .split_once('#')
        .map(|(head, _)| head)
        .unwrap_or(page_href.trim());
    let without_query = without_fragment
        .split_once('?')
        .map(|(head, _)| head)
        .unwrap_or(without_fragment);
    let (scheme, after_scheme) = without_query.split_once("://")?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return None;
    }

    let (authority, raw_path) = match after_scheme.find('/') {
        Some(slash_idx) => (&after_scheme[..slash_idx], &after_scheme[slash_idx..]),
        None => (after_scheme, "/"),
    };
    if authority.is_empty() {
        return None;
    }

    let parsed_origin = format!("{}://{}", scheme.to_ascii_lowercase(), authority);
    let origin = origin_hint
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_end_matches('/').to_string())
        .unwrap_or(parsed_origin);
    let page_dir_path = match raw_path.rfind('/') {
        Some(idx) => raw_path[..=idx].to_string(),
        None => "/".to_string(),
    };

    Some(MarkdownExportBase {
        origin,
        page_dir_path,
    })
}

fn rewrite_inline_markdown_links(input: &str, base: &MarkdownExportBase) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len() + 64);
    let mut cursor = 0usize;

    while let Some(rel_idx) = input[cursor..].find("](") {
        let marker = cursor + rel_idx;
        output.push_str(&input[cursor..marker + 2]);

        let mut i = marker + 2;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        output.push_str(&input[marker + 2..i]);

        if i >= bytes.len() {
            cursor = i;
            break;
        }

        let (target, after_target, wrapped_in_angles) = if bytes[i] == b'<' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'>' {
                j += 1;
            }
            if j >= bytes.len() {
                output.push_str(&input[i..]);
                cursor = bytes.len();
                break;
            }
            (&input[i + 1..j], j + 1, true)
        } else {
            let mut j = i;
            let mut nested_parens = 0i32;
            while j < bytes.len() {
                let b = bytes[j];
                if b == b'\\' {
                    j = (j + 2).min(bytes.len());
                    continue;
                }
                if b == b'(' {
                    nested_parens += 1;
                    j += 1;
                    continue;
                }
                if b == b')' {
                    if nested_parens == 0 {
                        break;
                    }
                    nested_parens -= 1;
                    j += 1;
                    continue;
                }
                if b.is_ascii_whitespace() && nested_parens == 0 {
                    break;
                }
                j += 1;
            }
            (&input[i..j], j, false)
        };

        let resolved = resolve_markdown_target(target, base);
        if wrapped_in_angles {
            output.push('<');
            output.push_str(&resolved);
            output.push('>');
        } else {
            output.push_str(&resolved);
        }

        let suffix_start = after_target;
        let mut j = after_target;
        let mut in_single_quote = false;
        let mut in_double_quote = false;

        while j < bytes.len() {
            let b = bytes[j];
            if b == b'\\' {
                j = (j + 2).min(bytes.len());
                continue;
            }
            if b == b'"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                j += 1;
                continue;
            }
            if b == b'\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                j += 1;
                continue;
            }
            if b == b')' && !in_single_quote && !in_double_quote {
                break;
            }
            j += 1;
        }

        output.push_str(&input[suffix_start..j]);
        if j < bytes.len() && bytes[j] == b')' {
            output.push(')');
            j += 1;
        }
        cursor = j;
    }

    output.push_str(&input[cursor..]);
    output
}

fn rewrite_reference_markdown_links(input: &str, base: &MarkdownExportBase) -> String {
    let mut output = String::with_capacity(input.len() + 32);

    for segment in input.split_inclusive('\n') {
        let (line, suffix) = if let Some(stripped) = segment.strip_suffix('\n') {
            (stripped, "\n")
        } else {
            (segment, "")
        };
        output.push_str(&rewrite_reference_link_line(line, base));
        output.push_str(suffix);
    }

    output
}

fn rewrite_reference_link_line(line: &str, base: &MarkdownExportBase) -> String {
    let trimmed_start = line.trim_start();
    if !trimmed_start.starts_with('[') {
        return line.to_string();
    }

    let Some(def_idx_rel) = trimmed_start.find("]:") else {
        return line.to_string();
    };

    let leading_ws_len = line.len() - trimmed_start.len();
    let prefix_end = leading_ws_len + def_idx_rel + 2;
    let bytes = line.as_bytes();

    let mut i = prefix_end;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return line.to_string();
    }

    let (target_start, target_end, wrapped_in_angles) = if bytes[i] == b'<' {
        let mut j = i + 1;
        while j < bytes.len() && bytes[j] != b'>' {
            j += 1;
        }
        if j >= bytes.len() {
            return line.to_string();
        }
        (i + 1, j, true)
    } else {
        let mut j = i;
        while j < bytes.len() && !bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        (i, j, false)
    };

    let raw_target = &line[target_start..target_end];
    let resolved = resolve_markdown_target(raw_target, base);
    let mut rewritten = String::with_capacity(line.len() + 32);
    rewritten.push_str(&line[..target_start]);
    if wrapped_in_angles {
        rewritten.push_str(&resolved);
        rewritten.push('>');
    } else {
        rewritten.push_str(&resolved);
    }
    rewritten.push_str(&line[target_end + usize::from(wrapped_in_angles)..]);
    rewritten
}

fn rewrite_html_src_href_attributes(input: &str, base: &MarkdownExportBase) -> String {
    let step1 = rewrite_html_attribute_urls(input, "href=\"", "\"", base);
    let step2 = rewrite_html_attribute_urls(&step1, "href='", "'", base);
    let step3 = rewrite_html_attribute_urls(&step2, "src=\"", "\"", base);
    rewrite_html_attribute_urls(&step3, "src='", "'", base)
}

fn rewrite_html_attribute_urls(
    input: &str,
    pattern: &str,
    quote: &str,
    base: &MarkdownExportBase,
) -> String {
    let mut output = String::with_capacity(input.len() + 32);
    let mut cursor = 0usize;

    while let Some(rel_idx) = input[cursor..].find(pattern) {
        let start = cursor + rel_idx;
        let value_start = start + pattern.len();
        output.push_str(&input[cursor..value_start]);

        let Some(end_rel) = input[value_start..].find(quote) else {
            output.push_str(&input[value_start..]);
            cursor = input.len();
            break;
        };

        let value_end = value_start + end_rel;
        let raw_target = &input[value_start..value_end];
        output.push_str(&resolve_markdown_target(raw_target, base));
        cursor = value_end;
    }

    output.push_str(&input[cursor..]);
    output
}

fn resolve_markdown_target(raw_target: &str, base: &MarkdownExportBase) -> String {
    let target = raw_target.trim();
    if target.is_empty() || is_non_http_target(target) {
        return raw_target.to_string();
    }

    if target.starts_with("/api/images/") {
        return format!("{}{}", base.origin, target);
    }
    if let Some(filename) = target.strip_prefix("images/") {
        return format!("{}/api/images/{}", base.origin, filename);
    }
    if target.starts_with('/') {
        return format!("{}{}", base.origin, target);
    }

    let suffix_idx = target.find(['?', '#']).unwrap_or(target.len());
    let path_part = &target[..suffix_idx];
    let suffix_part = &target[suffix_idx..];

    let base_dir = if path_part.starts_with("./")
        || path_part.starts_with("../")
        || path_part == "."
        || path_part == ".."
    {
        base.page_dir_path.as_str()
    } else {
        "/"
    };

    let mut stack: Vec<&str> = base_dir
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    for segment in path_part.split('/') {
        match segment {
            "" | "." => {},
            ".." => {
                stack.pop();
            },
            other => stack.push(other),
        }
    }

    let trailing_slash = path_part.ends_with('/')
        || path_part == "."
        || path_part == ".."
        || path_part.ends_with("/.")
        || path_part.ends_with("/..");

    let normalized_path = if stack.is_empty() {
        "/".to_string()
    } else if trailing_slash {
        format!("/{}/", stack.join("/"))
    } else {
        format!("/{}", stack.join("/"))
    };

    format!("{}{}{}", base.origin, normalized_path, suffix_part)
}

fn is_non_http_target(target: &str) -> bool {
    let lower = target.to_ascii_lowercase();
    lower.starts_with('#')
        || lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with("data:")
        || lower.starts_with("javascript:")
        || lower.starts_with("vbscript:")
        || lower.starts_with("ftp://")
        || lower.starts_with("//")
}

fn protect_display_math_blocks(content: &str) -> String {
    let mut result = String::new();
    let mut in_fenced_code = false;
    let mut active_fence = "";

    let mut in_math_block = false;
    let mut math_close_delimiter = "";
    let mut math_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if !in_math_block {
            let fence_marker = if trimmed.starts_with("```") {
                Some("```")
            } else if trimmed.starts_with("~~~") {
                Some("~~~")
            } else {
                None
            };

            if let Some(marker) = fence_marker {
                if in_fenced_code && active_fence == marker {
                    in_fenced_code = false;
                    active_fence = "";
                } else if !in_fenced_code {
                    in_fenced_code = true;
                    active_fence = marker;
                }

                result.push_str(line);
                result.push('\n');
                continue;
            }

            if !in_fenced_code {
                if line.starts_with("$$") {
                    in_math_block = true;
                    math_close_delimiter = "$$";
                    math_lines.push(line.to_string());

                    // Open + close on the same line, e.g. `$$E=mc^2$$`.
                    if line.matches("$$").count() >= 2 {
                        append_math_block(&mut result, &math_lines);
                        math_lines.clear();
                        in_math_block = false;
                        math_close_delimiter = "";
                    }
                    continue;
                }

                if line.starts_with("\\[") {
                    in_math_block = true;
                    math_close_delimiter = "\\]";
                    math_lines.push(line.to_string());

                    if let (Some(start), Some(end)) = (line.find("\\["), line.rfind("\\]")) {
                        if end > start {
                            append_math_block(&mut result, &math_lines);
                            math_lines.clear();
                            in_math_block = false;
                            math_close_delimiter = "";
                        }
                    }
                    continue;
                }
            }

            result.push_str(line);
            result.push('\n');
            continue;
        }

        math_lines.push(line.to_string());
        let should_close = match math_close_delimiter {
            "$$" => line.contains("$$"),
            "\\]" => line.contains("\\]"),
            _ => false,
        };

        if should_close {
            append_math_block(&mut result, &math_lines);
            math_lines.clear();
            in_math_block = false;
            math_close_delimiter = "";
        }
    }

    if in_math_block {
        for line in math_lines {
            result.push_str(&line);
            result.push('\n');
        }
    }

    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

fn append_math_block(result: &mut String, math_lines: &[String]) {
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }

    let math_text = math_lines.join("\n");
    result.push_str("<div class=\"sf-math-block\">\n");
    result.push_str(&escape_html_text(&math_text));
    result.push_str("\n</div>\n");
}

fn escape_html_text(content: &str) -> String {
    let mut escaped = String::with_capacity(content.len());
    for ch in content.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{markdown_for_external_export_with_base, markdown_to_html, sanitize_html};

    #[test]
    fn display_math_block_with_equals_is_not_promoted_to_heading() {
        let markdown = r#"矩阵乘法：

$$
\begin{bmatrix}
a & b \\
c & d
\end{bmatrix}
\begin{bmatrix}
x \\
y
\end{bmatrix}
=
\begin{bmatrix}
ax + by \\
cx + dy
\end{bmatrix}
$$

欧拉公式（数学中最美的公式之一）。"#;

        let html = markdown_to_html(markdown);

        assert!(html.contains("<div class=\"sf-math-block\">"));
        assert!(html.contains("欧拉公式（数学中最美的公式之一）。"));
        assert!(!html.contains("<h1>$$"));
    }

    #[test]
    fn export_markdown_absolutizes_links_images_and_refs() {
        let markdown = r#"
![img](images/a.png)
[post](/posts/abc)
[doc](docs/spec)
[rel](../x/y)
[ref-link]: article/foo "title"
<a href="posts/hello">x</a>
"#;

        let rewritten = markdown_for_external_export_with_base(
            markdown,
            "https://ackingliu.top/posts/current?x=1",
            Some("https://ackingliu.top"),
        );

        assert!(rewritten.contains("![img](https://ackingliu.top/api/images/a.png)"));
        assert!(rewritten.contains("[post](https://ackingliu.top/posts/abc)"));
        assert!(rewritten.contains("[doc](https://ackingliu.top/docs/spec)"));
        assert!(rewritten.contains("[rel](https://ackingliu.top/x/y)"));
        assert!(rewritten.contains("[ref-link]: https://ackingliu.top/article/foo \"title\""));
        assert!(rewritten.contains("<a href=\"https://ackingliu.top/posts/hello\">x</a>"));
    }

    #[test]
    fn standalone_break_markers_become_paragraph_boundaries() {
        let markdown = "第一段\n<br/>\n第二段";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<p>第一段</p>"));
        assert!(html.contains("<p>第二段</p>"));
        assert!(!html.contains("<br"));
    }

    #[test]
    fn standalone_break_marker_inside_code_fence_is_preserved_as_code_text() {
        let markdown = "```text\nline1\n<br/>\nline2\n```";
        let html = markdown_to_html(markdown);
        assert!(html.contains("&lt;br/&gt;"));
    }

    #[test]
    fn markdown_html_sanitizer_preserves_safe_markdown_output_attributes() {
        let markdown = r#"
[site](https://example.com)

<table><tr><td colspan="2">wide</td></tr></table>
<img src="/api/images/a.png" width="640" height="480" loading="lazy">
<script>alert(1)</script>
"#;

        let html = markdown_to_html(markdown);

        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("rel=\"noopener noreferrer\""));
        assert!(html.contains("colspan=\"2\""));
        assert!(html.contains("width=\"640\""));
        assert!(html.contains("height=\"480\""));
        assert!(html.contains("loading=\"lazy\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("alert(1)"));
    }

    #[test]
    fn sanitize_html_strips_xss_vectors_and_obfuscated_urls() {
        let dirty = concat!(
            r#"<div onclick="alert(1)" style="color:red">safe</div>"#,
            r#"<script>if (a < b) { alert("</p><img src=x onerror=alert(1)>"); }</script>"#,
            r#"<iframe src="https://evil.example" srcdoc="<script>alert(1)</script>">frame</iframe>"#,
            r#"<svg onload="alert(1)"><circle r="5"/></svg>"#,
            r#"<math><mi>x</mi></math>"#,
            r#"<form action="/steal"><input type="text" value="pw"><input type="checkbox" checked disabled></form>"#,
            r#"<a href="java&#x09;script:alert(1)">bad1</a>"#,
            r#"<a href="&#106;avascript:alert(2)">bad2</a>"#,
            r#"<a href="vbscript:msgbox(1)">bad3</a>"#,
            r#"<a href="data:text/html,<script>alert(1)</script>">bad4</a>"#,
            r#"<a href="//evil.example/payload">bad5</a>"#,
            r#"<img src="data:image/svg+xml;base64,PHN2Zz4=" alt="svg">"#,
            r#"<img src="data:image/png;base64,iVBORw0KGgo=" alt="png">"#,
        );

        let clean = sanitize_html(dirty);
        assert!(clean.contains("<div>safe</div>"));
        assert!(!clean.contains("onclick"));
        assert!(!clean.contains("style="));
        assert!(!clean.contains("<script"));
        assert!(!clean.contains("onerror"));
        assert!(!clean.contains("<iframe"));
        assert!(!clean.contains("srcdoc"));
        assert!(!clean.contains("<svg"));
        assert!(!clean.contains("<math"));
        assert!(!clean.contains("<form"));
        assert!(!clean.contains("type=\"text\""));
        assert!(!clean.contains("javascript:"));
        assert!(!clean.contains("vbscript:"));
        assert!(!clean.contains("data:text/html"));
        assert!(!clean.contains("//evil.example"));
        assert!(!clean.contains("image/svg+xml"));
        assert!(clean.contains("src=\"data:image/png;base64,iVBORw0KGgo=\""));
    }

    #[test]
    fn sanitize_html_preserves_checkbox_tasklists_and_balances_tags() {
        let task_md = "- [x] Done\n- [ ] Todo\n";
        let rendered = markdown_to_html(task_md);
        assert!(rendered.contains("<input type=\"checkbox\""));
        assert!(rendered.contains("checked=\"\""));
        assert!(rendered.contains("disabled=\"\""));

        let unbalanced = "<details open><summary title=\"1 > 0\">More</summary><div><mark>hit";
        let balanced = sanitize_html(unbalanced);
        assert_eq!(
            balanced,
            "<details open=\"\"><summary title=\"1 &gt; \
             0\">More</summary><div><mark>hit</mark></div></details>"
        );
    }
}
