use std::fmt::{Display, Formatter};
use crate::parser::unsupported::UNSUPPORTED_NOTATIONS;
use crate::utils::NotationMatching;

mod unsupported;

#[derive(Clone, Debug)]
pub(crate) struct ParsedDoxygen {
    pub brief: Option<String>,
    pub description: Option<String>,
    pub params: Option<Vec<Param>>,
    pub deprecated: Option<Deprecated>,
    pub todos: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub(crate) struct Param {
    pub arg_name: String,
    pub direction: Option<Direction>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct Deprecated {
    pub is_deprecated: bool,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum Direction {
    In,
    Out,
    InOut,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::In => f.write_str("In"),
            Direction::Out => f.write_str("In, Out"),
            Direction::InOut => f.write_str("Out"),
        }
    }
}

impl TryFrom<&str> for Direction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "in" {
            Ok(Direction::In)
        } else if value == "out" {
            Ok(Direction::Out)
        } else if value == "in,out" || value == "out,in" {
            Ok(Direction::InOut)
        } else {
            Err(())
        }
    }
}

pub(crate) fn parse_comment(input: &str) -> ParsedDoxygen {
    let input = input.to_string();
    let mut brief = String::new();
    let mut description = vec![];
    let mut params: Vec<Param> = vec![];
    let mut deprecated: Option<Deprecated> = None;
    let mut todos: Vec<String> = vec![];

    input
        .lines()
        .rev()
        .for_each(|v| {
            let v = v.trim();
            let mut v_split_whitespace = v.split_whitespace();
            if v.starts_with_notation("brief") || v.starts_with_notation("short") {
                brief = v.remove_notation("brief").remove_notation("short").trim().to_string();
            } else if v.starts_with_notation("param") {
                let mut raw_direction = v_split_whitespace.next().map(|v| v.to_string());
                if let Some(str) = raw_direction {
                    if !str.contains('[') || !str.contains(']') {
                        raw_direction = None;
                    } else {
                        let value = str.remove_notation("param").replace('[', "").replace(']', "");
                        raw_direction = Some(value)
                    }
                };
                let arg_name = v_split_whitespace.next().unwrap().to_string();
                let description = &v_split_whitespace.collect::<Vec<&str>>().join(" ");

                params.push(Param {
                    arg_name,
                    direction: raw_direction.map(|raw_direction| Direction::try_from(raw_direction.as_str()).unwrap()),
                    description: Some(description.to_owned())
                });
            } else if v.starts_with_notation("deprecated") {
                let message = v_split_whitespace.map(|v| v.to_string()).collect::<Vec<String>>();
                let message = &message[1..].to_vec().join(" ");
                let message = if message.is_empty() { None } else { Some(message) };

                deprecated = Some(Deprecated {
                    is_deprecated: true,
                    message: message.map(|message| message.to_owned()),
                })
            } else if v.starts_with_notation("todo") {
                todos.push(v.remove_notation("todo").trim().to_string())
            } else if v.starts_with_notation("details") {
                description.push(v.remove_notation("details"))
            } else {
                for notation in UNSUPPORTED_NOTATIONS {
                    if v.contains_notation(notation) {
                        return;
                    }
                }

                description.push(v.to_string());
            }
        });

    params.reverse();
    description.reverse();
    todos.reverse();

    ParsedDoxygen {
        brief: if brief.is_empty() { None } else { Some(brief) },
        description: if description.is_empty() { None } else { Some(description.join("\n").trim().to_string()) },
        params: if params.is_empty() { None } else { Some(params) },
        todos: if todos.is_empty() { None } else { Some(todos) },
        deprecated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_param() {
        let doxygen = parse_comment("@param random Random thing lmao\n@param[in] goes_in This goes in lmao");

        let first_param = doxygen.params.as_ref().unwrap();
        let first_param = first_param.get(0).unwrap();
        assert_eq!(first_param.arg_name, "random");
        assert_eq!(first_param.description, Some("Random thing lmao".to_string()));
        assert_eq!(first_param.direction, None);

        let second_param = doxygen.params.as_ref().unwrap();
        let second_param = second_param.get(1).unwrap();
        assert_eq!(second_param.arg_name, "goes_in");
        assert_eq!(second_param.description, Some("This goes in lmao".to_string()));
        assert_eq!(second_param.direction, Some(Direction::In));
    }

    #[test]
    fn parses_brief() {
        let doxygen = parse_comment("@brief This function does things");

        assert_eq!(doxygen.brief, Some("This function does things".to_string()));
    }

    #[test]
    fn parses_description() {
        let doxygen = parse_comment("@brief This is a function\n\nThis is the description of the thing.\nYou should do things with this function.\nOr not, I don't really care.");

        assert_eq!(doxygen.description, Some("This is the description of the thing.\nYou should do things with this function.\nOr not, I don't really care.".to_string()))
    }

    #[test]
    fn parses_deprecated() {
        let doxygen = parse_comment("@deprecated This function is pure spaghetti lmao\n\n@brief Creates a single spaghetti");

        let deprecated = doxygen.deprecated.unwrap();
        assert_eq!(deprecated.is_deprecated, true);
        assert_eq!(deprecated.message, Some("This function is pure spaghetti lmao".to_string()));
    }

    #[test]
    fn parses_details() {
        let doxygen = parse_comment("@brief This does things\n\n@details This does _advanced_ things\nAnd the _advanced_ things are not easy");

        let description = doxygen.description.unwrap();
        assert_eq!(description, "This does _advanced_ things\nAnd the _advanced_ things are not easy");
    }

    #[test]
    fn parses_todo() {
        let doxygen = parse_comment("@brief This is WIP\n\n@todo Fix the bug where the C: drive is deleted");

        let todos = doxygen.todos.unwrap();
        assert_eq!(todos.get(0).unwrap(), "Fix the bug where the C: drive is deleted");
    }

    #[test]
    fn parses_advanced_doxygen() {
        let doxygen = parse_comment("@brief Creates a new dog.\n\nCreates a new Dog named `_name` with half of its maximum energy.\n\n@param _name The dog's name.");

        let first_param = doxygen.params.unwrap();
        let first_param = first_param.first().unwrap();
        assert_eq!(doxygen.brief, Some("Creates a new dog.".to_string()));
        assert_eq!(doxygen.description, Some("Creates a new Dog named `_name` with half of its maximum energy.".to_string()));
        assert_eq!(first_param.arg_name, "_name".to_string());
        assert_eq!(first_param.description, Some("The dog's name.".to_string()));
        assert_eq!(first_param.direction, None);
    }
}