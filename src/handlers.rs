use crate::guard::call_cfn_guard;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, span, warn, Level};

pub async fn health_handler() -> impl IntoResponse {
    let span = span!(Level::INFO, "Starting Health");
    let _enter = span.enter();
    info!("Health Check {}", StatusCode::OK);

    return StatusCode::OK;
}

#[derive(Debug, Deserialize)]
pub struct Request {
    #[serde(default = "bool::default")]
    verbose: bool,
    manifests: Vec<Manifest>,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    name: String,
    data: String,
    rules_names: Option<Vec<String>>,
    rule: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    results: Vec<Output>,
}

#[derive(Debug, Serialize)]
pub struct Output {
    name: String,
    result: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ResultError {
    error: String,
}

pub async fn validate_handler(
    State(rules): State<HashMap<String, String>>,
    Json(payload): Json<Request>,
) -> impl IntoResponse {
    let span = span!(Level::INFO, "Starting validate");
    let _enter = span.enter();

    let mut response = Response {
        results: Vec::new(),
    };

    for man in payload.manifests {
        let mut defined_rule = String::new();
        let mut defined_rule_name = String::new();

        match man.rules_names {
            None => match man.rule {
                None => {
                    error!("Empty rules and rule_names in request");
                }
                Some(rule) => {
                    defined_rule = rule;
                    defined_rule_name = "body".to_string();
                    info!("Validating against request rule {}", &defined_rule_name);
                }
            },
            Some(rules_names) => {
                for rule_name in rules_names {
                    if rules.contains_key(&rule_name) {
                        defined_rule.push_str(rules.get(&rule_name).unwrap());
                        defined_rule_name.push_str(&format!("{}/", rule_name))
                    } else {
                        warn!("Rule name {} not found", rule_name);
                        defined_rule = "".to_string();
                        defined_rule_name = "".to_string();
                        break;
                    }
                }
                info!("Validating against local rule {}", &defined_rule_name);
            }
        }

        if defined_rule == "" || defined_rule_name == "" {
            warn!("Manifest without request or local rule, skipping validation");
            match serde_json::from_str(
                "{\"error\": \"Manifest without request or local rule, skipping validation\"}",
            ) {
                Ok(r) => {
                    response.results.push(Output {
                        name: man.name,
                        result: r,
                    });
                }
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)),
            }

            continue;
        }

        match call_cfn_guard(man.data, defined_rule, payload.verbose, defined_rule_name).await {
            Ok(out) => {
                info!("Result for Manifest {}: {:?}", &man.name, &out);
                response.results.push(Output {
                    name: man.name,
                    result: out,
                });
            }
            Err(e) => {
                error!("Error calling cfn_guard: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
            }
        };
    }

    return (StatusCode::OK, Json(response));
}
