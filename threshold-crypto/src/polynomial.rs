mod commitment;
pub use commitment::Commitment;

use bls12_381::{G1Affine, Scalar};
use std::ops::{AddAssign, Mul, MulAssign};

#[derive(PartialEq, Eq)]
pub struct Polynomial {
    coefficients: Vec<Scalar>,
}

impl Polynomial {
    pub fn new(coefficients: Vec<Scalar>) -> Self {
        Self { coefficients }
    }

    pub fn degree(&self) -> usize {
        self.coefficients.len().saturating_sub(1)
    }

    pub fn evaluate(&self, x: Scalar) -> Scalar {
        if self.coefficients.len() == 0 {
            Scalar::zero()
        } else {
            let mut result = *self.coefficients.last().unwrap();
            for c in self.coefficients.iter().rev().skip(1) {
                result.mul_assign(&x);
                result.add_assign(c);
            }
            result
        }
    }

    pub fn commitment(&self) -> Commitment {
        let to_g1 = |c: &Scalar| G1Affine::generator().mul(*c);
        Commitment::new(self.coefficients.iter().map(to_g1).collect())
    }
}
