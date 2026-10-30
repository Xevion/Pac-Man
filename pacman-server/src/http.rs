//! Outbound HTTP client construction.
//!
//! `reqwest` is built without a bundled rustls provider, so building a client before a
//! process-wide provider exists panics. Every client goes through [`client_builder`].

use std::sync::Once;

/// Installs aws-lc as the process-wide rustls crypto provider. Later calls are no-ops.
pub fn install_crypto_provider() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        // Err only means a provider was already installed, which serves equally well.
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

/// A `reqwest` client builder that is safe to `build()`.
pub fn client_builder() -> reqwest::ClientBuilder {
    install_crypto_provider();
    reqwest::Client::builder()
}
