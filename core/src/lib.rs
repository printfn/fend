#![forbid(unsafe_code)]

mod ast;
mod num;
mod parser;
mod value;

#[derive(PartialEq, Eq, Debug)]
pub struct FendResult {
    main_result: String,
    other_info: Vec<String>,
}

impl FendResult {
    pub fn get_main_result(&self) -> &str {
        self.main_result.as_str()
    }

    pub fn get_other_info(&self) -> impl Iterator<Item = &str> {
        self.other_info.iter().map(|string| string.as_str())
    }
}

pub fn evaluate(input: &str) -> Result<FendResult, String> {
    let (_, input) = parser::skip_whitespace(input)?;
    if input.is_empty() {
        // no or blank input: return no output
        return Ok(FendResult {
            main_result: "".to_string(),
            other_info: vec![],
        });
    }
    let (parsed, input) = parser::parse_expression(input)?;
    if !input.is_empty() {
        return Err(format!("Unexpected input found: '{}'", input));
    }
    let result = ast::evaluate(parsed)?;
    Ok(FendResult {
        main_result: format!("{}", result),
        other_info: vec![],
    })
}
