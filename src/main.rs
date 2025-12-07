use axum::{
    extract::State,
    routing::get,
    Router,
};
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::fs;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

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
    });

    // Build the router with a dynamic route based on the UUID
    // The route is exactly /{uuid}, preventing access without it.
    let app = Router::new()
        .route(&format!("/{}", sub_uuid), get(handle_subscription))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Server running on http://0.0.0.0:{}/{}", args.port, sub_uuid);
    println!("Subscription link: http://127.0.0.1:{}/{}", args.port, sub_uuid);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_subscription(State(state): State<Arc<AppState>>) -> Result<String, String> {
    // Read the file content
    let content = fs::read_to_string(&state.file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Process lines: trim, remove empty lines, remove comments
    let mut processed_lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        processed_lines.push(trimmed);
    }

    // Join with newlines
    let joined_content = processed_lines.join("\n");

    // Base64 encode
    let encoded = general_purpose::STANDARD.encode(joined_content);

    Ok(encoded)
}