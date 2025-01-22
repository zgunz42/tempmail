use tempmail::storage::email_storage::EmailStorage;
use tempmail::server::imap_server::run_imap_server;
use tempmail::server::smtp_server::run_smtp_server;
use tempmail::utils::utils::generate_email;

#[tokio::main]
async fn main() {
    let storage = EmailStorage::new();
    let smtp = run_smtp_server(storage.clone());
    let imap = run_imap_server(storage.clone());
    println!("Temp Email: {}", generate_email());
    tokio::join!(smtp, imap);
}