use bitcoin::hashes::{sha256, Hash};
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{Parity, Scalar, Secp256k1, SecretKey, Signing};
use bitcoin::XOnlyPublicKey;

/// The tag "BIP0340/challenge"
const SCHNORR_TAG_BYTES: [u8; 64] = [
    123, 181, 45, 122, 159, 239, 88, 50, 62, 177, 191, 122, 64, 125, 179, 130, 210, 243, 242, 216,
    27, 177, 34, 79, 73, 254, 81, 143, 109, 72, 211, 124, 123, 181, 45, 122, 159, 239, 88, 50, 62,
    177, 191, 122, 64, 125, 179, 130, 210, 243, 242, 216, 27, 177, 34, 79, 73, 254, 81, 143, 109,
    72, 211, 124,
];

fn get_schnorr_key<S: Signing>(secp: &Secp256k1<S>, key: SecretKey) -> (XOnlyPublicKey, SecretKey) {
    let (xonly, parity) = key.x_only_public_key(secp);

    match parity {
        Parity::Odd => {
            let key = key.negate();
            let (xonly, _) = key.x_only_public_key(secp);
            (xonly, key)
        }
        Parity::Even => (xonly, key),
    }
}

// DO NOT TRUST
// I copied logic from here: https://github.com/bitcoin-s/bitcoin-s/blob/ae0962d7eda0a218caaa9ed2b5862d5a1b298be3/crypto/src/main/scala/org/bitcoins/crypto/CryptoRuntime.scala#L304
pub fn schnorr_sign_with_nonce<S: Signing>(
    secp: &Secp256k1<S>,
    msg: &[u8],
    key: SecretKey,
    nonce_key: SecretKey,
) -> Signature {
    let (rx, k) = get_schnorr_key(secp, nonce_key);
    let (xonly, x) = get_schnorr_key(secp, key);

    // concat tag || msg
    let mut m = Vec::with_capacity(64 + 32 + 32 + msg.len());
    m.extend(SCHNORR_TAG_BYTES);
    m.extend(rx.serialize());
    m.extend(xonly.serialize());
    m.extend(msg);
    let e = sha256::Hash::hash(&m);

    let challenge = x
        .mul_tweak(&Scalar::from_be_bytes(e.into_inner()).unwrap())
        .unwrap();

    let sig = k.add_tweak(&Scalar::from(challenge)).unwrap();

    let mut sig_bytes = Vec::with_capacity(64);
    sig_bytes.extend(rx.serialize());
    sig_bytes.extend(sig.secret_bytes());
    Signature::from_slice(&sig_bytes).unwrap()
}
