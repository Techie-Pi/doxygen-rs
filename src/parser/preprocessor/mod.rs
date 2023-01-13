use crate::parser::preprocessor::emojis::EMOJIS;
// TODO: Improve preprocessor architecture
use crate::utils::NotationMatching;

mod emojis;

pub fn preprocess_line(input: &str) -> String {
    render_code(
        make_italic_text(
            add_emojis(
                make_refs_clickable(
                    make_links_clickable(input).as_str()
                ).as_str()
            ).as_str()
        ).as_str()
    )
}

fn add_emojis(input: &str) -> String {
    let mut apply_emoji_to_next = false;
    input
    .split(char::is_whitespace)
        .map(|v| {
            if apply_emoji_to_next {
                apply_emoji_to_next = false;
                EMOJIS.get(v.replace(":", "").as_str()).unwrap_or(&"Unknown emoji").to_string()
            } else if v.contains_notation("emoji") {
                apply_emoji_to_next = true;
                "".to_owned()
            } else {
                v.to_owned()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn make_italic_text(input: &str) -> String {
    let mut apply_italic_to_next = false;
    input
        .split(char::is_whitespace)
        .map(|v| {
            if apply_italic_to_next {
                apply_italic_to_next = false;
                format!("*{}*", v)
            } else if v.contains_notation("a") || v.contains_notation("em") || v.contains_notation("e") {
                apply_italic_to_next = true;
                "".to_owned()
            } else {
                v.to_owned()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn make_links_clickable(input: &str) -> String {
    input
    .split(char::is_whitespace)
        .map(|v| {
            if v.starts_with("http://") || v.starts_with("https://") {
                let v = remove_trailing_dot_or_colon(v);
                format!("<{}>", v)
            } else {
                v.to_owned()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn make_refs_clickable(input: &str) -> String {
    let mut apply_ref_to_next = false;
    input
    .split(char::is_whitespace)
        .map(|v| {
            if apply_ref_to_next {
                let v = remove_trailing_dot_or_colon(v);

                apply_ref_to_next = false;
                format!("[`{}`]", v)
            } else if v.contains_notation("ref") || v.contains_notation("sa") || v.contains_notation("see") {
                apply_ref_to_next = true;
                "".to_owned()
            } else {
                v.to_owned()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn render_code(input: &str) -> String {
    input
    .split(char::is_whitespace)
        .map(|v| {
            if v.contains_notation("code") {
                "```\n".to_string()
            } else if v.contains_notation("endcode") {
                "```".to_owned()
            } else {
                v.to_owned()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn remove_trailing_dot_or_colon(input: &str) -> &str {
    let last_char = input.chars().last();
    if last_char == Some('.') || last_char == Some(',') {
        &input[..input.len() - 1]
    } else {
        input
    }
}