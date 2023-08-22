//! This module offers functionality for interacting with testnets by Sunscreen.

use std::sync::Arc;

use ethers::{
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
};

use super::SignedMiddleware;

/// This module offers functionality for interacting with Sunscreen's Parasol testnet.
pub mod parasol {
    pub use fhe_precompiles::testnet::one::*;
    /// The chain ID of Sunscreen's Parasol testnet.
    const CHAIN_ID: u64 = 574;
    /// The RPC URL of Sunscreen's Parasol testnet.
    const RPC_URL: &str = "https://rpc.sunscreen.tech/parasol";
    /// The faucet URL of Sunscreen's Parasol testnet.
    const FAUCET_URL: &str = "https://faucet.sunscreen.tech/";

    use super::TestnetProvider;

    /// A provider for Sunscreen's Parasol testnet.
    ///
    /// ```no_run
    /// use std::str::FromStr;
    /// use ethers::{
    ///     providers::Middleware,
    ///     signers::{LocalWallet, Signer},
    ///     types::{Address, TransactionRequest},
    /// };
    /// use sunscreen_web3::testnet::parasol::PARASOL;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Instantiate a local wallet
    /// let wallet = LocalWallet::from_str("some_private_key")?;
    /// // Make a client that can sign and submit transactions from the wallet to the Parasol network
    /// let client = PARASOL.client(wallet);
    /// // Send someone 100 gwei
    /// let address = "0x0".parse::<Address>()?;
    /// let tx = TransactionRequest::new().to(address).value(100);
    /// client.send_transaction(tx, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub const PARASOL: TestnetProvider = TestnetProvider {
        rpc_url: RPC_URL,
        chain_id: CHAIN_ID,
        faucet_url: FAUCET_URL,
    };
}

/// A testnet specification which can generate [`Provider`]s and [`SignedMiddleware`].
pub struct TestnetProvider {
    pub rpc_url: &'static str,
    pub chain_id: u64,
    pub faucet_url: &'static str,
}

impl TestnetProvider {
    /// Construct a [`Provider<Http>`] for the testnet.
    pub fn provider(&self) -> Provider<Http> {
        Provider::try_from(self.rpc_url).unwrap()
    }

    /// Construct a client with signable middleware for this testnet. This is useful when
    /// instantiating a [`ethers::contract::Contract`], which underlies the Solidity to Rust
    /// contract bindings.
    pub fn client(&self, wallet: LocalWallet) -> Arc<SignedMiddleware> {
        let provider = Arc::new(self.provider());
        let middleware = SignedMiddleware::new(provider, wallet.with_chain_id(self.chain_id));
        Arc::new(middleware)
    }
}
