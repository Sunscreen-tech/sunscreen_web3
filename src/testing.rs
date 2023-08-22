//! This module offers functionality for testing contracts prior to deploying them on chain.
//!
//! As a prerequisite, you need to have Sunscreen's `anvil` fork installed:
//!
//! ```text
//! cargo install --git https://github.com/Sunscreen-tech/foundry --profile local anvil
//! ```
use std::{str::FromStr, sync::Arc};

use ethers::{
    prelude::Lazy,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    utils::{Anvil, AnvilInstance},
};

use super::SignedMiddleware;

/// A mnemonic for anvil to guarantee determinism. You must use this value to use the wallets for
/// [`ALICE`] and [`BOB`] below.
pub const ANVIL_MNEMONIC: &str =
    "gas monster ski craft below illegal discover limit dog bundle bus artefact";

/// A wallet for a test user, creatively named Alice. This user exists whenever Anvil is invoked with
/// [`ANVIL_MNEMONIC`].
///
/// Public address: 0xb5f27c716e44ffe48fd6622983c651355ad8c75a
pub static ALICE: Lazy<LocalWallet> = Lazy::new(|| {
    LocalWallet::from_str("0x1c0eb5244c165957525ef389fc14fac4424feaaefabf87c7e4e15bcc7b425e15")
        .unwrap()
});

/// A wallet for a test user, creatively named Bob. This user exists whenever Anvil is invoked with
/// [`ANVIL_MNEMONIC`].
///
/// Public address: 0x00d88e763c5764e69dd667fa8073d48022a4afef
pub static BOB: Lazy<LocalWallet> = Lazy::new(|| {
    LocalWallet::from_str("0x3b42a2df3c658b156b8240e1891723fab65ae0b97f9f5bba2abd5e240065baa1")
        .unwrap()
});

/// A simple way to construct and run a local node for development purposes.
pub struct Node {
    anvil: AnvilInstance,
}

impl Default for Node {
    fn default() -> Self {
        Self::spawn()
    }
}

impl Node {
    /// Spawn the node (i.e. launch the anvil subprocess) with default configuration.
    ///
    /// Note this configuration allows you to specify where the anvil executable lives via an
    /// environment variable `ANVIL_PATH`.
    pub fn spawn() -> Self {
        let anvil = std::env::var("ANVIL_PATH")
            .map(Anvil::at)
            .unwrap_or_else(|_| Anvil::new())
            .mnemonic(ANVIL_MNEMONIC)
            .args(["--gas-limit", "3000000000000000000"]);
        Self::spawn_from(anvil)
    }

    /// Spawn a node from the provided [`Anvil`]. Use this if you want to customize the way
    /// anvil is launched; otherwise just use [`Self::spawn()`].
    pub fn spawn_from(anvil: Anvil) -> Self {
        Self {
            anvil: anvil.spawn(),
        }
    }

    /// Get an http-based [`Provider`] from this anvil instance.
    pub fn provider(&self) -> Provider<Http> {
        Provider::<Http>::try_from(self.anvil.endpoint()).unwrap()
    }

    /// Construct a client with signable middleware for this node. This is useful when
    /// instantiating an [`ethers::contract::Contract`], which underlies the Solidity to Rust
    /// contract bindings.
    pub fn client(&self, wallet: LocalWallet) -> SignedMiddleware {
        SignedMiddleware::new(
            Arc::new(self.provider()),
            wallet.with_chain_id(self.anvil.chain_id()),
        )
    }
}

#[cfg(test)]
mod tests {
    use ethers::{providers::Middleware, signers::Signer, types::TransactionRequest};

    use super::*;

    #[tokio::test]
    async fn anvil_works() {
        let node = Node::default();
        let client = node.client(ALICE.clone());

        // craft the transaction
        let tx = TransactionRequest::new().to(BOB.address()).value(10000);

        // send it!
        let pending_tx = client.send_transaction(tx, None).await.unwrap();

        // get the mined tx
        let receipt = pending_tx.await.unwrap().unwrap();
        let _tx = client
            .get_transaction(receipt.transaction_hash)
            .await
            .unwrap();
    }
}
