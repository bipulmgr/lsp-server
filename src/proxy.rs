use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::accept_async;
use tokio::net::TcpStream;
use futures_util::{StreamExt, SinkExt};
use crate::config;
use crate::message_handler::*;
use crate::process_manager::ProcessManager;

pub async fn handle_connection(stream: TcpStream, path: String, pm: ProcessManager) {
    let mut query = path.split('?').nth(1).unwrap_or("").split('&')
        .map(|s| s.split('=').collect::<Vec<_>>())
        .filter(|v| v.len() == 2)
        .map(|v| (v[0], v[1]))
        .collect::<std::collections::HashMap<_, _>>();

    let server_name = query.get("name").map(|v| *v);

    let ws_stream = accept_async(stream).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    if server_name.is_none() {
        let servers = config::list_servers().keys().map(|&k| k).collect::<Vec<_>>();
        let response = format_server_list_response(servers);
        let _ = write.send(Message::Text(response)).await;
        return;
    }

    let server_name = server_name.unwrap();
    let server_config = config::get_server(server_name);

    if server_config.is_none() {
        let response = format_error_response("Unknown server");
        let _ = write.send(Message::Text(response)).await;
        return;
    }

    let (mut stdin, stdout) = pm.start_process(server_name, server_config.unwrap()).await;
    let mut stdout = BufReader::new(stdout);

    let ws_to_lsp = async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let data = format_jsonrpc_message(&text);
                stdin.write_all(&data).await.unwrap();
                stdin.flush().await.unwrap();
            }
        }
    };

    let lsp_to_ws = async {
        let mut buffer : Vec<u8> = Vec::new();

        loop {
            buffer.clear();
            let mut header_buf : Vec<u8> = Vec::new();

            // Read headers until \r\n\r\n (LSP standard)
            loop {
                let mut byte = [0u8; 1];
                if stdout.read_exact(&mut byte).await.is_err() {
                    return;
                }
                header_buf.push(byte[0]);

                if header_buf.ends_with(b"\r\n\r\n") {
                    break;
                }
            }

            let headers = String::from_utf8_lossy(&header_buf);
            let mut content_length = None;

            for line in headers.lines() {
                if let Some(len) = line.strip_prefix("Content-Length:") {
                    content_length = Some(len.trim().parse::<usize>().unwrap());
                }
            }

            let content_length = match content_length {
                Some(len) => len,
                None => continue,
            };

            // Read content
            let mut content_buf = vec![0u8; content_length];
            if stdout.read_exact(&mut content_buf).await.is_err() {
                return;
            }

            if let Some(parsed) = parse_jsonrpc_message(&content_buf) {
                let _ = write.send(Message::Text(parsed)).await;
            }
        }
    };

    tokio::join!(ws_to_lsp, lsp_to_ws);
}
