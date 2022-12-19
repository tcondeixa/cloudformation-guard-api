use cfn_guard::{run_checks, ValidateInput};

pub async fn call_cfn_guard(
    data: String,
    rule: String,
    verbose: bool,
    rule_name: String,
) -> Result<serde_json::Value, String> {
    let result = match run_checks(
        ValidateInput {
            content: &data,
            file_name: "body",
        },
        ValidateInput {
            content: &rule,
            file_name: &rule_name,
        },
        verbose,
    ) {
        Ok(t) => t,
        Err(e) => (e.to_string()),
    };
    match serde_json::from_str(&result) {
        Ok(json_value) => Ok(json_value),
        Err(e) => Err(e.to_string()),
    }
}
