use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::storage::email_storage::EmailStorage;

pub async fn handle_smtp_client(mut stream: TcpStream, storage: EmailStorage) {
    let _ = stream.write_all(b"220 temp-mail SMTP ready\r\n").await;
    let mut buffer = [0; 1024];
    let mut recipients: Vec<String> = Vec::new();
    let mut data = Vec::new();
    let mut reading_data = false;

    loop {
        let n = match stream.read(&mut buffer).await {
            Ok(n) if n == 0 => break,
            Ok(n) => n,
            Err(_) => break,
        };

        let input = String::from_utf8_lossy(&buffer[..n]);
        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }

            if reading_data {
                if line == "." {
                    reading_data = false;
                    for recipient in &recipients {
                        storage.add_email(recipient.clone(), data.clone());
                    }
                    let _ = stream.write_all(b"250 OK\r\n").await;
                    data.clear();
                } else {
                    data.extend_from_slice(line.as_bytes());
                    data.push(b'\n');
                }
            } else {
                let cmd: Vec<&str> = line.split_whitespace().collect();
                match cmd[0].to_uppercase().as_str() {
                    "EHLO" | "HELO" => { let _ = stream.write_all(b"250-Hello\r\n250 OK\r\n").await; }
                    "MAIL" => { /* Parse sender if needed */ }
                    "RCPT" => {
                        if let Some(recipient) = cmd.get(1).and_then(|s| s.split(':').nth(1)) {
                            recipients.push(recipient.trim_matches('>').to_string());
                        }
                    }
                    "DATA" => { reading_data = true; let _ = stream.write_all(b"354 Start input\r\n").await; }
                    "QUIT" => { let _ = stream.write_all(b"221 Bye\r\n").await; return; }
                    _ => { let _ = stream.write_all(b"500 Unknown command\r\n").await; }
                }
            }
        }
    }
}

pub async fn run_smtp_server(storage: EmailStorage) {
    let listener = TcpListener::bind("127.0.0.1:2525").await.unwrap();
    println!("{}:{}", listener.local_addr().unwrap().ip(), listener.local_addr().unwrap().port());
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let storage_clone = storage.clone();
        tokio::spawn(async move {
            handle_smtp_client(stream, storage_clone).await;
        });
    }
}