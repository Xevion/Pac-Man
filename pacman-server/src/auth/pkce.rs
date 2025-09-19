use dashmap::DashMap;
use oauth2::PkceCodeChallenge;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tracing::{trace, warn};

#[derive(Debug, Clone)]
pub struct PkceRecord {
    pub verifier: String,
    pub created_at: Instant,
}

#[derive(Default)]
pub struct PkceManager {
    pkce: DashMap<String, PkceRecord>,
    last_purge_at_secs: AtomicU32,
    pkce_additions: AtomicU32,
}

impl PkceManager {
    pub fn generate_challenge(&self) -> (PkceCodeChallenge, String) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        trace!("PKCE challenge generated");
        (pkce_challenge, pkce_verifier.secret().to_string())
    }

    pub fn store_verifier(&self, state: &str, verifier: String) {
        self.pkce.insert(
            state.to_string(),
            PkceRecord {
                verifier,
                created_at: Instant::now(),
            },
        );
        self.pkce_additions.fetch_add(1, Ordering::Relaxed);
        self.maybe_purge_stale_entries();
        trace!(state = state, "Stored PKCE verifier for state");
    }

    pub fn take_verifier(&self, state: &str) -> Option<String> {
        let Some(record) = self.pkce.remove(state).map(|e| e.1) else {
            trace!(state = state, "PKCE verifier not found for state");
            return None;
        };

        // Verify PKCE TTL
        if Instant::now().duration_since(record.created_at) > Duration::from_secs(5 * 60) {
            warn!(state = state, "PKCE verifier expired for state");
            return None;
        }

        trace!(state = state, "PKCE verifier retrieved for state");
        Some(record.verifier)
    }

    fn maybe_purge_stale_entries(&self) {
        // Purge when at least 5 minutes passed or more than 128 additions occurred
        const PURGE_INTERVAL_SECS: u32 = 5 * 60;
        const ADDITIONS_THRESHOLD: u32 = 128;

        let now_secs = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_secs() as u32,
            Err(_) => return,
        };

        let last = self.last_purge_at_secs.load(Ordering::Relaxed);
        let additions = self.pkce_additions.load(Ordering::Relaxed);
        if additions < ADDITIONS_THRESHOLD && now_secs.saturating_sub(last) < PURGE_INTERVAL_SECS {
            return;
        }

        const PKCE_TTL: Duration = Duration::from_secs(5 * 60);
        let now_inst = Instant::now();
        for entry in self.pkce.iter() {
            if now_inst.duration_since(entry.value().created_at) > PKCE_TTL {
                self.pkce.remove(entry.key());
            }
        }

        // Reset counters after purge
        self.pkce_additions.store(0, Ordering::Relaxed);
        self.last_purge_at_secs.store(now_secs, Ordering::Relaxed);
    }
}
