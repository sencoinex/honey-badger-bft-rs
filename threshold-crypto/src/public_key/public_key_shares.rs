use super::{PublicKey, PublicKeyShare};
use crate::{
    hasher, polynomial::Commitment, Ciphertext, DecryptionShare, Error, Result, Signature,
    SignatureShare,
};
use bls12_381::Scalar;
use group::Curve;
use std::ops::{AddAssign, Mul, MulAssign, SubAssign};

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct PublicKeyShares {
    /// The coefficients of a polynomial whose value at `0` is the "master key", and value at
    /// `i + 1` is key share number `i`.
    commit: Commitment,
}

impl PublicKeyShares {
    pub(crate) fn new(commit: Commitment) -> Self {
        Self { commit }
    }

    pub fn threshold(&self) -> usize {
        self.commit.degree()
    }

    /// Returns the master public key
    pub fn public_key(&self) -> PublicKey {
        let c0 = self.commit.get_coefficient(0);
        PublicKey::new(c0)
    }

    /// Returns the `i`-th public key share.
    pub fn public_key_share<T: Into<Scalar>>(&self, i: T) -> PublicKeyShare {
        let mut x = Scalar::one();
        x.add_assign(i.into());
        let value = self.commit.evaluate(x);
        PublicKeyShare::new(PublicKey::new(value))
    }

    /// Combines the shares into a signature that can be verified with the main public key.
    pub fn combine_signatures<'a, I>(&self, shares: I) -> Result<Signature>
    where
        I: Iterator<Item = (u64, &'a SignatureShare)>,
    {
        let samples =
            shares.map(|(i, signature_share)| (Scalar::from(i), signature_share.as_ref().as_ref()));
        Ok(Signature::new(interpolate(self.commit.degree(), samples)?))
    }

    pub fn decrypt<'a, I>(&self, shares: I, ct: &Ciphertext) -> Result<Vec<u8>>
    where
        I: Iterator<Item = (u64, &'a DecryptionShare)>,
    {
        let samples =
            shares.map(|(i, decryption_share)| (Scalar::from(i), decryption_share.as_ref()));
        let g = interpolate(self.commit.degree(), samples)?;
        Ok(hasher::xor_with_hash(g.to_affine(), &ct.as_msg()))
    }
}

impl From<Commitment> for PublicKeyShares {
    fn from(commit: Commitment) -> Self {
        Self::new(commit)
    }
}

fn interpolate<'a, I, C, A>(t: usize, items: I) -> Result<C>
where
    I: Iterator<Item = (Scalar, &'a C)>,
    C: Curve<AffineRepr = A> + AddAssign<C>,
    A: Mul<Scalar, Output = C>,
{
    let samples: Vec<_> = items
        .take(t + 1)
        .map(|(i, sample)| {
            let mut x = Scalar::one();
            x.add_assign(i);
            (x, sample)
        })
        .collect();
    if samples.len() <= t {
        return Err(Error::NotEnoughShares);
    }
    if t == 0 {
        return Ok(*samples[0].1);
    }
    // Compute the products `x_prod[i]` of all but the `i`-th entry.
    let mut x_prod: Vec<Scalar> = Vec::with_capacity(t);
    let mut tmp = Scalar::one();
    x_prod.push(tmp);
    for (x, _) in samples.iter().take(t) {
        tmp.mul_assign(x);
        x_prod.push(tmp);
    }
    tmp = Scalar::one();
    for (i, (x, _)) in samples[1..].iter().enumerate().rev() {
        tmp.mul_assign(x);
        x_prod[i].mul_assign(&tmp);
    }

    let mut result = C::identity();
    for (mut l0, (x, sample)) in x_prod.into_iter().zip(&samples) {
        // Compute the value at 0 of the Lagrange polynomial that is `0` at the other data
        // points but `1` at `x`.
        let mut denom = Scalar::one();
        for (x0, _) in samples.iter().filter(|(x0, _)| x0 != x) {
            let mut diff = *x0;
            diff.sub_assign(x);
            denom.mul_assign(&diff);
        }
        let inv = &denom.invert();
        if inv.is_none().into() {
            return Err(Error::DuplicateEntry);
        }
        l0.mul_assign(inv.unwrap());
        result.add_assign(&sample.to_affine().mul(l0));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polynomial::Polynomial;
    use group::ff::Field;
    use rand::{thread_rng, Rng, RngCore};

    fn gen_random_polynomial(degree: usize, mut rng: impl RngCore) -> Polynomial {
        let mut coefficients = vec![];
        for _ in 0..(degree + 1) {
            let coefficient = Scalar::random(&mut rng);
            coefficients.push(coefficient);
        }
        Polynomial::new(coefficients)
    }

    #[test]
    fn test_interpolate() {
        let mut rng = thread_rng();
        for degree in 0..5 {
            println!("degree = {}", degree);
            let polynomial = gen_random_polynomial(degree, &mut rng);
            let commitment = polynomial.commitment();
            let mut values = Vec::new();
            let mut x = 0;
            for _ in 0..=degree {
                x += rng.gen_range(1..5);
                values.push((x - 1, commitment.evaluate(x.into())));
            }
            let items = values.iter().map(|(x, curve)| (Scalar::from(*x), curve));
            let actual = interpolate(degree, items).expect("wrong number of values");
            assert_eq!(commitment.evaluate(0.into()), actual);
        }
    }
}
