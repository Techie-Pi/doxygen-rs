#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum LexItem {
    At(String),
    Paren(char),
    Word(String),
    Whitespace(char),
    NewLine,
}

impl LexItem {
    pub(crate) fn push_to(&self, acc: &mut String) {
        match self {
            LexItem::At(s) => acc.push_str(s),
            LexItem::Paren(c) => acc.push(*c),
            LexItem::Word(s) => acc.push_str(s),
            LexItem::Whitespace(c) => acc.push(*c),
            LexItem::NewLine => acc.push('\n'),
        }
    }
}

pub(crate) fn lex(input: String) -> Vec<LexItem> {
    let mut result = vec![];

    for c in input.chars() {
        match c {
            '@' => {
                result.push(LexItem::At(c.into()));
            }
            '\\' => {
                if let Some(value) = result.last_mut() {
                    match value {
                        LexItem::At(v) => {
                            if v == "\\" {
                                *v += "\\"
                            } else {
                                result.push(LexItem::At(c.into()))
                            }
                        }
                        _ => result.push(LexItem::At(c.into())),
                    }
                } else {
                    result.push(LexItem::At(c.into()));
                }
            }
            '{' | '}' => {
                result.push(LexItem::Paren(c));
            }
            ' ' | '\t' => {
                result.push(LexItem::Whitespace(c));
            }
            '\n' => {
                result.push(LexItem::NewLine);
            }
            _ => {
                if let Some(v) = result.last_mut() {
                    match v {
                        LexItem::Word(v) => *v += &c.to_string(),
                        _ => result.push(LexItem::Word(String::from(c))),
                    }
                } else {
                    result.push(LexItem::Word(String::from(c)))
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_notation() {
        let result = lex("@name Memory Management".into());
        assert_eq!(
            result,
            vec![
                LexItem::At("@".into()),
                LexItem::Word("name".into()),
                LexItem::Whitespace(' '),
                LexItem::Word("Memory".into()),
                LexItem::Whitespace(' '),
                LexItem::Word("Management".into())
            ]
        );

        let result = lex("\\name Memory Management".into());
        assert_eq!(
            result,
            vec![
                LexItem::At("\\".into()),
                LexItem::Word("name".into()),
                LexItem::Whitespace(' '),
                LexItem::Word("Memory".into()),
                LexItem::Whitespace(' '),
                LexItem::Word("Management".into())
            ]
        );

        let result = lex("\\\\name Memory Management".into());
        assert_eq!(
            result,
            vec![
                LexItem::At("\\\\".into()),
                LexItem::Word("name".into()),
                LexItem::Whitespace(' '),
                LexItem::Word("Memory".into()),
                LexItem::Whitespace(' '),
                LexItem::Word("Management".into())
            ]
        );
    }

    #[test]
    fn basic_groups() {
        let result = lex("@{\n* @name Memory Management\n@}".into());
        assert_eq!(
            result,
            vec![
                LexItem::At("@".into()),
                LexItem::Paren('{'),
                LexItem::NewLine,
                LexItem::Word("*".into()),
                LexItem::Whitespace(' '),
                LexItem::At("@".into()),
                LexItem::Word("name".into()),
                LexItem::Whitespace(' '),
                LexItem::Word("Memory".into()),
                LexItem::Whitespace(' '),
                LexItem::Word("Management".into()),
                LexItem::NewLine,
                LexItem::At("@".into()),
                LexItem::Paren('}')
            ]
        );
    }
}
