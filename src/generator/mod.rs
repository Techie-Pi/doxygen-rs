use crate::ast::ParsedDoxygen;

pub(crate) fn generate_rustdoc(doxygen: ParsedDoxygen) -> String {
    let mut rustdoc = String::new();

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
        rustdoc += description.as_str();
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

    if let Some(todos) = doxygen.todos {
        rustdoc += "# To Do\n\n";
        for todo in todos {
            rustdoc += format!("* {}", todo).as_str();
        }

        rustdoc += "\n";
    }

    rustdoc
}