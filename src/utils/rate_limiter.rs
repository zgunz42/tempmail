use governor::{clock::QuantaInstant, state::InMemoryState, NotUntil, Quota, RateLimiter as GovRateLimiter};
use std::num::NonZeroU32;

type IpAddress = String;

pub struct RateLimiter {
    smtp_limiter: GovRateLimiter<IpAddress, dashmap::DashMap<IpAddress, InMemoryState>, governor::clock::QuantaClock, governor::middleware::NoOpMiddleware<governor::clock::QuantaInstant>>,
    imap_limiter: GovRateLimiter<IpAddress, dashmap::DashMap<IpAddress, InMemoryState>, governor::clock::QuantaClock, governor::middleware::NoOpMiddleware<governor::clock::QuantaInstant>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        let smtp_quota = Quota::with_period(std::time::Duration::from_secs(60))
            .unwrap()
            .allow_burst(NonZeroU32::new(10).unwrap());
        
        let imap_quota = Quota::with_period(std::time::Duration::from_secs(30))
            .unwrap()
            .allow_burst(NonZeroU32::new(5).unwrap());
        let smtp_limiter = GovRateLimiter::keyed(smtp_quota.clone());
        let imap_limiter  = GovRateLimiter::keyed(imap_quota.clone());

        Self {
            smtp_limiter,
            imap_limiter,
        }
    }

    pub async fn check_smtp_limit(&self, ip: &IpAddress) -> Result<(), NotUntil<QuantaInstant>> {
        self.smtp_limiter.check_key(ip)
    }

    pub async fn check_imap_limit(&self, ip: &IpAddress) -> Result<(), NotUntil<QuantaInstant>> {
        self.imap_limiter.check_key(ip)
    }
}