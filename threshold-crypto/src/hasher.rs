use bls12_381::{G1Affine, G2Affine, G2Projective};
use group::{Curve, Group};
use rand::{distributions::Standard, Rng};
use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use tiny_keccak::{Hasher, Sha3};

fn sha3_256(data: &[u8]) -> [u8; 32] {
    let mut output = [0; 32];
    let mut sha3 = Sha3::v256();
    sha3.update(data);
    sha3.finalize(&mut output);
    output
}

/// convert bytes message to G2 element
pub fn hash<M: AsRef<[u8]>>(msg: M) -> G2Affine {
    let digest = sha3_256(msg.as_ref());
    let mut rand_core = ChaChaRng::from_seed(digest);
    let projective = G2Projective::random(&mut rand_core);
    projective.to_affine()
}

/// convert bytes message & G1 element into G2 element
pub fn hash_with_g1<M: AsRef<[u8]>>(g1: G1Affine, msg: M) -> G2Affine {
    let mut msg = if msg.as_ref().len() > 64 {
        sha3_256(msg.as_ref()).to_vec()
    } else {
        msg.as_ref().to_vec()
    };
    msg.extend(g1.to_compressed().as_ref());
    hash(&msg)
}

pub fn xor_with_hash(g1: G1Affine, bytes: &[u8]) -> Vec<u8> {
    let digest = sha3_256(g1.to_compressed().as_ref());
    let rand_core = ChaChaRng::from_seed(digest);
    let xor = |(a, b): (u8, &u8)| a ^ b;
    rand_core
        .sample_iter(&Standard)
        .zip(bytes)
        .map(xor)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bls12_381::G1Projective;
    use rand::distributions::Standard;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_hash() {
        let rng = thread_rng();
        let msg: Vec<u8> = rng.sample_iter(&Standard).take(1000).collect();
        let msg_end0: Vec<u8> = msg.iter().chain(b"end0").cloned().collect();
        let msg_end1: Vec<u8> = msg.iter().chain(b"end1").cloned().collect();
        assert_eq!(hash(&msg), hash(&msg));
        assert_ne!(hash(&msg), hash(&msg_end0));
        assert_ne!(hash(&msg_end0), hash(&msg_end1));
    }

    #[test]
    fn test_hash_with_g1() {
        let rng = thread_rng();
        let msg: Vec<u8> = rng.sample_iter(&Standard).take(1000).collect();
        let msg_end0: Vec<u8> = msg.iter().chain(b"end0").cloned().collect();
        let msg_end1: Vec<u8> = msg.iter().chain(b"end1").cloned().collect();
        let mut rng = thread_rng();
        let g0 = G1Projective::random(&mut rng).to_affine();
        let g1 = G1Projective::random(&mut rng).to_affine();
        assert_eq!(hash_with_g1(g0, &msg), hash_with_g1(g0, &msg));
        assert_ne!(hash_with_g1(g0, &msg), hash_with_g1(g0, &msg_end0));
        assert_ne!(hash_with_g1(g0, &msg_end0), hash_with_g1(g0, &msg_end1));
        assert_ne!(hash_with_g1(g0, &msg), hash_with_g1(g1, &msg));
    }

    #[test]
    fn test_xor_with_hash() {
        let mut rng = thread_rng();
        let g0 = G1Projective::random(&mut rng).to_affine();
        let g1 = G1Projective::random(&mut rng).to_affine();
        assert_eq!(xor_with_hash(g0, &[0; 5]), xor_with_hash(g0, &[0; 5]));
        assert_ne!(xor_with_hash(g0, &[0; 5]), xor_with_hash(g1, &[0; 5]));
        assert_eq!(5, xor_with_hash(g0, &[0; 5]).len());
        assert_eq!(6, xor_with_hash(g0, &[0; 6]).len());
        assert_eq!(20, xor_with_hash(g0, &[0; 20]).len());
    }
}
