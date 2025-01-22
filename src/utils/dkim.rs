use dkim::{DkimPrivateKey, DkimSigner, SigningAlgorithm};
use rsa::{RsaPrivateKey, pkcs8::DecodePrivateKey};

pub struct DkimConfig {
    private_key: RsaPrivateKey,
    selector: String,
    domain: String,
}

impl DkimConfig {
    fn new(private_key: RsaPrivateKey, selector: &str, domain: &str) -> Self {
        Self {
            private_key,
            selector: selector.to_string(),
            domain: domain.to_string(),
        }
    }

    pub async fn sign_email(&self, raw_email: &[u8]) -> Result<Vec<u8>, dkim::Error> {
        let signer = DkimSigner::new(
            self.private_key.clone(),
            SigningAlgorithm::RsaSha256,
            self.selector.clone(),
            self.domain.clone(),
        )?;
        
        signer.sign(raw_email).await
    }
}

// Generate DKIM keys (run once)
pub fn generate_dkim_keys() -> (String, String) {
    let private_key = RsaPrivateKey::new(&mut rand::thread_rng(), 2048).unwrap();
    let public_key = private_key.to_public_key();
    
    let pem_private = private_key.to_pkcs8_pem().unwrap();
    let dns_record = format!(
        "v=DKIM1; k=rsa; p={}",
        base64::encode(public_key.to_public_key_der().unwrap())
    );
    
    (pem_private, dns_record)
}