mod analysis;
mod api;
mod auth;
mod cli;
mod config;
mod diagnostics;
mod domain;
mod errors;
mod ingestion;
mod observability;
mod runtime;
mod update;

use std::net::SocketAddr;

use api::router::build_router;
use config::{service, store::ConfigStore};
use tracing::info;

#[tokio::main]
async fn main() {
    observability::init();
    let args: Vec<String> = std::env::args().collect();
    let config = ConfigStore::default();
    match args.get(1).map(String::as_str) {
        Some("version") => {
            println!("glucose-dashboard {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Some("doctor") => {
            for check in diagnostics::checks::run(&config, None) {
                println!("{} {}", if check.ok { "✓" } else { "✗" }, check.name);
            }
            return;
        }
        Some("config") => {
            let sheet_id = args.get(2).cloned().unwrap_or_default();
            let fixture = args.get(3).cloned();
            match service::configure(&config, sheet_id, "Sheet1".into(), fixture, None, None) {
                Ok(_) => println!("設定完成，請執行 glucose-dashboard 啟動 Dashboard。"),
                Err(error) => {
                    eprintln!("設定失敗：{error}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("update") => {
            println!(
                "目前版本 {}，更新服務尚未連接 release 來源。",
                env!("CARGO_PKG_VERSION")
            );
            return;
        }
        _ => {}
    }
    let app = build_router(config);
    let address: SocketAddr = ([127, 0, 0, 1], 3000).into();
    info!(%address, "Glucose Dashboard local service starting");
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("bind local service");
    axum::serve(listener, app)
        .await
        .expect("serve local service");
}
