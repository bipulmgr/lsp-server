use std::collections::HashMap;

pub fn list_servers() -> HashMap<&'static str, Vec<&'static str>> {
    let mut servers = HashMap::new();
    servers.insert("python", vec!["pylsp"]);
    servers.insert("typescript", vec!["typescript-language-server", "--stdio"]);
    servers.insert("rust", vec!["rust-analyzer"]);
    servers.insert("go", vec!["gopls"]);
    servers
}

pub fn get_server(name: &str) -> Option<Vec<&'static str>> {
    list_servers().get(name).cloned()
}
