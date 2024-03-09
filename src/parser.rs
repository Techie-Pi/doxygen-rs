use crate::lexer::{lex, LexItem};

const OPEN_PAREN: char = '{';
const CLOSED_PAREN: char = '}';

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedEndOfInput,
    UnexpectedInput {
        found: String,
        expected: Vec<String>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum GrammarItem {
    Notation {
        meta: Vec<String>,
        params: Vec<String>,
        tag: String,
    },
    Text(String),
    GroupStart,
    GroupEnd,
}

pub(crate) fn parse(input: String) -> Result<Vec<GrammarItem>, ParseError> {
    let lexed = lex(input);
    parse_items(lexed)
}

fn parse_items(input: Vec<LexItem>) -> Result<Vec<GrammarItem>, ParseError> {
    let mut grammar_items = vec![];
    let mut param_word_skip_count = 0;

    for (index, current) in input.iter().enumerate() {
        let rest = &input[index..];
        let next = rest.get(1);

        if matches!(current, LexItem::Word(_)) && param_word_skip_count > 0 {
            param_word_skip_count -= 1;
            continue;
        }

        // Do not do any formatting inside of code blocks
        let ends_code = matches!(current, LexItem::At(_))
            && matches!(next, Some(LexItem::Word(v)) if v == "endcode");
        if !ends_code {
            match &mut grammar_items[..] {
                [.., GrammarItem::Notation { tag, .. }] if tag == "code" => {
                    let mut text = String::new();
                    current.push_to(&mut text);

                    grammar_items.push(GrammarItem::Text(text));
                    continue;
                }
                [.., GrammarItem::Notation { tag, .. }, GrammarItem::Text(text)]
                    if tag == "code" =>
                {
                    current.push_to(text);
                    continue;
                }
                _ => {}
            }
        }

        match current {
            LexItem::At(_) => {
                if let Some(next) = next {
                    match next {
                        LexItem::Paren(v) => match *v {
                            OPEN_PAREN => grammar_items.push(GrammarItem::GroupStart),
                            CLOSED_PAREN => grammar_items.push(GrammarItem::GroupEnd),
                            _ => {
                                return Err(ParseError::UnexpectedInput {
                                    found: v.to_string(),
                                    expected: vec![OPEN_PAREN.into(), CLOSED_PAREN.into()],
                                })
                            }
                        },
                        LexItem::Word(v) => {
                            let mut meta = vec![];
                            let content;

                            let expects_params;

                            if v.starts_with("param") {
                                let value = v.split('[').collect::<Vec<_>>();
                                match value.get(1) {
                                    Some(&"in]") => meta.push("in".into()),
                                    Some(&"out]") => meta.push("out".into()),
                                    Some(&"in,out]") | Some(&"out,in]") => {
                                        meta.push("in".into());
                                        meta.push("out".into());
                                    }
                                    _ => match value.get(1) {
                                        None => {}
                                        Some(v) => {
                                            return Err(ParseError::UnexpectedInput {
                                                found: v.to_string(),
                                                expected: vec!["in]".into(), "out]".into()],
                                            })
                                        }
                                    },
                                }

                                content = "param";
                                expects_params = true;
                            } else {
                                content = v;

                                expects_params = match v.as_str() {
                                    "a" | "b" | "c" | "p" | "emoji" | "e" | "em" | "def"
                                    | "class" | "category" | "concept" | "enum" | "example"
                                    | "extends" | "file" | "sa" | "see" | "retval"
                                    | "exception" | "throw" | "throws" => true,
                                    _ => false,
                                };
                            }

                            let params = if expects_params {
                                rest.iter().skip(2).find_map(|next| match next {
                                    LexItem::Word(word) => Some(word),
                                    _ => None,
                                })
                            } else {
                                None
                            };

                            let params = if let Some(word) = params {
                                param_word_skip_count = 2;
                                vec![word.into()]
                            } else {
                                param_word_skip_count = 1;
                                vec![]
                            };

                            grammar_items.push(GrammarItem::Notation {
                                meta,
                                params,
                                tag: content.into(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            LexItem::Word(v) => {
                if let Some(prev) = grammar_items.last_mut() {
                    match prev {
                        GrammarItem::Text(text) => *text += v,
                        _ => grammar_items.push(GrammarItem::Text(v.into())),
                    }
                } else {
                    grammar_items.push(GrammarItem::Text(v.into()));
                }
            }
            LexItem::Whitespace(whitespace) => match &mut grammar_items[..] {
                [.., GrammarItem::Notation { tag, .. }] if tag == "code" => {
                    grammar_items.push(GrammarItem::Text((*whitespace).into()))
                }
                [.., GrammarItem::Notation { tag, .. }, GrammarItem::Text(text)]
                    if tag == "code" =>
                {
                    text.push(*whitespace)
                }
                [.., GrammarItem::Text(text)] if text.ends_with(' ') => {}
                [.., GrammarItem::Text(text)] => text.push(' '),
                [] => grammar_items.push(GrammarItem::Text(' '.into())),
                _ => grammar_items.push(GrammarItem::Text("".into())),
            },
            LexItem::NewLine => {
                if let Some(GrammarItem::Text(text)) = grammar_items.last_mut() {
                    *text += "\n"
                }
            }
            LexItem::Paren(v) => {
                if let Some(GrammarItem::Text(text)) = grammar_items.last_mut() {
                    *text += &v.to_string()
                }
            }
        }
    }

    Ok(grammar_items)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn simple_notation() {
        let result = parse("@name Memory Management".into()).unwrap();
        assert_eq!(
            result,
            vec![
                GrammarItem::Notation {
                    meta: vec![],
                    params: vec![],
                    tag: "name".into(),
                },
                GrammarItem::Text("Memory Management".into())
            ]
        );
    }

    #[test]
    pub fn paren_in_notation() {
        let result = parse("@note hoge_t = {a, b, c}".into()).unwrap();
        assert_eq!(
            result,
            vec![
                GrammarItem::Notation {
                    meta: vec![],
                    params: vec![],
                    tag: "note".into(),
                },
                GrammarItem::Text("hoge_t = {a, b, c}".into())
            ]
        );
    }

    #[test]
    pub fn param() {
        let result =
            parse("@param[in] random This is, without a doubt, a random argument.".into()).unwrap();
        assert_eq!(
            result,
            vec![
                GrammarItem::Notation {
                    meta: vec!["in".into()],
                    params: vec!["random".into()],
                    tag: "param".into(),
                },
                GrammarItem::Text(" This is, without a doubt, a random argument.".into())
            ]
        );
    }

    #[test]
    pub fn groups() {
        let result = parse("@{\n* @name Memory Management\n@}".into()).unwrap();
        assert_eq!(
            result,
            vec![
                GrammarItem::GroupStart,
                GrammarItem::Text("* ".into()),
                GrammarItem::Notation {
                    meta: vec![],
                    params: vec![],
                    tag: "name".into(),
                },
                GrammarItem::Text("Memory Management\n".into()),
                GrammarItem::GroupEnd
            ]
        );
    }

    #[test]
    pub fn trims_param_texts() {
        let result = parse(
            "@param[in]           var                                         Example description"
                .into(),
        )
        .unwrap();
        assert_eq!(
            result,
            vec![
                GrammarItem::Notation {
                    meta: vec!["in".into()],
                    params: vec!["var".into()],
                    tag: "param".into(),
                },
                GrammarItem::Text(" Example description".into())
            ]
        )
    }
}
