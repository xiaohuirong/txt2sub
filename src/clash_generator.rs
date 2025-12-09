use anyhow::Result;
use serde::Serialize;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
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
    #[serde(rename = "trojan")]
    Trojan(TrojanProxy),
    #[serde(rename = "ss")]
    Shadowsocks(ShadowsocksProxy),
    #[serde(rename = "tuic")]
    Tuic(TuicProxy),
    #[serde(rename = "wireguard")]
    WireGuard(WireGuardProxy),
}

#[derive(Debug, Serialize, Clone)]
pub struct WireGuardProxy {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<String>,
    #[serde(rename = "private-key")]
    pub private_key: String,
    #[serde(rename = "public-key")]
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pre-shared-key")]
    pub pre_shared_key: Option<String>,
    #[serde(rename = "allowed-ips")]
    pub allowed_ips: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserved: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrojanProxy {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub password: String,
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
    #[serde(skip_serializing_if = "Option::is_none", rename = "flow")]
    pub flow: Option<String>,
    
    // Reality options for Trojan
    #[serde(skip_serializing_if = "Option::is_none", rename = "reality-opts")]
    pub reality_opts: Option<RealityOpts>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ShadowsocksProxy {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub password: String,
    pub cipher: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "plugin")]
    pub plugin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "plugin-opts")]
    pub plugin_opts: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TuicProxy {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub password: String, // Usually token
    #[serde(rename = "uuid")]
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "skip-cert-verify")]
    pub skip_cert_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "servername")]
    pub servername: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "alpn")]
    pub alpn: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "congestion-controller")]
    pub congestion_controller: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "zero-rtt")]
    pub zero_rtt: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
pub struct VlessProxy {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "flow")]
    pub flow: Option<String>,
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


