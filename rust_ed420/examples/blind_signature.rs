// examples/blind_signature.rs

use rust_ed420::schnorr::{
    keygen,
    BlindSignatureRequester,
    BlindSignatureSigner,
    verify_blind_signature,
};

fn main() {
    println!("--- Blind Signature Protocol ---");

    // 1. Setup: Signer generates keys
    let (signer_sk, signer_pk) = keygen();
    println!("Signer has generated a key pair.");

    // The message to be blindly signed
    let message = b"This message should be signed without the signer knowing its content.";
    println!("Requester has a message: \"{}\"", String::from_utf8_lossy(message));

    // --- Protocol Steps ---

    // 2. Signer creates a nonce and commitment R
    let signer = BlindSignatureSigner::new(signer_sk, signer_pk.clone());
    println!("\nStep 1: Signer creates a nonce commitment R.");

    // 3. Requester receives R and creates a blinded challenge
    println!("Step 2: Requester receives R and creates a blinded challenge.");
    let requester = BlindSignatureRequester::new(signer_pk.clone(), message, &signer.r);
    let blinded_challenge = requester.create_blinded_challenge();

    // 4. Signer signs the blinded challenge
    println!("Step 3: Signer signs the blinded challenge and sends it back.");
    let signed_challenge = signer.sign(&blinded_challenge);

    // 5. Requester unblinds the signature
    println!("Step 4: Requester unblinds the signature.");
    let final_signature = requester.unblind_signature(&signed_challenge);

    // 6. Verification
    println!("\n--- Verification ---");
    let is_valid = verify_blind_signature(&signer_pk, message, &final_signature);
    println!("Is the final signature valid? {}", is_valid);
    assert!(is_valid);

    // A third party can also verify the signature
    let third_party_is_valid = verify_blind_signature(&signer_pk, message, &final_signature);
    println!("Is the signature valid for a third party? {}", third_party_is_valid);
    assert!(third_party_is_valid);

    // Verification with a tampered message should fail
    let tampered_message = b"This is not the original message.";
    let tampered_is_valid = verify_blind_signature(&signer_pk, tampered_message, &final_signature);
    println!("Is the signature valid for a tampered message? {}", tampered_is_valid);
    assert!(!tampered_is_valid);
}
