use axum::{routing::get, routing::post, Router};
use std::{collections::HashMap, env, fs, net::SocketAddr};
use tracing::{debug, error, info, span, warn, Level};
use tracing_subscriber;

mod guard;
mod handlers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut rules: HashMap<String, String> = HashMap::new();

    match env::var("CF_GUARD_RULES") {
        Err(_) => info!("ENV for rules not defined"),
        Ok(path) => {
            info!("Using Rules from {}", path);
            rules = load_rules(path);
        }
    }

    let app = Router::with_state(rules)
        .route("/healthz", get(handlers::health_handler))
        .route("/validate", post(handlers::validate_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("starting server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn load_rules(dir_path: String) -> HashMap<String, String> {
    let mut rules = HashMap::new();

    let paths = fs::read_dir(dir_path).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let filename = path.display();

        info!("Load file name: {}", &filename);
        let rule = std::fs::read_to_string(filename.to_string()).expect("could not open file");
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        info!("Rule {}: {:?}", &name, &rule);
        rules.insert(name, rule);
    }

    return rules;
}