pub fn generate_clash_yaml(links: Vec<String>, extra_proxies: Vec<Proxy>, template: Option<String>) -> Result<String> {
    let mut proxies = Vec::new();
    let mut proxy_names = Vec::new();

    // Add extra proxies (e.g. from WireGuard config)
    for proxy in extra_proxies {
        let name = match &proxy {
            Proxy::Vless(v) => v.name.clone(),
            Proxy::Vmess(v) => v.name.clone(),
            Proxy::Hysteria2(v) => v.name.clone(),
            Proxy::Trojan(v) => v.name.clone(),
            Proxy::Shadowsocks(v) => v.name.clone(),
            Proxy::Tuic(v) => v.name.clone(),
            Proxy::WireGuard(v) => v.name.clone(),
        };
        proxy_names.push(name);
        proxies.push(proxy);
    }

    for link in links {
        let proxy = if link.starts_with("vless://") {
            parse_vless(&link)
        } else if link.starts_with("vmess://") {
            parse_vmess(&link)
        } else if link.starts_with("hy2://") || link.starts_with("hysteria2://") {
            parse_hy2(&link)
        } else if link.starts_with("trojan://") {
            parse_trojan(&link)
        } else if link.starts_with("ss://") {
            parse_ss(&link)
        } else if link.starts_with("tuic://") {
            parse_tuic(&link)
        }
        else {
            None
        };

        if let Some(p) = proxy {
            // Extract name for groups
            let name = match &p {
                Proxy::Vless(v) => v.name.clone(),
                Proxy::Vmess(v) => v.name.clone(),
                Proxy::Hysteria2(v) => v.name.clone(),
                Proxy::Trojan(v) => v.name.clone(),
                Proxy::Shadowsocks(v) => v.name.clone(),
                Proxy::Tuic(v) => v.name.clone(),
                Proxy::WireGuard(v) => v.name.clone(),
            };
            proxy_names.push(name);
            proxies.push(p);
        }
    }

    if let Some(tmpl_str) = template {
        // --- Template Merging Logic ---
        let mut doc: YamlValue = serde_yaml::from_str(&tmpl_str)?;

        // 1. Merge Proxies
        // Ensure "proxies" key exists and is a sequence
        if doc.get("proxies").is_none() || doc.get("proxies").map_or(false, |v| v.is_null()) {
            if let Some(mapping) = doc.as_mapping_mut() {
                mapping.insert(YamlValue::String("proxies".to_string()), YamlValue::Sequence(Vec::new()));
            }
        }

        if let Some(proxies_seq) = doc.get_mut("proxies").and_then(|v| v.as_sequence_mut()) {
            for proxy in proxies {
                let proxy_val = serde_yaml::to_value(proxy)?;
                proxies_seq.push(proxy_val);
            }
        }

        // 2. Merge into "PROXY" Group
        // Ensure "proxy-groups" key exists and is a sequence
        if doc.get("proxy-groups").is_none() || doc.get("proxy-groups").map_or(false, |v| v.is_null()) {
            if let Some(mapping) = doc.as_mapping_mut() {
                mapping.insert(YamlValue::String("proxy-groups".to_string()), YamlValue::Sequence(Vec::new()));
            }
        }

        let mut proxy_group_found = false;
        
        if let Some(groups_seq) = doc.get_mut("proxy-groups").and_then(|v| v.as_sequence_mut()) {
            for group in groups_seq.iter_mut() {
                // Check if group name is "PROXY"
                let is_target_group = group.get("name")
                    .and_then(|n| n.as_str())
                    .map(|s| s == "PROXY")
                    .unwrap_or(false);

                if is_target_group {
                    proxy_group_found = true;
                    // Append generated proxy names to this group
                    if let Some(group_proxies) = group.get_mut("proxies").and_then(|v| v.as_sequence_mut()) {
                        for name in &proxy_names {
                            group_proxies.push(YamlValue::String(name.clone()));
                        }
                    } else {
                        // If "proxies" key is missing in the group, create it
                         if let Some(mapping) = group.as_mapping_mut() {
                            let mut new_proxies = Vec::new();
                            for name in &proxy_names {
                                new_proxies.push(YamlValue::String(name.clone()));
                            }
                            mapping.insert(YamlValue::String("proxies".to_string()), YamlValue::Sequence(new_proxies));
                         }
                    }
                    break; 
                }
            }

            // If "PROXY" group not found, create it
            if !proxy_group_found {
                 let mut new_group_proxies = Vec::new();
                 // Optionally add "Auto" or others if you want, but user asked for "all proxies"
                 for name in &proxy_names {
                     new_group_proxies.push(YamlValue::String(name.clone()));
                 }

                 let mut new_group = serde_yaml::Mapping::new();
                 new_group.insert(YamlValue::String("name".to_string()), YamlValue::String("PROXY".to_string()));
                 new_group.insert(YamlValue::String("type".to_string()), YamlValue::String("select".to_string()));
                 new_group.insert(YamlValue::String("proxies".to_string()), YamlValue::Sequence(new_group_proxies));
                 
                 groups_seq.push(YamlValue::Mapping(new_group));
            }
        }

        return Ok(serde_yaml::to_string(&doc)?);

    } else {
        // --- Default Logic (No Template) ---
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
    let flow = query.get("flow").map(|s| s.to_string());
    let allow_insecure = query.get("allowInsecure").map(|s| s == "1" || s == "true").unwrap_or(false);
    
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
        flow,
        udp: Some(true),
        tls: Some(security.is_some()), // simplified
        skip_cert_verify: if allow_insecure { Some(true) } else { None },
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
    let v: JsonValue = serde_json::from_str(&json_str).ok()?;

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

fn parse_trojan(link: &str) -> Option<Proxy> {
    let url = Url::parse(link).ok()?;
    let name = url.fragment().unwrap_or("Trojan Node").to_string();
    let query: HashMap<_, _> = url.query_pairs().collect();

    let server = url.host_str()?.to_string();
    let port = url.port()?;
    let password = url.username().to_string();

    let security = query.get("security").map(|s| s.to_string());
    let sni = query.get("sni").map(|s| s.to_string());
    let fp = query.get("fp").map(|s| s.to_string());
    let flow = query.get("flow").map(|s| s.to_string());

    // Reality options for Trojan
    let reality_opts = if security.as_deref() == Some("reality") {
        Some(RealityOpts {
            public_key: query.get("pbk").unwrap_or(&"".into()).to_string(),
            short_id: query.get("sid").unwrap_or(&"".into()).to_string(),
        })
    } else {
        None
    };

    Some(Proxy::Trojan(TrojanProxy {
        name,
        server,
        port,
        password,
        udp: Some(true),
        tls: Some(true), // Trojan usually implies TLS
        skip_cert_verify: Some(true),
        servername: sni,
        network: None, // Trojan network is usually tcp
        client_fingerprint: fp,
        flow,
        reality_opts,
    }))
}

fn parse_ss(link: &str) -> Option<Proxy> {
    let mut config_part = link.trim_start_matches("ss://");
    let name_part;

    // Split name and config
    if let Some(pos) = config_part.find('#') {
        name_part = config_part[pos + 1..].to_string();
        config_part = &config_part[..pos];
    } else {
        name_part = "Shadowsocks Node".to_string();
    }

    let decoded_config = general_purpose::STANDARD.decode(config_part).ok()?;
    let decoded_str = String::from_utf8(decoded_config).ok()?;

    let parts: Vec<&str> = decoded_str.split('@').collect();
    if parts.len() != 2 { return None; } // Expecting "method:password@server:port"

    let method_pass: Vec<&str> = parts[0].splitn(2, ':').collect();
    if method_pass.len() != 2 { return None; }
    let cipher = method_pass[0].to_string();
    let password = method_pass[1].to_string();

    let server_port: Vec<&str> = parts[1].splitn(2, ':').collect();
    if server_port.len() != 2 { return None; }
    let server = server_port[0].to_string();
    let port = server_port[1].parse::<u16>().ok()?;

    Some(Proxy::Shadowsocks(ShadowsocksProxy {
        name: name_part,
        server,
        port,
        password,
        cipher,
        udp: Some(true),
        network: None,
        plugin: None,
        plugin_opts: None,
    }))
}

fn parse_tuic(link: &str) -> Option<Proxy> {
    let url = Url::parse(link).ok()?;
    let name = url.fragment().unwrap_or("TUIC Node").to_string();
    let query: HashMap<_, _> = url.query_pairs().collect();

    let server = url.host_str()?.to_string();
    let port = url.port()?;
    
    // TUIC userinfo is typically uuid:password
    let userinfo = url.username().to_string();
    let userinfo_parts: Vec<&str> = userinfo.splitn(2, ':').collect();
    let uuid = userinfo_parts.get(0)?.to_string();
    let password = userinfo_parts.get(1).unwrap_or(&"").to_string(); // password might be empty or missing

    let sni = query.get("sni").map(|s| s.to_string());
    let congestion_controller = query.get("congestion_control").map(|s| s.to_string()); // Renamed
    let alpn_str = query.get("alpn").map(|s| s.to_string());
    let zero_rtt = query.get("zero_rtt").map(|s| s == "1" || s == "true");
    let insecure = query.get("insecure").map(|s| s == "1" || s == "true"); // Used for skip-cert-verify

    let alpn = alpn_str.map(|s| s.split(',').map(|a| a.to_string()).collect());

    Some(Proxy::Tuic(TuicProxy {
        name,
        server,
        port,
        password,
        uuid,
        udp: Some(true),
        tls: Some(true), // TUIC implies TLS
        skip_cert_verify: insecure,
        servername: sni,
        alpn,
        congestion_controller,
        zero_rtt,
    }))
}

pub fn parse_wireguard(content: &str) -> Option<Proxy> {
    let mut current_section = "";
    
    // Interface fields
    let mut private_key = None;
    let mut ip = None;
    let mut ipv6 = None;
    let mut mtu = None;
    
    // Peer fields
    let mut public_key = None;
    let mut endpoint = None;
    let mut allowed_ips = Vec::new();
    let mut pre_shared_key = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = &line[1..line.len()-1];
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            continue;
        }

        let key = parts[0].trim().to_lowercase();
        let value = parts[1].trim();

        match current_section.to_lowercase().as_str() {
            "interface" => {
                match key.as_str() {
                    "privatekey" => private_key = Some(value.to_string()),
                    "address" => {
                        let addrs: Vec<&str> = value.split(',').collect();
                        for addr in addrs {
                            let addr = addr.trim();
                            // Split CIDR if present
                            let ip_part = addr.split('/').next().unwrap_or(addr);
                            if ip_part.contains(':') {
                                ipv6 = Some(ip_part.to_string());
                            } else {
                                ip = Some(ip_part.to_string());
                            }
                        }
                    },
                    "mtu" => mtu = value.parse::<u32>().ok(),
                    _ => {}
                }
            },
            "peer" => {
                match key.as_str() {
                    "publickey" => public_key = Some(value.to_string()),
                    "endpoint" => endpoint = Some(value.to_string()),
                    "allowedips" => {
                        allowed_ips = value.split(',').map(|s| s.trim().to_string()).collect();
                    },
                    "presharedkey" => pre_shared_key = Some(value.to_string()),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // Validation
    if private_key.is_none() || public_key.is_none() || endpoint.is_none() || ip.is_none() {
        return None;
    }

    let endpoint_str = endpoint.unwrap();
    let (server, port) = if let Some(idx) = endpoint_str.rfind(':') {
        let host = &endpoint_str[..idx];
        let port_str = &endpoint_str[idx+1..];
        
        // Handle IPv6 literal in endpoint [::1]:port
        let host = if host.starts_with('[') && host.ends_with(']') {
            &host[1..host.len()-1]
        } else {
            host
        };
        
        (host.to_string(), port_str.parse::<u16>().unwrap_or(51820))
    } else {
        (endpoint_str, 51820)
    };
    
    if allowed_ips.is_empty() {
        allowed_ips.push("0.0.0.0/0".to_string());
        allowed_ips.push("::/0".to_string());
    }

    Some(Proxy::WireGuard(WireGuardProxy {
        name: "WireGuard".to_string(),
        server,
        port,
        ip: ip.unwrap(),
        ipv6,
        private_key: private_key.unwrap(),
        public_key: public_key.unwrap(),
        pre_shared_key,
        allowed_ips,
        udp: Some(true),
        mtu,
        reserved: None,
    }))
}