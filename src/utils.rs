pub trait NotationMatching {
    fn starts_with_notation(&self, notation: &str) -> bool;
    fn replace_notation(&self, notation: &str, to: &str) -> String;
    fn contains_notation(&self, notation: &str) -> bool;
    fn remove_notation(&self, notation: &str) -> String;
    fn contains_any_notation(&self) -> Option<String>;
}

macro_rules! notation_matching {
    ($t:ty) => {
        impl NotationMatching for $t {
            fn starts_with_notation(&self, notation: &str) -> bool {
                self.starts_with(format!("@{}", notation).as_str()) || self.starts_with(format!("\\{}", notation).as_str()) || self.starts_with(format!("\\\\{}", notation).as_str())
            }

            fn replace_notation(&self, notation: &str, to: &str) -> String {
                self.replace(format!("@{}", notation).as_str(), to).replace(format!("\\{}", notation).as_str(), to).replace(format!("\\\\{}", notation).as_str(), to)
            }

            fn contains_notation(&self, notation: &str) -> bool {
                self.contains(format!("@{}", notation).as_str()) || self.contains(format!("\\{}", notation).as_str()) || self.contains(format!("\\\\{}", notation).as_str())
            }

            fn remove_notation(&self, notation: &str) -> String {
                self.replace_notation(notation, "")
            }

            fn contains_any_notation(&self) -> Option<String> {
                if self.starts_with("@") || self.starts_with("\\") || self.starts_with("\\\\") {
                    let split = self.split_whitespace().collect::<Vec<&str>>();
                    Some(split.first()?.to_string())
                } else {
                    None
                }
            }
        }
    }
}

notation_matching!(&str);
notation_matching!(String);