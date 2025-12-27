use clap::Parser;
use colored::*;
use futures_util::TryStreamExt;
use reqwest::Client;
use std::io::{self, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use warp::Filter;
use emissary_cli::cli::{
    Arguments, HttpProxyOptions, MetricsOptions, PortForwardingOptions, ReseedOptions,
    RouterUiOptions, SocksProxyOptions, TransitOptions, TunnelOptions,
};

// --- CLI ARGUMENTS STRUCT ---
#[derive(Parser, Debug)]
#[command(name = "i2a")]
#[command(author = "BlackTechX011")]
#[command(version = "1.0")]
#[command(about = "I2P to API Bridge", long_about = None)]
struct Args {
    /// The Target I2P URL (e.g., http://i2p-projekt.i2p)
    #[arg(short, long, default_value = "http://i2p-projekt.i2p")]
    target: String,

    /// Local port to host the API/Proxy on
    #[arg(short, long, default_value_t = 8790)]
    port: u16,

    /// The upstream I2P HTTP Proxy port (Emissary default: 4444)
    #[arg(long, default_value_t = 4444)]
    upstream: u16,
}

#[tokio::main]
async fn main() {
    // 1. Parse Arguments
    let args = Args::parse();

    // 2. Print Banner
    print_banner();

    println!(
        "{} {} -> {}",
        "[CONFIG]".bold().blue(),
        "Target".yellow(),
        args.target.cyan()
    );
    println!(
        "{} {} -> 127.0.0.1:{}",
        "[CONFIG]".bold().blue(),
        "Local API".yellow(),
        args.port.to_string().cyan()
    );

    // 3. Launch Emissary (Embedded)
    launch_embedded_router(args.upstream);

    // 4. Wait for Upstream Proxy
    if !wait_for_upstream(args.upstream) {
        println!(
            "\n{}",
            "[FATAL] Could not connect to I2P Router.".red().bold()
        );
        return;
    }

    // 5. Build Client
    let proxy_url = format!("http://127.0.0.1:{}", args.upstream);
    let client = Client::builder()
        .proxy(reqwest::Proxy::http(&proxy_url).expect("Invalid proxy URL"))
        .build()
        .expect("Failed to build HTTP client");

    // 6. Setup Warp Server
    let proxy_route = warp::path::full()
        .and(warp::method())
        .and(warp::header::headers_cloned())
        .and(warp::body::stream())
        .and_then(move |path: warp::path::FullPath, method, headers, body| {
            let client = client.clone();
            let target = args.target.clone();
            async move { handle_request(client, target, path, method, headers, body).await }
        });

    println!(
        "\n{} Bridge is active. Access your API at:",
        "[SUCCESS]".bold().green()
    );
    println!("      {}", format!("http://127.0.0.1:{}", args.port).underline().white());
    println!("{}", "Press CTRL+C to stop.".dimmed());

    warp::serve(proxy_route)
        .run(([127, 0, 0, 1], args.port))
        .await;
}

fn print_banner() {
    let art = r#"
   _  _____           
  (_)/ __  \   __ _   
  | |`' / /'  / _` |  
  | |  / /   | (_| |  
  |_|./ /___  \__,_|  
     \_____/          
    "#;
    println!("{}", art.magenta().bold());
    println!("  I2P to API Bridge | v1.0");
    println!("  --------------------------\n");
}

fn launch_embedded_router(port: u16) {
    print!("{} Starting embedded I2P Router...", "[INIT]".bold().blue());
    io::stdout().flush().unwrap();

    // Construct Emissary Arguments manually
    let emissary_args = Arguments {
        base_path: None,
        log: None, // Use default logging
        floodfill: None,
        allow_local: None,
        caps: None,
        net_id: None,
        overwrite_config: None,
        tunnel: TunnelOptions {
            exploratory_inbound_len: None,
            exploratory_inbound_count: None,
            exploratory_outbound_len: None,
            exploratory_outbound_count: None,
            insecure_tunnels: None,
        },
        reseed: ReseedOptions {
            reseed_hosts: None,
            disable_reseed: None,
            reseed_threshold: None,
            force_reseed: None,
            disable_force_ipv4: None,
        },
        metrics: MetricsOptions {
            metrics_server_port: None,
            disable_metrics: None,
        },
        http_proxy: HttpProxyOptions {
            http_proxy_port: Some(port),
            http_proxy_host: None,
            http_outproxy: None,
        },
        socks_proxy: SocksProxyOptions {
            socks_proxy_port: None,
            socks_proxy_host: None,
        },
        transit: TransitOptions {
            max_transit_tunnels: None,
            disable_transit_tunnels: None,
        },
        port_forwarding: PortForwardingOptions {
            disable_upnp: None,
            disable_nat_pmp: None,
            upnp_name: None,
        },
        router_ui: RouterUiOptions {
            disable_ui: Some(true), // Disable UI by default for embedded usage to save resources
            refresh_interval: None,
            theme: None,
            web_ui_port: None,
        },
        command: None,
    };

    // Spawn Emissary in a separate task
    tokio::spawn(async move {
        if let Err(e) = emissary_cli::run(emissary_args).await {
            eprintln!("Emissary Router Error: {:?}", e);
        }
    });

    println!(" {}", "Background process started.".green());
}

fn wait_for_upstream(port: u16) -> bool {
    print!("{} Connecting to upstream (Port {})...", "[NET]".bold().blue(), port);
    io::stdout().flush().unwrap();

    // Increased timeout for router startup
    for _ in 0..60 {
        if TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
            println!(" {}", "Connected.".green());
            return true;
        }
        thread::sleep(Duration::from_secs(1));
        print!(".");
        io::stdout().flush().unwrap();
    }
    println!(" {}", "Timeout.".red());
    false
}

async fn handle_request(
    client: Client,
    base_url: String,
    path: warp::path::FullPath,
    method: warp::http::Method,
    _headers: warp::http::HeaderMap,
    body: impl futures_util::Stream<Item = Result<impl bytes::Buf, warp::Error>> + Send + Sync + 'static,
) -> Result<impl warp::Reply, warp::Rejection> {
    
    // Construct target URL
    let url = format!("{}{}", base_url, path.as_str());

    // Transform stream (Buf -> Bytes)
    let reqwest_body_stream = body.map_ok(|mut buf| {
        buf.copy_to_bytes(buf.remaining())
    }).map_err(|e| {
        Box::new(e) as Box<dyn std::error::Error + Send + Sync>
    });

    let req_body = reqwest::Body::wrap_stream(reqwest_body_stream);

    let resp = client.request(method, &url).body(req_body).send().await;

    match resp {
        Ok(response) => {
            let status = response.status();
            let body = response.bytes_stream();
            Ok(warp::reply::with_status(
                warp::reply::html(warp::hyper::Body::wrap_stream(body)),
                status,
            ))
        }
        Err(_) => Ok(warp::reply::with_status(
            warp::reply::html("<h1>i2a Error</h1><p>Upstream I2P connection failed.</p>".into()),
            warp::http::StatusCode::BAD_GATEWAY,
        )),
    }
}