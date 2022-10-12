//! Generate raw Rustdoc comments from a given [`crate::ast::ParsedDoxygen`]
//!
//! **The functions and structs here should _not_ be considered stable**

use crate::ast::ParsedDoxygen;

/// Generate raw Rustdoc comments from a given [`crate::ast::ParsedDoxygen`]
///
/// # Examples
/// ```
/// use doxygen_rs::ast::generate_ast;
/// use doxygen_rs::generator::generate_rustdoc;
/// use doxygen_rs::parser::parse_comment;
///
/// let parsed = parse_comment("@brief Random comment");
/// let ast = generate_ast(parsed);
/// let rustdoc = generate_rustdoc(ast);
/// ```
pub fn generate_rustdoc(doxygen: ParsedDoxygen) -> String {
    let mut rustdoc = String::new();

    if let Some(title) = doxygen.title {
        rustdoc += format!("# {}\n\n", title).as_str();
    }

    if let Some(deprecated) = doxygen.deprecated {
        if let Some(message) = deprecated.message {
            rustdoc += format!("**Warning!** This is deprecated! - {}", message).as_str();
        } else {
            rustdoc += "**Warning!** This is deprecated!".to_string().as_str();
        }
        rustdoc += "\n\n";
    }

    if let Some(brief) = doxygen.brief {
        rustdoc += brief.as_str();
        rustdoc += "\n\n";
    }

    if let Some(description) = doxygen.description {
        rustdoc += description.replace("< ", "").as_str();
        rustdoc += "\n\n";
    }

    if let Some(returns) = doxygen.returns {
        let mut returns = returns;
        returns.get_mut(0..1).unwrap().make_ascii_uppercase();

        rustdoc += format!("Returns: {}", returns).as_str();
        rustdoc += "\n\n";
    }

    if let Some(params) = doxygen.params {
        rustdoc += "# Arguments\n\n";
        for param in params {
            if let Some(description) = param.description {
                rustdoc += format!("* `{}` - {}", param.arg_name, description).as_str();
            } else {
                rustdoc += format!("* `{}`", param.arg_name).as_str();
            }

            if let Some(direction) = param.direction {
                rustdoc += format!(" [Direction: {}]", direction).as_str();
            }

            rustdoc += "\n";
        }
    }

    if let Some(notes) = doxygen.notes {
        rustdoc += "# Notes\n\n";
        for note in notes {
            rustdoc += format!("- {}\n\n", note.0).as_str();
        }
    }

    if let Some(todos) = doxygen.todos {
        rustdoc += "# To Do\n\n";
        for todo in todos {
            rustdoc += format!("* {}", todo).as_str();
        }

        rustdoc += "\n";
    }

    rustdoc
}