use std::fmt::{Display, Formatter};
use crate::ast::unsupported::UNSUPPORTED_NOTATIONS;
use crate::parser::Value;
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

#[derive(Debug, Clone)]
enum MoveBufferTo {
    Description,
    ToDo,
}

pub(crate) fn generate_ast(input: Vec<Value>) -> ParsedDoxygen {
    let mut brief = None;
    let mut deprecated = None;
    let mut todos = vec![];
    let mut params = vec![];
    let mut description = vec![];

    let mut currently_saving_paragraph = false;
    let mut paragraph_buffer = vec![];
    let mut move_buffer_to = None;

    let mut previous_value = None;
    for value in input {
        let value_clone = value.clone();
        match value {
            Value::Notation(notation, content) => {
                if notation.starts_with_notation("brief") {
                    brief = Some(content);
                } else if notation.starts_with_notation("deprecated") {
                    deprecated = Some(Deprecated {
                        is_deprecated: true,
                        message: Some(content),
                    });
                } else if notation.starts_with_notation("details") {
                    currently_saving_paragraph = true;
                    move_buffer_to = Some(MoveBufferTo::Description);
                    paragraph_buffer.push(content);
                } else if notation.starts_with_notation("todo") {
                    currently_saving_paragraph = true;
                    move_buffer_to = Some(MoveBufferTo::ToDo);
                    paragraph_buffer.push(content);
                } else if notation.starts_with_notation("param") {
                    let direction = {
                        let raw_direction = notation.remove_notation("param");
                        if raw_direction.is_empty() || raw_direction.starts_with("[]") {
                            None
                        } else if raw_direction.starts_with("[in]") {
                            Some(Direction::In)
                        } else if raw_direction.starts_with("[out]") {
                            Some(Direction::Out)
                        } else if raw_direction.starts_with("[in,out]") || raw_direction.starts_with("[out,in]") {
                            Some(Direction::InOut)
                        } else {
                            None
                        }
                    };

                    let arg_name = {
                        let split = content.split_whitespace().map(|v| v.to_string()).collect::<Vec<String>>();
                        split.first().unwrap().to_owned()
                    };

                    let description = {
                        let split = content.split_whitespace().map(|v| v.to_string()).collect::<Vec<String>>();
                        let description = split[1..].to_vec().join(" ");
                        if description.is_empty() {
                            None
                        } else {
                            Some(description)
                        }
                    };

                    params.push(Param {
                        arg_name,
                        direction,
                        description,
                    })
                }
            }
            Value::Text(content) => {
                if currently_saving_paragraph {
                    paragraph_buffer.push(content);
                } else {
                    description.push(content);
                }
            }
            Value::Separator => {
                if currently_saving_paragraph {
                    if let Some(move_buffer_to) = move_buffer_to.clone() {
                        match move_buffer_to {
                            MoveBufferTo::Description => description.append(&mut paragraph_buffer.clone()),
                            MoveBufferTo::ToDo => todos.push(paragraph_buffer.join("\n")),
                        }
                    }
                    currently_saving_paragraph = false;
                    paragraph_buffer = vec![];
                    move_buffer_to = None;
                }
            }
            Value::Unknown => {}
        }

        previous_value = Some(value_clone);
    }

    let description = if description.is_empty() { None } else { Some(description.join("\n")) };
    let todos = if todos.is_empty() { None } else { Some(todos) };
    let params = if params.is_empty() { None } else { Some(params) };

    ParsedDoxygen {
        brief,
        description,
        params,
        deprecated,
        todos,
    }
}

#[cfg(test)]
mod tests {
    use crate::parser;
    use super::*;

    #[test]
    fn parses_param() {
        let doxygen = generate_ast(parser::parse_comment("@param random Random thing lmao\n@param[in] goes_in This goes in lmao"));

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
        let doxygen = generate_ast(parser::parse_comment("@brief This function does things"));

        assert_eq!(doxygen.brief, Some("This function does things".to_string()));
    }

    #[test]
    fn parses_description() {
        let doxygen = generate_ast(parser::parse_comment("@brief This is a function\n\nThis is the description of the thing.\nYou should do things with this function.\nOr not, I don't really care."));

        assert_eq!(doxygen.description, Some("This is the description of the thing.\nYou should do things with this function.\nOr not, I don't really care.".to_string()))
    }

    #[test]
    fn parses_deprecated() {
        let doxygen = generate_ast(parser::parse_comment("@deprecated This function is pure spaghetti lmao\n\n@brief Creates a single spaghetti"));

        let deprecated = doxygen.deprecated.unwrap();
        assert_eq!(deprecated.is_deprecated, true);
        assert_eq!(deprecated.message, Some("This function is pure spaghetti lmao".to_string()));
    }

    #[test]
    fn parses_details() {
        let doxygen = generate_ast(parser::parse_comment("@brief This does things\n\n@details This does _advanced_ things\nAnd the _advanced_ things are not easy"));

        let description = doxygen.description.unwrap();
        assert_eq!(description, "This does _advanced_ things\nAnd the _advanced_ things are not easy");
    }

    #[test]
    fn parses_todo() {
        let doxygen = generate_ast(parser::parse_comment("@brief This is WIP\n\n@todo Fix the bug where the C: drive is deleted"));

        let todos = doxygen.todos.unwrap();
        assert_eq!(todos.get(0).unwrap(), "Fix the bug where the C: drive is deleted");
    }

    #[test]
    fn parses_advanced_doxygen() {
        let doxygen = generate_ast(parser::parse_comment("@brief Creates a new dog.\n\nCreates a new Dog named `_name` with half of its maximum energy.\n\n@param _name The dog's name."));

        let first_param = doxygen.params.unwrap();
        let first_param = first_param.first().unwrap();
        assert_eq!(doxygen.brief, Some("Creates a new dog.".to_string()));
        assert_eq!(doxygen.description, Some("Creates a new Dog named `_name` with half of its maximum energy.".to_string()));
        assert_eq!(first_param.arg_name, "_name".to_string());
        assert_eq!(first_param.description, Some("The dog's name.".to_string()));
        assert_eq!(first_param.direction, None);
    }
}