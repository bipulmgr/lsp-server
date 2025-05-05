use serde_json::json;

pub fn format_jsonrpc_message(message: &str) -> Vec<u8> {
    let content = message.as_bytes();
    let header = format!("Content-Length: {}\r\n\r\n", content.len());
    [header.as_bytes(), content].concat()
}

pub fn parse_jsonrpc_message(content: &[u8]) -> Option<String> {
    serde_json::from_slice::<serde_json::Value>(content)
        .ok()
        .map(|_| String::from_utf8_lossy(content).to_string())
}

pub fn format_server_list_response(servers: Vec<&str>) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 0,
        "result": {
            "servers": servers
        }
    }).to_string()
}

pub fn format_error_response(error_message: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": {
            "code": -32603,
            "message": error_message
        }
    }).to_string()
}
