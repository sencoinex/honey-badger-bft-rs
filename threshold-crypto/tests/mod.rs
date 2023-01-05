use bls12_381::Scalar;
use group::ff::Field;
use rand::thread_rng;
use threshold_crypto::{Ciphertext, DecryptionShare, SecretKey, SecretKeyShares, SignatureShare};

fn gen_random_secret() -> SecretKey {
    let mut rnd = thread_rng();
    let scalar = Scalar::random(&mut rnd);
    SecretKey::new(scalar)
}

fn gen_random_secret_key_shares(threshold: usize) -> SecretKeyShares {
    let mut rnd = thread_rng();
    SecretKeyShares::random(threshold, &mut rnd)
}

#[test]
fn test_simple_sig() {
    let sk0 = gen_random_secret();
    let sk1 = gen_random_secret();
    let pk0 = sk0.compute_public_key();
    let msg0 = b"Real news";
    let msg1 = b"Fake news";
    assert!(pk0.verify(&sk0.sign(msg0), msg0));
    assert!(!pk0.verify(&sk1.sign(msg0), msg0)); // Wrong key.
    assert!(!pk0.verify(&sk0.sign(msg1), msg0)); // Wrong message.
}

#[test]
fn test_threshold_sig() {
    let threshold = 3;
    let sk_shares = gen_random_secret_key_shares(threshold);
    let pk_shares = sk_shares.public_keys();
    let pk_master = pk_shares.public_key();

    // Make sure the keys are different, and the first coefficient is the main key.
    assert_ne!(pk_master, pk_shares.public_key_share(0u64).into_inner());
    assert_ne!(pk_master, pk_shares.public_key_share(1u64).into_inner());
    assert_ne!(pk_master, pk_shares.public_key_share(2u64).into_inner());

    // Make sure we don't hand out the main secret key to anyone.
    let sk_master = sk_shares.secret_key();
    let sk_share_0 = sk_shares.secret_key_share(0u64).into_inner();
    let sk_share_1 = sk_shares.secret_key_share(1u64).into_inner();
    let sk_share_2 = sk_shares.secret_key_share(2u64).into_inner();
    assert_ne!(sk_master, sk_share_0);
    assert_ne!(sk_master, sk_share_1);
    assert_ne!(sk_master, sk_share_2);

    let msg = "Totally real news";

    // The threshold is 3, so 4 signature shares will suffice to recreate the share.
    let sigs: Vec<(u64, SignatureShare)> = [5u64, 8u64, 7u64, 10u64]
        .iter()
        .map(|&i| {
            let sig = sk_shares.secret_key_share(i).sign(msg);
            (i, sig)
        })
        .collect();

    // Each of the shares is a valid signature matching its public key share.
    for (i, sig) in &sigs {
        assert!(pk_shares.public_key_share(*i).verify(sig, msg));
    }

    // Combined, they produce a signature matching the main public key.
    let sig = pk_shares
        .combine_signatures(sigs.iter().map(|(i, sig)| (*i, sig)))
        .expect("signatures match");
    assert!(pk_master.verify(&sig, msg));

    // Another set of signatories produces the same signature.
    let sigs2: Vec<(u64, SignatureShare)> = [42u64, 43u64, 44u64, 45u64]
        .iter()
        .map(|&i| {
            let sig = sk_shares.secret_key_share(i).sign(msg);
            (i, sig)
        })
        .collect();
    let sig2 = pk_shares
        .combine_signatures(sigs2.iter().map(|(i, sig)| (*i, sig)))
        .expect("signatures match");
    assert_eq!(sig, sig2);
}

#[test]
fn test_simple_enc() {
    let sk_bob = gen_random_secret();
    let sk_eve = gen_random_secret();
    let pk_bob = sk_bob.compute_public_key();
    let msg = b"Muffins in the canteen today! Don't tell Eve!";
    let ciphertext = pk_bob.encrypt(&msg[..]);
    assert!(ciphertext.verify());

    // Bob can decrypt the message.
    let decrypted = sk_bob.decrypt(&ciphertext).expect("invalid ciphertext");
    assert_eq!(msg[..], decrypted[..]);

    // Eve can't.
    let decrypted_eve = sk_eve.decrypt(&ciphertext).expect("invalid ciphertext");
    assert_ne!(msg[..], decrypted_eve[..]);

    // Eve tries to trick Bob into decrypting `msg` xor `v`, but it doesn't validate.
    let u = ciphertext.as_g1();
    let v = ciphertext.as_msg();
    let w = ciphertext.as_g2();
    let fake_ciphertext = Ciphertext::new(*u, vec![0; v.len()], *w);
    assert!(!fake_ciphertext.verify());
    assert_eq!(None, sk_bob.decrypt(&fake_ciphertext));
}

#[test]
fn test_threshold_enc() {
    let threshold = 3;
    let sk_shares = gen_random_secret_key_shares(threshold);
    let pk_shares = sk_shares.public_keys();
    let pk_master = pk_shares.public_key();
    let msg = b"Totally real news";
    let ciphertext = pk_master.encrypt(&msg[..]);

    // The threshold is 3, so 4 signature shares will suffice to decrypt.
    let shares: Vec<(u64, DecryptionShare)> = [5u64, 8u64, 7u64, 10u64]
        .iter()
        .map(|&i| {
            let dec_share = sk_shares
                .secret_key_share(i)
                .decrypt_share(&ciphertext)
                .expect("ciphertext must be invalid");
            (i, dec_share)
        })
        .collect();

    // Each of the shares is valid matching its public key share.
    for (i, share) in &shares {
        pk_shares
            .public_key_share(*i)
            .verify_decryption_share(share, &ciphertext);
    }

    // Combined, they can decrypt the message.
    let decrypted = pk_shares
        .decrypt(
            shares.iter().map(|(i, dec_share)| (*i, dec_share)),
            &ciphertext,
        )
        .expect("decryption shares must match");
    assert_eq!(msg[..], decrypted[..]);
}
