// adapted and extended from https://github.com/dalek-cryptography/bulletproofs/blob/main/src/util.rs
// licensed under MIT license

#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use byteorder::{LittleEndian, ByteOrder};
use bls12_381::{G1Affine, Scalar};
use group::Curve;

use super::inner_product_proof::inner_product;

/// Represents a degree-1 vector polynomial \\(\mathbf{a} + \mathbf{b} \cdot x\\).
pub struct VecPoly1(pub Vec<Scalar>, pub Vec<Scalar>);

/// Represents a degree-2 scalar polynomial \\(a + b \cdot x + c \cdot x^2\\)
pub struct Poly2(pub Scalar, pub Scalar, pub Scalar);

/// Provides an iterator over the powers of a `Scalar`.
///
/// This struct is created by the `exp_iter` function.
pub struct ScalarExp {
    x: Scalar,
    next_exp_x: Scalar,
}

impl Iterator for ScalarExp {
    type Item = Scalar;

    fn next(&mut self) -> Option<Scalar> {
        let exp_x = self.next_exp_x.clone();
        self.next_exp_x = &self.next_exp_x * &self.x;
        Some(exp_x)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::max_value(), None)
    }
}

/// Return an iterator of the powers of `x`.
pub fn exp_iter(x: Scalar) -> ScalarExp {
    let next_exp_x = Scalar::one();
    ScalarExp { x, next_exp_x }
}

enum Operation {
    Add,
    Mul,
    Sub,
}

fn subadd_vec(a: &[Scalar], b: &[Scalar], op: Operation) -> Vec<Scalar> {
    if a.len() != b.len() {
        // throw some error
        panic!("lengths of vectors don't match for vector addition");
    }
    let mut out = vec![Scalar::zero(); b.len()];
    for i in 0..a.len() {
        match op {
            Operation::Add => out[i] = &a[i] + &b[i],
            Operation::Mul => out[i] = &a[i] * &b[i],
            Operation::Sub => out[i] = &a[i] - &b[i],
        }
    }
    out
}

pub fn add_vec(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    subadd_vec(a,b,Operation::Add)
}
pub fn sub_vec(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    subadd_vec(a,b,Operation::Sub)
}
pub fn mul_vec(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    subadd_vec(a,b,Operation::Mul)
}

pub fn smul_vec(x: &Scalar, a: &[Scalar]) -> Vec<Scalar> {
    let mut out = vec![Scalar::zero(); a.len()];
    for i in 0..a.len() {
        out[i] = x * &a[i];
    }
    out
}

impl VecPoly1 {
    pub fn zero(n: usize) -> Self {
        VecPoly1(vec![Scalar::zero(); n], vec![Scalar::zero(); n])
    }

    pub fn inner_product(&self, rhs: &VecPoly1) -> Poly2 {
        // Uses Karatsuba's method
        let l = self;
        let r = rhs;

        let t0 = inner_product(&l.0, &r.0);
        let t2 = inner_product(&l.1, &r.1);

        let l0_plus_l1 = add_vec(&l.0, &l.1);
        let r0_plus_r1 = add_vec(&r.0, &r.1);

        let t1 = inner_product(&l0_plus_l1, &r0_plus_r1) - &t0 - &t2;

        Poly2(t0, t1, t2)
    }

    pub fn eval(&self, x: Scalar) -> Vec<Scalar> {
        let n = self.0.len();
        let mut out = vec![Scalar::zero(); n];
        for i in 0..n {
            out[i] = &self.0[i] + &self.1[i] * &x;
        }
        out
    }
}

impl Poly2 {
    pub fn eval(&self, x: Scalar) -> Scalar {
        &self.0 + &x * (&self.1 + &x * &self.2)
    }
}

/// Takes the sum of all the powers of `x`, up to `n`
/// If `n` is a power of 2, it uses the efficient algorithm with `2*lg n` multiplications and additions.
/// If `n` is not a power of 2, it uses the slow algorithm with `n` multiplications and additions.
/// In the Bulletproofs case, all calls to `sum_of_powers` should have `n` as a power of 2.
pub fn sum_of_powers(x: &Scalar, n: usize) -> Scalar {
    if !n.is_power_of_two() {
        return sum_of_powers_slow(x, n);
    }
    if n == 0 || n == 1 {
        return Scalar::from(n as u64);
    }
    let mut m = n;
    let mut result = Scalar::one() + x;
    let mut factor = x.clone();
    while m > 2 {
        factor = &factor * &factor;
        result = &result + &factor * &result;
        m = m / 2;
    }
    result
}

// takes the sum of all of the powers of x, up to n
fn sum_of_powers_slow(x: &Scalar, n: usize) -> Scalar {
    exp_iter(x.clone()).take(n).sum()
}

// returns a uniformly random scalar value
pub fn rand_scalar() -> Scalar {
    <bls12_381::Scalar as ff::Field>::random(rand::thread_rng())
}

