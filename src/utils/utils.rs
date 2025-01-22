use rand::Rng;

pub fn generate_email() -> String {
    let mut rng = rand::thread_rng();
    let username: String = (0..10)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();
    format!("{}@example.com", username)
}