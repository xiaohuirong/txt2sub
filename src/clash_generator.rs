use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use url::Url;
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum Proxy {
    #[serde(rename = "vless")]
    Vless(VlessProxy),
    #[serde(rename = "vmess")]
    Vmess(VmessProxy),
    #[serde(rename = "hysteria2")]
    Hysteria2(Hysteria2Proxy),
}

#[derive(Debug, Serialize, Clone)]
pub struct VlessProxy {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "skip-cert-verify")]
    pub skip_cert_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "servername")]
    pub servername: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "client-fingerprint")]
    pub client_fingerprint: Option<String>,
    
    // Reality options
    #[serde(skip_serializing_if = "Option::is_none", rename = "reality-opts")]
    pub reality_opts: Option<RealityOpts>,

    // WS options
    #[serde(skip_serializing_if = "Option::is_none", rename = "ws-opts")]
    pub ws_opts: Option<WsOpts>,
    
    // Grpc options
    #[serde(skip_serializing_if = "Option::is_none", rename = "grpc-opts")]
    pub grpc_opts: Option<GrpcOpts>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RealityOpts {
    #[serde(rename = "public-key")]
    pub public_key: String,
    #[serde(rename = "short-id")]
    pub short_id: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct WsOpts {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GrpcOpts {
    #[serde(rename = "grpc-service-name")]
    pub grpc_service_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct VmessProxy {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    #[serde(rename = "alterId")]
    pub alter_id: u16,
    pub cipher: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "skip-cert-verify")]
    pub skip_cert_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "servername")]
    pub servername: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "ws-opts")]
    pub ws_opts: Option<WsOpts>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Hysteria2Proxy {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sni: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "skip-cert-verify")]
    pub skip_cert_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "obfs-password")]
    pub obfs_password: Option<String>,
}


#[derive(Debug, Serialize)]
pub struct ClashConfig {
    pub proxies: Vec<Proxy>,
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub proxies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
}


pub fn generate_clash_yaml(links: Vec<String>) -> Result<String> {
    let mut proxies = Vec::new();
    let mut proxy_names = Vec::new();

    for link in links {
        let proxy = if link.starts_with("vless://") {
            parse_vless(&link)
        } else if link.starts_with("vmess://") {
            parse_vmess(&link)
        } else if link.starts_with("hy2://") || link.starts_with("hysteria2://") {
            parse_hy2(&link)
        } else {
            None
        };

        if let Some(p) = proxy {
            // Extract name for groups
            let name = match &p {
                Proxy::Vless(v) => v.name.clone(),
                Proxy::Vmess(v) => v.name.clone(),
                Proxy::Hysteria2(v) => v.name.clone(),
            };
            proxy_names.push(name);
            proxies.push(p);
        }
    }

    // Default Groups
    let mut groups = Vec::new();
    
    // Proxy Select Group
    let mut select_proxies = vec!["Auto".to_string()];
    select_proxies.extend(proxy_names.clone());
    groups.push(ProxyGroup {
        name: "Proxy".to_string(),
        group_type: "select".to_string(),
        proxies: select_proxies,
        url: None,
        interval: None,
    });

    // Auto Select Group
    groups.push(ProxyGroup {
        name: "Auto".to_string(),
        group_type: "url-test".to_string(),
        proxies: proxy_names,
        url: Some("http://www.gstatic.com/generate_204".to_string()),
        interval: Some(300),
    });

    let config = ClashConfig {
        proxies,
        proxy_groups: groups,
        rules: vec![
            "MATCH,Proxy".to_string(),
        ],
    };

    let yaml = serde_yaml::to_string(&config)?;
    Ok(yaml)
}

