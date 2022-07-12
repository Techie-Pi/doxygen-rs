#[derive(Clone, Debug)]
pub(crate) struct ParsedDoxygen {
    pub brief: Option<String>,
    pub description: Option<String>,
    pub params: Option<Vec<Param>>,
}

#[derive(Clone, Debug)]
pub(crate) struct Param {
    pub arg_name: String,
    pub direction: Option<Direction>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum Direction {
    In,
    Out,
    InOut,
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

    input
        .lines()
        .rev()
        .for_each(|v| {
            if v.starts_with("@brief") {
                brief = v.replace("@brief", "").trim().to_string();
            } else if v.starts_with("@param") {
                let mut v_split_whitespace = v.split_whitespace();

                let mut raw_direction = v_split_whitespace.next().map_or(None, |v| Some(v.to_string()));
                if let Some(str) = raw_direction {
                    if !str.contains("[") || !str.contains("]") {
                        raw_direction = None;
                    } else {
                        let value = str.replace("@param", "").replace("[", "").replace("]", "");
                        raw_direction = Some(value)
                    }
                };
                let arg_name = v_split_whitespace.next().unwrap().to_string();
                let description = &v_split_whitespace.collect::<Vec<&str>>().join(" ");

                params.push(Param {
                    arg_name,
                    direction: if let Some(raw_direction) = raw_direction { Some(Direction::try_from(raw_direction.as_str()).unwrap()) } else { None },
                    description: Some(description.to_owned())
                });
            } else if !v.contains("@") {
                description.push(v);
            }
        });

    params.reverse();
    description.reverse();

    ParsedDoxygen {
        brief: if brief.is_empty() { None } else { Some(brief) },
        description: if description.is_empty() { None } else { Some(description.join("\n").trim().to_string()) },
        params: if params.is_empty() { None } else { Some(params) },
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