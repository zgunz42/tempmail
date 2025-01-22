use tempmail::storage::email_storage::EmailStorage;
use tempmail::server::imap_server::run_imap_server;
use tempmail::server::smtp_server::run_smtp_server;
use tempmail::utils::utils::generate_email;

#[tokio::main]
async fn main() {
    let storage = EmailStorage::new();
    let dkim = generate_dkim_keys();
    let rate_limiter = Arc::new(RateLimiter::new(100));
    let smtp = run_smtp_server(storage.clone(), Arc::new(dkim), rate_limiter.clone());
    let imap = run_imap_server(storage.clone());
    println!("Temp Email: {}", generate_email());
    tokio::join!(smtp, imap);
}