fn parse_vless(link: &str) -> Option<Proxy> {
    let url = Url::parse(link).ok()?;
    let name = url.fragment().unwrap_or("VLESS Node").to_string();
    let query: HashMap<_, _> = url.query_pairs().collect();

    let server = url.host_str()?.to_string();
    let port = url.port()?;
    let uuid = url.username().to_string();

    let security = query.get("security").map(|s| s.to_string());
    let type_ = query.get("type").map(|s| s.to_string());
    let sni = query.get("sni").map(|s| s.to_string());
    let fp = query.get("fp").map(|s| s.to_string());
    
    // Reality check
    let reality_opts = if security.as_deref() == Some("reality") {
        Some(RealityOpts {
            public_key: query.get("pbk").unwrap_or(&"".into()).to_string(),
            short_id: query.get("sid").unwrap_or(&"".into()).to_string(),
        })
    } else {
        None
    };

    let network = if type_.as_deref() == Some("tcp") && security.as_deref() == Some("reality") {
        // Clash usually treats this as just network: tcp + tls + reality-opts
        Some("tcp".to_string())
    } else {
        type_.clone()
    };
    
    // WS Opts
    let ws_opts = if network.as_deref() == Some("ws") {
        Some(WsOpts {
            path: query.get("path").unwrap_or(&"/".into()).to_string(),
            headers: Some(HashMap::from([("Host".to_string(), sni.clone().unwrap_or(server.clone()))]))
        })
    } else {
        None
    };
    
    // GRPC Opts
    let grpc_opts = if network.as_deref() == Some("grpc") {
         Some(GrpcOpts {
            grpc_service_name: query.get("serviceName").unwrap_or(&"".into()).to_string()
         })
    } else {
        None
    };

    Some(Proxy::Vless(VlessProxy {
        name,
        server,
        port,
        uuid,
        udp: Some(true),
        tls: Some(security.is_some()), // simplified
        skip_cert_verify: Some(true),
        servername: sni,
        network,
        client_fingerprint: fp,
        reality_opts,
        ws_opts,
        grpc_opts,
    }))
}

fn parse_vmess(link: &str) -> Option<Proxy> {
    let base64_part = link.trim_start_matches("vmess://");
    let decoded_bytes = general_purpose::STANDARD.decode(base64_part).ok()?;
    let json_str = String::from_utf8(decoded_bytes).ok()?;
    let v: Value = serde_json::from_str(&json_str).ok()?;

    let name = v["ps"].as_str().unwrap_or("VMess Node").to_string();
    let server = v["add"].as_str()?.to_string();
    let port = v["port"].as_str().and_then(|s| s.parse::<u16>().ok())?;
    let vmess_uuid = v["id"].as_str()?.to_string(); // Renamed to vmess_uuid
    let aid = v["aid"].as_str().and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
    let net = v["net"].as_str().unwrap_or("tcp").to_string();
    let _type_ = v["type"].as_str().unwrap_or("none"); // Prefixed with _ to silence warning
    let host = v["host"].as_str().unwrap_or("");
    let path = v["path"].as_str().unwrap_or("");
    let tls_str = v["tls"].as_str().unwrap_or("");
    
    let tls = if tls_str == "tls" { Some(true) } else { None };

    let ws_opts = if net == "ws" {
         Some(WsOpts {
            path: if path.is_empty() { "/".to_string() } else { path.to_string() },
            headers: if !host.is_empty() { 
                Some(HashMap::from([("Host".to_string(), host.to_string())])) 
            } else { 
                None 
            }
        })
    } else {
        None
    };

    Some(Proxy::Vmess(VmessProxy {
        name,
        server,
        port,
        uuid: vmess_uuid, // Used vmess_uuid here
        alter_id: aid, // Renamed to alter_id
        cipher: "auto".to_string(),
        udp: Some(true),
        tls,
        skip_cert_verify: Some(true),
        servername: if !host.is_empty() { Some(host.to_string()) } else { None },
        network: Some(net),
        ws_opts,
    }))
}

fn parse_hy2(link: &str) -> Option<Proxy> {
    let url = Url::parse(link).ok()?;
    let name = url.fragment().unwrap_or("Hy2 Node").to_string();
    let query: HashMap<_, _> = url.query_pairs().collect();

    let server = url.host_str()?.to_string();
    let port = url.port()?;
    let password = url.username().to_string();
    
    let sni = query.get("sni").map(|s| s.to_string());
    let obfs = query.get("obfs").map(|s| s.to_string());
    let obfs_password = query.get("obfs-password").map(|s| s.to_string());

    Some(Proxy::Hysteria2(Hysteria2Proxy {
        name,
        server,
        port,
        password,
        sni,
        skip_cert_verify: Some(true),
        obfs,
        obfs_password,
    }))
}
