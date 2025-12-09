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
    file: Option<PathBuf>,

    /// Path to the WireGuard configuration file
    #[arg(short, long)]
    wireguard: Option<PathBuf>,

    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Host to listen on
    #[arg(short = 'L', long, default_value = "0.0.0.0")]
    host: String,

    /// Custom UUID for the subscription URL. If not provided, a random one will be generated.
    #[arg(short, long)]
    uuid: Option<String>,

    /// Path to the Clash config template (optional)
    #[arg(short, long)]
    template: Option<PathBuf>,

    /// Path to output the generated Clash config file. If specified, the server will not start.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Clone)]
struct AppState {
    file_path: Option<PathBuf>,
    wireguard_path: Option<PathBuf>,
    sub_uuid: String,
    template_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Determine the UUID to use
    let sub_uuid = args.uuid.unwrap_or_else(|| Uuid::new_v4().to_string());

    // Check if at least one source is provided
    if args.file.is_none() && args.wireguard.is_none() {
        eprintln!("Error: You must provide either --file or --wireguard.");
        std::process::exit(1);
    }

    // Check file existence
    if let Some(path) = &args.file {
        if !path.exists() {
            eprintln!("Error: File {:?} does not exist.", path);
            std::process::exit(1);
        }
    }
    if let Some(path) = &args.wireguard {
        if !path.exists() {
            eprintln!("Error: WireGuard file {:?} does not exist.", path);
            std::process::exit(1);
        }
    }
    
    if let Some(tmpl) = &args.template {
         if !tmpl.exists() {
            eprintln!("Error: Template file {:?} does not exist.", tmpl);
            std::process::exit(1);
        }
    }

    if let Some(output_path) = args.output {
        let mut raw_links = Vec::new();
        if let Some(path) = &args.file {
            let content = fs::read_to_string(path).await?;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                    continue;
                }
                raw_links.push(trimmed.to_string());
            }
        }

        let mut extra_proxies = Vec::new();
        if let Some(path) = &args.wireguard {
            let content = fs::read_to_string(path).await?;
            if let Some(proxy) = clash_generator::parse_wireguard(&content) {
                extra_proxies.push(proxy);
            } else {
                 eprintln!("Warning: Failed to parse WireGuard config from {:?}", path);
            }
        }

        let template_content = if let Some(path) = &args.template {
            Some(fs::read_to_string(path).await?)
        } else {
            None
        };

        let yaml_content = clash_generator::generate_clash_yaml(raw_links, extra_proxies, template_content)?;
        fs::write(&output_path, yaml_content).await?;
        println!("Clash config written to {:?}", output_path);
        return Ok(())
    }

    let state = Arc::new(AppState {
        file_path: args.file.clone(),
        wireguard_path: args.wireguard.clone(),
        sub_uuid: sub_uuid.clone(), // Store the UUID in the app state
        template_path: args.template.clone(),
    });

    // Build the router with a fixed path, expecting the UUID as a query parameter
    let app = Router::new()
        .route("/sub", get(handle_subscription)) // Fixed path /sub
        .with_state(state);

    let host_ip: std::net::IpAddr = args.host.parse().expect("Invalid host IP address");
    let addr = SocketAddr::from((host_ip, args.port));
    println!("Server running on http://{}:{}/sub?token={}", args.host, args.port, sub_uuid);
    if args.host == "0.0.0.0" {
        println!("Subscription link: http://127.0.0.1:{}/sub?token={}", args.port, sub_uuid);
    } else {
        println!("Subscription link: http://{}:{}/sub?token={}", args.host, args.port, sub_uuid);
    }

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

    let mut raw_links = Vec::new(); 
    let mut processed_lines = Vec::new();

    if let Some(path) = &state.file_path {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read file: {}", e)))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }
            processed_lines.push(trimmed.to_string());
            raw_links.push(trimmed.to_string());
        }
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
        // Read template if available
        let template_content = if let Some(path) = &state.template_path {
            let tmpl = fs::read_to_string(path)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read template: {}", e)))?;
            Some(tmpl)
        } else {
            None
        };

        let mut extra_proxies = Vec::new();
        if let Some(path) = &state.wireguard_path {
            let content = fs::read_to_string(path).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read WG file: {}", e)))?;
             if let Some(proxy) = clash_generator::parse_wireguard(&content) {
                extra_proxies.push(proxy);
            }
        }

        // Generate Clash YAML
        let yaml_content = clash_generator::generate_clash_yaml(raw_links, extra_proxies, template_content)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate Clash config: {}", e)))?;
        
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/yaml; charset=utf-8"));
        
        return Ok((headers, yaml_content));
    }

    // Default: Base64 encode
    // Note: If only WireGuard file is provided, processed_lines will be empty.
    // This is expected behavior as Base64 sub usually implies a list of links.
    let joined_content = processed_lines.join("\n");
    let encoded = general_purpose::STANDARD.encode(joined_content);
    
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));

    Ok((headers, encoded))
}
