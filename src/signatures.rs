use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use signature::Signature as Sign;

pub fn verify_signature(public_key: String, signature: String, message: &[u8]) -> bool {
    let public_key = hex::decode(public_key).unwrap();
    let signature = hex::decode(signature).unwrap();
    let signature = Signature::from_bytes(&signature).unwrap();
    let verify_key = VerifyingKey::from_sec1_bytes(&public_key).unwrap();

    verify_key.verify(&message, &signature).is_ok()
}
