// 临时探针：验证 CLI 能否直接调用 GUI 进程的 Tauri IPC 接口。
// 用法：cargo run --bin hs -- ipc-probe
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub fn probe(port: u16) -> Result<(), String> {
    let mut s = TcpStream::connect(("127.0.0.1", port)).map_err(|e| format!("connect: {e}"))?;
    s.set_read_timeout(Some(Duration::from_secs(3))).ok();

    // WebSocket 握手，带 Tauri 的 Origin 头
    let req = format!(
        "GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\
         Origin: http://tauri.localhost\r\n\r\n"
    );
    s.write_all(req.as_bytes()).map_err(|e| format!("write handshake: {e}"))?;

    let mut buf = [0u8; 2048];
    let n = s.read(&mut buf).map_err(|e| format!("read handshake: {e}"))?;
    let head = String::from_utf8_lossy(&buf[..n]).to_string();
    println!("--- handshake response ---\n{}", head.lines().next().unwrap_or("(empty)"));

    // 发一帧文本：{"id":1,"cmd":"get_current_version"}
    let payload = br#"{"id":1,"cmd":"get_current_version"}"#;
    let mut frame = vec![0x81u8];
    let len = payload.len();
    if len < 126 {
        frame.push(0x80 | len as u8);
    } else {
        frame.push(0x80 | 126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    }
    let mask = [0x12u8, 0x34, 0x56, 0x78];
    frame.extend_from_slice(&mask);
    for (i, b) in payload.iter().enumerate() {
        frame.push(b ^ mask[i % 4]);
    }
    s.write_all(&frame).map_err(|e| format!("write frame: {e}"))?;

    let n = s.read(&mut buf).map_err(|e| format!("read reply: {e}"))?;
    let reply = String::from_utf8_lossy(&buf[..n]).to_string();
    println!("--- reply ({} bytes) ---\n{:?}", n, reply.chars().take(400).collect::<String>());
    Ok(())
}
