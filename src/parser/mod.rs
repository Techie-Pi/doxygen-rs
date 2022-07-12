use crate::utils::NotationMatching;

#[derive(Debug, Clone)]
pub enum Value {
    // (String, String) -> (Raw Notation, Raw Contents)
    Notation(String, String),
    // String -> Raw Contents
    Text(String),
    Separator,
    Unknown,
}

pub(crate) fn generate_ast(input: &str) -> Vec<Value> {
    let lines = input.split('\n').map(|v| v.to_string()).collect::<Vec<String>>();
    let mut ast = vec![];

    for line in lines {
        if let Some(notation) = line.contains_any_notation() {
            let split = line.split_whitespace().collect::<Vec<&str>>();
            ast.push(Value::Notation(notation.clone(), split[1..].to_vec().join(" ").to_string()));
        } else if line.is_empty() {
            ast.push(Value::Separator);
        } else {
            ast.push(Value::Text(line));
        }
    }

    ast
}

#[cfg(test)]
mod tests {
    use crate::parser::generate_ast;

    #[test]
    fn test() {
        let ast = generate_ast("@param random Random thing lmao\n\n@block This is going to be\nA block of text\nThis is crazy right??\n\nHello this is not anotated\n");
        println!("{:?}", ast);
    }
}

