# txt2sub: Universal Subscription Link Converter for Clash/v2rayN

`txt2sub` is a Rust-based command-line interface (CLI) tool and web server that converts a local text file containing various proxy subscription links into an online subscription URL. It supports generating both standard Base64-encoded subscription lists (compatible with v2rayN, Shadowrocket, etc.) and Clash-compatible YAML configuration files, with the added flexibility of custom Clash templates.

## Features

-   **Dynamic Subscription Generation**: Converts a text file of proxy links into a single, accessible online subscription.
-   **UUID Protection**: Access to the subscription is secured by a UUID (Universally Unique Identifier), ensuring only authorized clients can fetch the content.
-   **Intelligent Client Detection**: Automatically serves Clash-compatible YAML configurations when accessed by Clash clients (e.g., User-Agent containing "Clash", "Mihomo", "Stash") or when a `flag=clash` query parameter is present. Otherwise, it provides a standard Base64-encoded list of links.
-   **Clash Template Merging**: Supports merging generated proxy nodes into a user-provided Clash `config.yaml` template, allowing for custom rules, proxy groups, DNS settings, and more. Robustly handles templates even if `proxies` or `proxy-groups` keys are missing or null.
-   **Multi-Protocol Support**: Parses and generates Clash configurations for a wide range of proxy protocols:
    -   **VLESS**: Supports Reality, gRPC, WebSocket transports.
    -   **VMess**: Supports WebSocket transport.
    -   **Hysteria2**: Supports obfuscation (obfs) and ALPN.
    -   **Trojan**: Supports Reality.
    -   **Shadowsocks (SS)**: Supports `method:password@server:port` format (both plain and base64 encoded).
    -   **TUIC**: Supports various parameters like congestion control, ALPN, SNI.

## Installation

To build `txt2sub`, you need to have [Rust](https://www.rust-lang.org/tools/install) installed.

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/txt2sub.git # Replace with your actual repo URL
    cd txt2sub
    ```
2.  **Build the project:**
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/txt2sub`.

## Usage

`txt2sub` runs as a local web server.

### Command-Line Arguments

```
Usage: txt2sub [OPTIONS]

Options:
  -f, --file <FILE>        Path to the text file containing subscription links
  -p, --port <PORT>        Port to listen on [default: 3000]
  -u, --uuid <UUID>        Custom UUID for the subscription URL. If not provided, a random one will be generated.
  -t, --template <TEMPLATE>  Path to the Clash config template (optional)
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version information
```

### Example

1.  **Prepare your subscription links file (`my_subs.txt`):**
    Create a text file with one proxy link per line. Comments starting with `#` or `//` will be ignored.

    ```
    # My V2Ray Nodes
    vless://bb7ff52a-xxxx-xxxx-xxxx-xxxx@1.2.3.4:443?security=reality&sni=example.com#VLESS_Reality

    vmess://eyJ2IjoiMiIsInBzIjoiVm1lc3NfV1MiLCJhZGQiOiI1LjYuNy44IiwicG9ydCI6IjgwODAiLCJpZCI6ImJiN2ZmNTJhLxxxx-xxxx-xxxx-xxxxIiwiYWlkIjoiMCIsIm5ldCI6IndzIiwidHlwZSI6Im5vbmUiLCJob3N0IjoiIiIsInBhdGgiOiIvdnciLCJ0bHMiOiIifQ==

    hy2://my-password@9.10.11.12:8443?sni=hy2.example.com&obfs=salamander&obfs-password=my-obfs-pass#Hysteria2_Node

    trojan://password@10.0.0.1:443?security=tls&sni=trojan.example.com#Trojan_TLS

    ss://YWVzLTI1Ni1nY206cGFzc3dvcmRAc2VydmVyLmNvbTo4MDgw#Shadowsocks_Example

    tuic://uuid:password@11.0.0.1:443?congestion_control=bbr&alpn=h3&sni=tuic.example.com#TUIC_Node
    ```

2.  **Prepare your Clash template file (optional, `clash_template.yaml`):**
    If you want to use a custom Clash configuration as a base, create a YAML file. `txt2sub` will inject generated proxies into the `proxies` list and into a proxy group named `PROXY`.

    ```yaml
    port: 7890
    socks-port: 7891
    allow-lan: true
    mode: Rule
    log-level: info
    external-controller: :9090

    dns:
      enable: true
      listen: 0.0.0.0:53
      enhanced-mode: fake-ip
      nameserver:
        - 114.114.114.114
        - 8.8.8.8

    proxies:
      # Existing static nodes can be here.
      - name: "DIRECT"
        type: direct

    proxy-groups:
      - name: PROXY # Generated nodes will be added here
        type: select
        proxies:
          - DIRECT # Keep existing proxies in the group
      - name: Auto-Select
        type: url-test
        url: http://www.gstatic.com/generate_204
        interval: 300
        proxies:
          - PROXY
      - name: Final
        type: select
        proxies:
          - Auto-Select
          - DIRECT

    rules:
      - DOMAIN-SUFFIX,google.com,Final
      - GEOIP,CN,DIRECT
      - MATCH,Final
    ```

3.  **Run the server:**

    ```bash
    ./target/release/txt2sub -f my_subs.txt -t clash_template.yaml -u my-secret-token -p 8080
    ```

    The server will start and print the generated subscription links:

    ```
    Server running on http://0.0.0.0:8080/sub?token=my-secret-token
    Subscription link: http://127.0.0.1:8080/sub?token=my-secret-token
    ```

### Accessing the Subscription

Use the printed "Subscription link" in your client. The server intelligently determines the output format:

-   **Standard Base64 List** (for v2rayN, Shadowrocket, browsers):
    Access the URL directly: `http://127.0.0.1:8080/sub?token=my-secret-token`
    (The server will detect non-Clash User-Agents and return `text/plain` with Base64 content).

-   **Clash YAML Configuration** (for Clash, Mihomo, Stash):
    Access the URL. The server will detect Clash User-Agents and return `text/yaml` with the merged configuration:
    `http://127.0.0.1:8080/sub?token=my-secret-token`
    Alternatively, you can force Clash YAML output by adding `&flag=clash` to the URL:
    `http://127.0.0.1:8080/sub?token=my-secret-token&flag=clash`

### Stopping the Server

To stop the server, find its process ID (PID) and terminate it. If you ran it in the background (`&`), you can use:

```bash
kill <SERVER_PID>
```
(Replace `<SERVER_PID>` with the PID printed when you started the server.)

## Contributing

Feel free to open issues or pull requests.