// asserts pairwise equality of labeled tuple of points
pub fn assert_point(points: &[(&str, G1Affine, G1Affine)]) {
    for (id, p, g) in points {
        assert!(g.eq(&p), "point {} did not match {:?}", id, p.to_compressed());
    }
}

// asserts correctness of a precomputed generator based on a hash to curve operation
pub fn assert_generators(prefix: &str, g1: std::collections::HashMap<&str, G1Affine>) {
    use bls12_381::hash_to_curve::{HashToCurve, ExpandMsgXmd};
    for (k, g) in &g1 {
        let id = format!("{}_{}", prefix, k);
        let tag = id.as_bytes();
        let x = <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(tag, tag).to_affine();
        assert!(g.eq(&x), "generator {} did not match {:?}", id, x.to_compressed());
    }
}

// helper function to cast a scalar to an u32 integer with potential loss of precision
pub fn as_u32(s: &Scalar) -> u32 {
    let b = s.to_bytes();
    LittleEndian::read_u32(&b[..8])
}

// helper function to cast an u32 integer to a scalar
pub fn as_scalar(u: u32) -> Scalar {
    Scalar::from(u as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_2_is_powers_of_2() {
        let exp_2: Vec<_> = exp_iter(Scalar::from(2u64)).take(4).collect();

        assert_eq!(exp_2[0], Scalar::from(1u64));
        assert_eq!(exp_2[1], Scalar::from(2u64));
        assert_eq!(exp_2[2], Scalar::from(4u64));
        assert_eq!(exp_2[3], Scalar::from(8u64));
    }

    #[test]
    fn vectorops() {
        let a = vec![Scalar::from(12u64), Scalar::from(7u64)];
        let b = vec![Scalar::from(4u64), Scalar::from(5u64)];
        let s = Scalar::from(9u64);

        assert_eq!(mul_vec(&a,&b), vec![Scalar::from(48u64), Scalar::from(35u64)]);
        assert_eq!(add_vec(&a,&b), vec![Scalar::from(16u64), Scalar::from(12u64)]);
        assert_eq!(sub_vec(&a,&b), vec![Scalar::from(8u64), Scalar::from(2u64)]);
        assert_eq!(smul_vec(&s,&b), vec![Scalar::from(36u64), Scalar::from(45u64)]);
    }

    #[test]
    fn test_polynomials() {
        let p1 = VecPoly1(vec![Scalar::from(2u64), Scalar::from(3u64)], vec![Scalar::from(5u64), Scalar::from(7u64)]);
        let p2 = VecPoly1(vec![Scalar::from(3u64), Scalar::from(5u64)], vec![Scalar::from(6u64), Scalar::from(4u64)]);

        assert_eq!(p1.eval(Scalar::from(7u64)), vec![Scalar::from(37u64),Scalar::from(52u64)]);

        let comb = p1.inner_product(&p2);
        assert_eq!(comb.0,Scalar::from(21u64));
        assert_eq!(comb.1,Scalar::from(74u64));
        assert_eq!(comb.2,Scalar::from(58u64));
    }

    #[test]
    fn test_inner_product() {
        let a = vec![
            Scalar::from(1u64),
            Scalar::from(2u64),
            Scalar::from(3u64),
            Scalar::from(4u64),
        ];
        let b = vec![
            Scalar::from(2u64),
            Scalar::from(3u64),
            Scalar::from(4u64),
            Scalar::from(5u64),
        ];
        assert_eq!(Scalar::from(40u64), inner_product(&a, &b));
    }

    #[test]
    fn test_sum_of_powers() {
        let x = Scalar::from(10u64);
        assert_eq!(sum_of_powers_slow(&x, 0), sum_of_powers(&x, 0));
        assert_eq!(sum_of_powers_slow(&x, 1), sum_of_powers(&x, 1));
        assert_eq!(sum_of_powers_slow(&x, 2), sum_of_powers(&x, 2));
        assert_eq!(sum_of_powers_slow(&x, 4), sum_of_powers(&x, 4));
        assert_eq!(sum_of_powers_slow(&x, 8), sum_of_powers(&x, 8));
        assert_eq!(sum_of_powers_slow(&x, 16), sum_of_powers(&x, 16));
        assert_eq!(sum_of_powers_slow(&x, 32), sum_of_powers(&x, 32));
        assert_eq!(sum_of_powers_slow(&x, 64), sum_of_powers(&x, 64));
    }

    #[test]
    fn test_sum_of_powers_slow() {
        let x = Scalar::from(10u64);
        assert_eq!(sum_of_powers_slow(&x, 0), Scalar::zero());
        assert_eq!(sum_of_powers_slow(&x, 1), Scalar::one());
        assert_eq!(sum_of_powers_slow(&x, 2), Scalar::from(11u64));
        assert_eq!(sum_of_powers_slow(&x, 3), Scalar::from(111u64));
        assert_eq!(sum_of_powers_slow(&x, 4), Scalar::from(1111u64));
        assert_eq!(sum_of_powers_slow(&x, 5), Scalar::from(11111u64));
        assert_eq!(sum_of_powers_slow(&x, 6), Scalar::from(111111u64));
    }

}
