use std::{fs::File, path::Path, str::FromStr, sync::Arc};

use ethers::{
    abi::{self, token::Tokenizer},
    prelude::{k256, SignerMiddleware},
    providers::{Http, Provider},
    signers::{self, LocalWallet, Wallet},
    types::{Bytes, U256},
};
pub mod testing;
pub mod testnet;
pub use sunscreen::{types::bfv::*, Ciphertext, FheRuntime, PrivateKey, PublicKey};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("EthABI error: {0}")]
    Abi(#[from] abi::Error),
    #[error("Bincode conversion error: {0}")]
    Conversion(#[from] bincode::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Wallet error: {0}")]
    Wallet(#[from] signers::WalletError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenient `Result` wrapper for [`Error`]s.
pub type Result<T> = std::result::Result<T, Error>;

/// A convenient alias for a signing-capable client over an HTTP provider.
pub type SignedMiddleware = SignerMiddleware<Arc<Provider<Http>>, Wallet<k256::ecdsa::SigningKey>>;

/// Our FHE types are encoded into [`Bytes`] in solidity contracts. This trait allows you to convert
/// the bytes to and from the FHE types.
// TODO maybe will want a bfv fractional impl?
pub trait AsBytes: Sized {
    /// Convert from bytes into an FHE type. This is useful for contract return values.
    fn from_bytes(bytes: &Bytes) -> Result<Self>;
    /// Convert from an FHE type into bytes. This is useful for supplying contract method
    /// arguments.
    fn as_bytes(&self) -> Result<Bytes>;
}

/// When generating keypairs, you'll need to save your private key (and it is often convenient to
/// have your public key saved locally as well). For a CLI application, the natural way to store
/// keys is in the filesystem.
pub trait AsFile: Sized {
    /// Read FHE type from a file.
    fn read<P: AsRef<Path>>(path: P) -> Result<Self>;
    /// Write FHE type to a file.
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}

/// Convert between ethers and sunscreen numeric types. This should be a bijection, hence
/// the associated type. (Note: implicit assumption of 64-bit architecture!)
pub trait AsNum {
    type Output;
    fn to(&self) -> Self::Output;
}

impl AsNum for Unsigned256 {
    type Output = U256;
    fn to(&self) -> Self::Output {
        U256(crypto_bigint::U256::from(*self).to_words())
    }
}
impl AsNum for U256 {
    type Output = Unsigned256;
    fn to(&self) -> Self::Output {
        Unsigned256::from(crypto_bigint::U256::from_words(self.0))
    }
}

/// Parses an ether value from a string.
///
/// The amount can be tagged with a unit, e.g. "1ether". If the string represents an untagged
/// amount (e.g. "100") then it is interpreted as wei.
///
/// This function can be useful as a clap `value_parser`.
pub fn parse_ether_value(value: &str) -> Result<U256> {
    Ok(if value.starts_with("0x") {
        U256::from_str(value).map_err(anyhow::Error::new)?
    } else {
        U256::from(abi::token::LenientTokenizer::tokenize_uint(value)?)
    })
}

macro_rules! impl_bytes_via_bincode {
    ($($ty:ty),+) => {
        $(
            impl AsBytes for $ty {
                fn from_bytes(bytes: &Bytes) -> Result<Self> {
                    let val = bincode::deserialize(bytes)?;
                    Ok(val)
                }

                fn as_bytes(&self) -> Result<Bytes> {
                    let bytes = bincode::serialize(self)?.into();
                    Ok(bytes)
                }
            }
        )+
    };
}

impl_bytes_via_bincode! {
    PublicKey, PrivateKey, Ciphertext
}

macro_rules! impl_file_via_bincode {
    ($($ty:ty),+) => {
        $(
            impl AsFile for $ty {
                fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
                    let mut file = File::open(path)?;
                    let val = bincode::deserialize_from(&mut file)?;
                    Ok(val)
                }

                fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
                    let mut file = File::create(path)?;
                    bincode::serialize_into(&mut file, &self)?;
                    Ok(())
                }
            }
        )+
    };
}

impl_file_via_bincode! {
    PublicKey, PrivateKey, Ciphertext
}

impl AsFile for LocalWallet {
    fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let wallet = LocalWallet::from_bytes(&bytes)?;
        Ok(wallet)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let bytes = self.signer().to_bytes();
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
