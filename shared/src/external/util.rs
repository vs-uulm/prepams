#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use byteorder::{LittleEndian, ByteOrder};
use bls12_381::{G1Affine, G1Projective, Scalar};
use group::Curve;

use super::inner_product_proof::inner_product;

/// Represents a degree-1 vector polynomial \\(\mathbf{a} + \mathbf{b} \cdot x\\).
pub struct VecPoly1(pub Vec<Scalar>, pub Vec<Scalar>);

/// Represents a degree-3 vector polynomial
/// \\(\mathbf{a} + \mathbf{b} \cdot x + \mathbf{c} \cdot x^2 + \mathbf{d} \cdot x^3 \\).
#[cfg(feature = "yoloproofs")]
pub struct VecPoly3(
    pub Vec<Scalar>,
    pub Vec<Scalar>,
    pub Vec<Scalar>,
    pub Vec<Scalar>,
);

/// Represents a degree-2 scalar polynomial \\(a + b \cdot x + c \cdot x^2\\)
pub struct Poly2(pub Scalar, pub Scalar, pub Scalar);

/// Represents a degree-6 scalar polynomial, without the zeroth degree
/// \\(a \cdot x + b \cdot x^2 + c \cdot x^3 + d \cdot x^4 + e \cdot x^5 + f \cdot x^6\\)
#[cfg(feature = "yoloproofs")]
pub struct Poly6 {
    pub t1: Scalar,
    pub t2: Scalar,
    pub t3: Scalar,
    pub t4: Scalar,
    pub t5: Scalar,
    pub t6: Scalar,
}

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

pub fn exp_vec(a: &[Scalar], b: &[G1Affine]) -> Vec<G1Affine> {
    if a.len() != b.len() {
        // throw some error
        panic!("lengths of vectors don't match for vector addition");
    }
    let mut out = vec![G1Projective::identity(); b.len()];
    for i in 0..a.len() {
        out[i] =  &b[i] * &a[i]
    }

    let mut normalized = vec![G1Affine::identity(); b.len()];
    G1Projective::batch_normalize(&out, &mut normalized);

    normalized
}

pub fn smul_vec(x: &Scalar, a: &[Scalar]) -> Vec<Scalar> {
    let mut out = vec![Scalar::zero(); a.len()];
    for i in 0..a.len() {
        out[i] = x * &a[i];
    }
    out
}

pub fn inv_vec(a: &[Scalar]) -> Vec<Scalar> {
    let mut out = vec![Scalar::zero(); a.len()];
    for i in 0..a.len() {
        out[i] = a[i].invert().unwrap();
    }
    out
}

pub fn kron_vec(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    let mut out = Vec::<Scalar>::new();
    for ina in a {
        out.extend(smul_vec(ina, b))
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

#[cfg(feature = "yoloproofs")]
impl VecPoly3 {
    pub fn zero(n: usize) -> Self {
        VecPoly3(
            vec![Scalar::zero(); n],
            vec![Scalar::zero(); n],
            vec![Scalar::zero(); n],
            vec![Scalar::zero(); n],
        )
    }

    /// Compute an inner product of `lhs`, `rhs` which have the property that:
    /// - `lhs.0` is zero;
    /// - `rhs.2` is zero;
    /// This is the case in the constraint system proof.
    pub fn special_inner_product(lhs: &Self, rhs: &Self) -> Poly6 {
        // TODO: make checks that l_poly.0 and r_poly.2 are zero.

        let t1 = inner_product(&lhs.1, &rhs.0);
        let t2 = inner_product(&lhs.1, &rhs.1) + inner_product(&lhs.2, &rhs.0);
        let t3 = inner_product(&lhs.2, &rhs.1) + inner_product(&lhs.3, &rhs.0);
        let t4 = inner_product(&lhs.1, &rhs.3) + inner_product(&lhs.3, &rhs.1);
        let t5 = inner_product(&lhs.2, &rhs.3);
        let t6 = inner_product(&lhs.3, &rhs.3);

        Poly6 {
            t1,
            t2,
            t3,
            t4,
            t5,
            t6,
        }
    }

    pub fn eval(&self, x: Scalar) -> Vec<Scalar> {
        let n = self.0.len();
        let mut out = vec![Scalar::zero(); n];
        for i in 0..n {
            out[i] = self.0[i] + x * (self.1[i] + x * (self.2[i] + x * self.3[i]));
        }
        out
    }
}

impl Poly2 {
    pub fn eval(&self, x: Scalar) -> Scalar {
        &self.0 + &x * (&self.1 + &x * &self.2)
    }
}

#[cfg(feature = "yoloproofs")]
impl Poly6 {
    pub fn eval(&self, x: Scalar) -> Scalar {
        &x * (&self.t1 + &x * (&self.t2 + &x * (&self.t3 + &x * (&self.t4 + &x * (&self.t5 + &x * &self.t6)))))
    }
}

/// Raises `x` to the power `n` using binary exponentiation,
/// with (1 to 2)*lg(n) scalar multiplications.
/// TODO: a consttime version of this would be awfully similar to a Montgomery ladder.
pub fn scalar_exp_vartime(x: &Scalar, mut n: u64) -> Scalar {
    let mut result = Scalar::one();
    let mut aux = x.clone(); // x, x^2, x^4, x^8, ...
    while n > 0 {
        let bit = n & 1;
        if bit == 1 {
            result = result * &aux;
        }
        n = n >> 1;
        aux = &aux * &aux; // FIXME: one unnecessary mult at the last step here!
    }
    result
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

/// Given `data` with `len >= 32`, return the first 32 bytes.
pub fn read32(data: &[u8]) -> [u8; 32] {
    let mut buf32 = [0u8; 32];
    buf32[..].copy_from_slice(&data[..32]);
    buf32
}

pub fn rand_scalar() -> Scalar {
    <bls12_381::Scalar as ff::Field>::random(rand::thread_rng())
}

pub fn assert_point(points: &[(&str, G1Affine, G1Affine)]) {
    for (id, p, g) in points {
        assert!(g.eq(&p), "point {} did not match {:?}", id, p.to_compressed());
    }
}

pub fn assert_generators(prefix: &str, g1: std::collections::HashMap<&str, G1Affine>) {
    use bls12_381::hash_to_curve::{HashToCurve, ExpandMsgXmd};
    for (k, g) in &g1 {
        let id = format!("{}_{}", prefix, k);
        let tag = id.as_bytes();
        let x = <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(tag, tag).to_affine();
        assert!(g.eq(&x), "generator {} did not match {:?}", id, x.to_compressed());
    }
}

pub fn as_u32(s: &Scalar) -> u32 {
    let b = s.to_bytes();
    LittleEndian::read_u32(&b[..8])
}

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
    fn invert() {
        let vec = vec![Scalar::from(123u64), Scalar::from(654u64)];
        let inv = inv_vec(&vec);

        assert_eq!(inv[0], Scalar::from(123u64).invert().unwrap());
        assert_eq!(inv[1], Scalar::from(654u64).invert().unwrap());
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

    /// Raises `x` to the power `n`.
    fn scalar_exp_vartime_slow(x: &Scalar, n: u64) -> Scalar {
        let mut result = Scalar::one();
        for _ in 0..n {
            result = result * x;
        }
        result
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
