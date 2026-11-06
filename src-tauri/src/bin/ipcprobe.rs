#[path = "../cli/probe.rs"]
mod probe;

fn main() {
    let port: u16 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    if port == 0 {
        eprintln!("usage: ipcprobe <port>");
        return;
    }
    if let Err(e) = probe::probe(port) {
        eprintln!("probe failed: {e}");
    }
}
