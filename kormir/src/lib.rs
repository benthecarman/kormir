use bitcoin::secp256k1::{All, Secp256k1, SecretKey};
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey};
use std::str::FromStr;

// first key for taproot address
const SIGNING_KEY_PATH: &str = "m/86'/0'/0'/0/0";

const NONCE_KEY_PATH: &str = "m/585'/0'/0'";

#[derive(Debug, Clone)]
pub struct Oracle {
    signing_key: SecretKey,
    nonce_xpriv: ExtendedPrivKey,
    secp: Secp256k1<All>,
}

impl Oracle {
    pub fn new(signing_key: SecretKey, nonce_xpriv: ExtendedPrivKey) -> Self {
        let secp = Secp256k1::new();
        Oracle {
            signing_key,
            nonce_xpriv,
            secp,
        }
    }

    pub fn from_xpriv(xpriv: ExtendedPrivKey) -> anyhow::Result<Self> {
        let secp = Secp256k1::new();

        let signing_key = xpriv
            .derive_priv(&secp, &DerivationPath::from_str(SIGNING_KEY_PATH)?)?
            .private_key;
        let nonce_xpriv = xpriv.derive_priv(&secp, &DerivationPath::from_str(NONCE_KEY_PATH)?)?;

        Ok(Self {
            signing_key,
            nonce_xpriv,
            secp,
        })
    }
}
