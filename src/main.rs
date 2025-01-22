use tokio::{
    net::TcpListener,
    io::{AsyncReadExt, AsyncWriteExt},
};
use std::error::Error;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct Email {
    from: String,
    to: String,
    subject: String,
    body: String,
    timestamp: chrono::DateTime<Utc>,
}

struct MailServer {
    mailboxes: Arc<Mutex<HashMap<String, Vec<Email>>>>,
}

impl MailServer {
    fn new() -> Self {
        MailServer {
            mailboxes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn handle_smtp(&self, mut stream: tokio::net::TcpStream) -> Result<(), Box<dyn Error>> {
        // Send greeting
        stream.write_all(b"220 localhost SMTP server ready\r\n").await?;

        let mut buffer = [0; 1024];
        let mut current_email = Email {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: String::new(),
            timestamp: Utc::now(),
        };
        
        let mut in_data_mode = false;
        let mut data_buffer = String::new();

        loop {
            let n = stream.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            let cmd = String::from_utf8_lossy(&buffer[..n]);
            let cmd = cmd.trim();

            if in_data_mode {
                if cmd == "." {
                    in_data_mode = false;
                    current_email.body = data_buffer.clone();
                    
                    // Store the email
                    let mut mailboxes = self.mailboxes.lock().await;
                    mailboxes
                        .entry(current_email.to.clone())
                        .or_insert_with(Vec::new)
                        .push(current_email.clone());

                    stream.write_all(b"250 Ok: message accepted\r\n").await?;
                    data_buffer.clear();
                } else {
                    data_buffer.push_str(cmd);
                    data_buffer.push_str("\r\n");
                }
                continue;
            }

            match cmd.split_whitespace().next() {
                Some("HELO") | Some("EHLO") => {
                    stream.write_all(b"250 localhost\r\n").await?;
                }
                Some("MAIL") => {
                    if let Some(from) = cmd.split(':').nth(1) {
                        current_email.from = from.trim().to_string();
                        stream.write_all(b"250 Ok\r\n").await?;
                    }
                }
                Some("RCPT") => {
                    if let Some(to) = cmd.split(':').nth(1) {
                        current_email.to = to.trim().to_string();
                        stream.write_all(b"250 Ok\r\n").await?;
                    }
                }
                Some("DATA") => {
                    stream.write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n").await?;
                    in_data_mode = true;
                }
                Some("QUIT") => {
                    stream.write_all(b"221 Bye\r\n").await?;
                    break;
                }
                _ => {
                    stream.write_all(b"500 Unknown command\r\n").await?;
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:2525").await?;
    println!("Server listening on port 2525");

    let mail_server = Arc::new(MailServer::new());

    loop {
        let (stream, _) = listener.accept().await?;
        let mail_server = Arc::clone(&mail_server);
        
        tokio::spawn(async move {
            if let Err(e) = mail_server.handle_smtp(stream).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}