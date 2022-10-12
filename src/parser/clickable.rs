use crate::utils::NotationMatching;

pub fn make_clickable(input: &str) -> String {
    make_refs_clickable(make_links_clickable(input).as_str())
}

fn make_links_clickable(input: &str) -> String {
    input
        .split_whitespace()
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
        .split_whitespace()
        .map(|v| {
            if apply_ref_to_next {
                let v = remove_trailing_dot_or_colon(v);

                apply_ref_to_next = false;
                format!("[`{}`]", v)
            } else {
                if v.contains_notation("ref") {
                    apply_ref_to_next = true;
                    "".to_owned()
                } else {
                    v.to_owned()
                }
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