use xkpass::{Args, Case, List, generate_password};

pub enum TokenType {
    Session,
    URL,
    Extension,
}

pub fn generate(t_type: TokenType) -> String {
    let args = match t_type {
        TokenType::Session => Args {
            number: 4,
            separator: "".to_string(),
            list: List::Short2,
            case: Case::Capitalized,
        },
        TokenType::URL => Args {
            number: 3,
            separator: "".to_string(),
            list: List::Short1,
            case: Case::Capitalized,
        },
        TokenType::Extension => Args {
            number: 1,
            separator: "".to_string(),
            list: List::Short1,
            case: Case::Capitalized,
        },
    };

    generate_password(args)
}

pub fn extend(existing: &str) -> String {
    let new = generate(TokenType::Extension);
    return format!("{}{}", existing, new);
}
