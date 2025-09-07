// examples/partially_blind_signature.rs

use rust_ed420::schnorr::{
    keygen,
    PartiallyBlindSignatureRequester,
    BlindSignatureSigner, // The same signer can be used
    verify_partially_blind_signature,
};

fn main() {
    println!("--- Partially Blind Signature Protocol ---");

    // 1. Setup: Signer generates keys
    let (signer_sk, signer_pk) = keygen();
    println!("Signer has generated a key pair.");

    // The message to be partially blindly signed
    let message = b"This is the blinded part of the message.";
    let info = b"This is public info, known to both parties.";
    println!("Requester has a message with a public part (info) and a private part.");
    println!("  Public Info: \"{}\"", String::from_utf8_lossy(info));
    println!("  Private Msg: \"{}\"", String::from_utf8_lossy(message));

    // --- Protocol Steps ---

    // 2. Signer creates a nonce and commitment R
    let signer = BlindSignatureSigner::new(signer_sk, signer_pk.clone());
    println!("\nStep 1: Signer creates a nonce commitment R.");

    // 3. Requester receives R and creates a blinded challenge
    println!("Step 2: Requester receives R and creates a blinded challenge.");
    let requester = PartiallyBlindSignatureRequester::new(signer_pk.clone(), message, info, &signer.r);
    let blinded_challenge = requester.create_blinded_challenge();

    // 4. Signer signs the blinded challenge
    println!("Step 3: Signer signs the blinded challenge and sends it back.");
    let signed_challenge = signer.sign(&blinded_challenge);

    // 5. Requester unblinds the signature
    println!("Step 4: Requester unblinds the signature.");
    let final_signature = requester.unblind_signature(&signed_challenge);

    // 6. Verification
    println!("\n--- Verification ---");
    let is_valid = verify_partially_blind_signature(&signer_pk, message, &final_signature);
    println!("Is the final signature valid? {}", is_valid);
    assert!(is_valid);

    // A third party can also verify the signature
    let third_party_is_valid = verify_partially_blind_signature(&signer_pk, message, &final_signature);
    println!("Is the signature valid for a third party? {}", third_party_is_valid);
    assert!(third_party_is_valid);

    // Verification with a tampered message should fail
    let tampered_message = b"This is not the original private part.";
    let tampered_is_valid = verify_partially_blind_signature(&signer_pk, tampered_message, &final_signature);
    println!("Is the signature valid for a tampered message? {}", tampered_is_valid);
    assert!(!tampered_is_valid);

    // Verification with tampered info should also fail
    let mut tampered_sig_info = final_signature.clone();
    tampered_sig_info.info = b"This is not the original public info.".to_vec();
    let tampered_info_is_valid = verify_partially_blind_signature(&signer_pk, message, &tampered_sig_info);
    println!("Is the signature valid for tampered info? {}", tampered_info_is_valid);
    assert!(!tampered_info_is_valid);
}
