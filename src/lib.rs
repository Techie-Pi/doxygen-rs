use crate::parser::parse_comment;
use crate::transformer::transform_doxygen;

mod parser;
mod transformer;

pub fn transform(input: &str) -> String {
    let doxygen = parse_comment(input);
    transform_doxygen(doxygen)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_transform() {
        println!("{:?}", transform("@brief Creates a new dog.\n\nCreates a new Dog named `_name` with half of its maximum energy.\n\n@param _name The dog's name.\n@param[in] _test Test for In\n\n@deprecated"))
    }
}