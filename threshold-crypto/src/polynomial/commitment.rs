use bls12_381::{G1Projective, Scalar};
use group::Curve;
use std::cmp::Ordering;
use std::ops::{AddAssign, MulAssign};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commitment {
    coefficients: Vec<G1Projective>,
}

impl Commitment {
    pub(crate) fn new(coefficients: Vec<G1Projective>) -> Self {
        Self { coefficients }
    }

    pub fn degree(&self) -> usize {
        self.coefficients.len().saturating_sub(1)
    }

    pub fn get_coefficient(&self, index: usize) -> G1Projective {
        self.coefficients[index]
    }

    pub fn evaluate(&self, x: Scalar) -> G1Projective {
        if self.coefficients.len() == 0 {
            // return zero
            G1Projective::identity()
        } else {
            let mut result = *self.coefficients.last().unwrap();
            for c in self.coefficients.iter().rev().skip(1) {
                result.mul_assign(x);
                result.add_assign(c);
            }
            result
        }
    }
}

impl PartialOrd<Self> for Commitment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Commitment {
    fn cmp(&self, other: &Self) -> Ordering {
        self.coefficients
            .len()
            .cmp(&other.coefficients.len())
            .then_with(|| {
                self.coefficients
                    .iter()
                    .zip(&other.coefficients)
                    .find(|(x, y)| x != y)
                    .map_or(Ordering::Equal, |(x, y)| {
                        let xc = x.to_affine().to_compressed();
                        let yc = y.to_affine().to_compressed();
                        xc.as_ref().cmp(yc.as_ref())
                    })
            })
    }
}
