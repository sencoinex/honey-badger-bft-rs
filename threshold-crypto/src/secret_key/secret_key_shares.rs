use super::{SecretKey, SecretKeyShare};
use crate::polynomial::Polynomial;
use crate::PublicKeyShares;
use bls12_381::Scalar;
use group::ff::Field;
use rand::RngCore;
use std::ops::AddAssign;

pub struct SecretKeyShares {
    polynomial: Polynomial,
}

impl SecretKeyShares {
    pub(crate) fn new(polynomial: Polynomial) -> Self {
        Self { polynomial }
    }

    pub fn threshold(&self) -> usize {
        self.polynomial.degree()
    }

    /// Returns the `i`-th secret key share.
    pub fn secret_key_share<T: Into<Scalar>>(&self, i: T) -> SecretKeyShare {
        let mut x = Scalar::one();
        x.add_assign(i.into());
        let scalar = self.polynomial.evaluate(x);
        SecretKeyShare::new(SecretKey::new(scalar))
    }

    pub fn public_keys(&self) -> PublicKeyShares {
        PublicKeyShares::new(self.polynomial.commitment())
    }

    pub fn secret_key(&self) -> SecretKey {
        SecretKey::new(self.polynomial.evaluate(Scalar::zero()))
    }

    pub fn random(threshold: usize, mut rng: impl RngCore) -> Self {
        let degree = threshold;
        let mut coefficients = vec![];
        for _ in 0..(degree + 1) {
            let coefficient = Scalar::random(&mut rng);
            coefficients.push(coefficient);
        }
        let polynomial = Polynomial::new(coefficients);
        Self::from(polynomial)
    }
}

impl From<Polynomial> for SecretKeyShares {
    fn from(polynomial: Polynomial) -> Self {
        Self::new(polynomial)
    }
}
