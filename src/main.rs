use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::fs;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

mod clash_generator;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the text file containing subscription links
    #[arg(short, long)]
    file: PathBuf,

    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Custom UUID for the subscription URL. If not provided, a random one will be generated.
    #[arg(short, long)]
    uuid: Option<String>,
}

#[derive(Clone)]
struct AppState {
    file_path: PathBuf,
    sub_uuid: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Determine the UUID to use
    let sub_uuid = args.uuid.unwrap_or_else(|| Uuid::new_v4().to_string());

    // Check if file exists to give early feedback
    if !args.file.exists() {
        eprintln!("Error: File {:?} does not exist.", args.file);
        std::process::exit(1);
    }

    let state = Arc::new(AppState {
        file_path: args.file.clone(),
        sub_uuid: sub_uuid.clone(), // Store the UUID in the app state
    });

    // Build the router with a fixed path, expecting the UUID as a query parameter
    let app = Router::new()
        .route("/sub", get(handle_subscription)) // Fixed path /sub
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Server running on http://0.0.0.0:{}/sub?token={}", args.port, sub_uuid);
    println!("Subscription link: http://127.0.0.1:{}/sub?token={}", args.port, sub_uuid);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}



async fn handle_subscription(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let token = params.get("token");

    // Check if token exists and matches the expected sub_uuid
    if token.is_none() || token.unwrap() != &state.sub_uuid {
        return Err((StatusCode::FORBIDDEN, "Invalid or missing token".to_string()));
    }

    // Read the file content
    let content = fs::read_to_string(&state.file_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read file: {}", e)))?;

    // Process lines: trim, remove empty lines, remove comments
    let mut processed_lines = Vec::new();
    let mut raw_links = Vec::new(); // Store raw strings for Clash generator

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        processed_lines.push(trimmed);
        raw_links.push(trimmed.to_string());
    }

    // Determine if Clash config is requested
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    
    let is_clash = user_agent.contains("clash") 
        || user_agent.contains("mihomo") 
        || user_agent.contains("stash")
        || params.get("flag").map(|v| v.as_str()) == Some("clash");

    if is_clash {
        // Generate Clash YAML
        let yaml_content = clash_generator::generate_clash_yaml(raw_links)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate Clash config: {}", e)))?;
        
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/yaml; charset=utf-8"));
        // Optional: add Content-Disposition to suggest a filename
        headers.insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename=\"config.yaml\""));
        
        return Ok((headers, yaml_content));
    }

    // Default: Base64 encode
    let joined_content = processed_lines.join("\n");
    let encoded = general_purpose::STANDARD.encode(joined_content);
    
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));

    Ok((headers, encoded))
}