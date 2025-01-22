use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::storage::email_storage::EmailStorage;

pub async fn handle_imap_client(mut stream: TcpStream, storage: EmailStorage) {
  let _ = stream.write_all(b"* OK IMAP ready\r\n").await;
  let mut buffer = [0; 1024];
  let mut authenticated = false;

  loop {
      let n = match stream.read(&mut buffer).await {
          Ok(n) if n == 0 => break,
          Ok(n) => n,
          Err(_) => break,
      };

      let input = String::from_utf8_lossy(&buffer[..n]);
      let parts: Vec<&str> = input.trim().split_whitespace().collect();
      if parts.len() < 2 { continue; }

      let tag = parts[0];
      let command = parts[1].to_uppercase();

      match command.as_str() {
          "LOGIN" if parts.len() >= 3 => {
              authenticated = true;
              let _ = stream.write_all(format!("{} OK Login successful\r\n", tag).as_bytes()).await;
          }
          "SELECT" => {
              let emails = storage.get_emails(parts[2]).unwrap_or_default();
              let response = format!("* FLAGS (\\Seen)\r\n* {} EXISTS\r\n* OK [UIDVALIDITY 1]\r\n{} OK Select completed\r\n", emails.len(), tag);
              let _ = stream.write_all(response.as_bytes()).await;
          }
          "FETCH" => {
              let email_id = parts[2].parse::<usize>().unwrap();
              if let Some(emails) = storage.get_emails(parts[4]) {
                  if let Some(email) = emails.get(email_id - 1) {
                      let response = format!("* {} FETCH (BODY[] {{{}}})\r\n{}\r\n{} OK Fetch completed\r\n", 
                          email_id, email.len(), String::from_utf8_lossy(email), tag);
                      let _ = stream.write_all(response.as_bytes()).await;
                  }
              }
          }
          "LOGOUT" => { let _ = stream.write_all(b"* BYE\r\n").await; break; }
          _ => { let _ = stream.write_all(format!("{} BAD Unknown command\r\n", tag).as_bytes()).await; }
      }
  }
}

pub async fn run_imap_server(storage: EmailStorage) {
  let listener = TcpListener::bind("127.0.0.1:2143").await.unwrap();
  println!("{}:{}", listener.local_addr().unwrap().ip(), listener.local_addr().unwrap().port());
  loop {
      let (stream, _) = listener.accept().await.unwrap();
      let storage_clone = storage.clone();
      tokio::spawn(async move {
          handle_imap_client(stream, storage_clone).await;
      });
  }
}