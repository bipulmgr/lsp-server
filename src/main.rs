mod config;
mod message_handler;
mod process_manager;
mod proxy;

use process_manager::ProcessManager;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:9997";
    println!("LSP Proxy running at ws://{}", addr);

    let pm = ProcessManager::new();
    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (stream, _addr) = listener.accept().await.unwrap();
        let path = "/?name=python".to_string();
        let pm_clone = pm.clone();

        tokio::spawn(async move {
            proxy::handle_connection(stream, path, pm_clone).await;
        });
    }
}
