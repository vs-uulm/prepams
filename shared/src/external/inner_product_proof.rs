// adapted from https://github.com/dalek-cryptography/bulletproofs/blob/main/src/inner_product_proof.rs
// licensed under MIT license

#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate alloc;

use core::iter;
use alloc::vec::Vec;
use alloc::borrow::Borrow;

use group::Curve;
use merlin::Transcript;
use bls12_381::{G1Affine, G1Projective, Scalar};
use serde::{Serialize, Deserialize};
use serde_with::serde_as;

use crate::types::ProofError;
use super::transcript::TranscriptProtocol;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InnerProductProof {
    #[serde_as(as = "Vec<crate::serialization::SerializableG1Affine>")]
    pub(crate) L_vec: Vec<G1Affine>,
    #[serde_as(as = "Vec<crate::serialization::SerializableG1Affine>")]
    pub(crate) R_vec: Vec<G1Affine>,
    #[serde(with = "crate::serialization::Scalar")]
    pub(crate) a: Scalar,
    #[serde(with = "crate::serialization::Scalar")]
    pub(crate) b: Scalar,
}

pub fn vartime_multiscalar_mul<I, J>(scalars: I, points: J) -> G1Affine
    where
        I: IntoIterator,
        I::Item: Borrow<Scalar>,
        J: IntoIterator,
        J::Item: Borrow<G1Affine>,
    {
        // I multiplications, I additions
        scalars.into_iter()
            .zip(points.into_iter())
            .fold(G1Projective::identity(), |a, (l, r)| a + r.borrow() * l.borrow())
            .to_affine()
    }

impl InnerProductProof {
    /// Create an inner-product proof.
    ///
    /// The proof is created with respect to the bases \\(G\\), \\(H'\\),
    /// where \\(H'\_i = H\_i \cdot \texttt{Hprime\\_factors}\_i\\).
    ///
    /// The `verifier` is passed in as a parameter so that the
    /// challenges depend on the *entire* transcript (including parent
    /// protocols).
    ///
    /// The lengths of the vectors must all be the same, and must all be
    /// either 0 or a power of 2.
    pub fn create(
        transcript: &mut dyn TranscriptProtocol,
        Q: &G1Affine,
        G_factors: &[Scalar],
        H_factors: &[Scalar],
        mut G_vec: Vec<G1Affine>,
        mut H_vec: Vec<G1Affine>,
        mut a_vec: Vec<Scalar>,
        mut b_vec: Vec<Scalar>,
    ) -> InnerProductProof {
        // Create slices G, H, a, b backed by their respective
        // vectors.  This lets us reslice as we compress the lengths
        // of the vectors in the main loop below.
        let mut G = &mut G_vec[..];
        let mut H = &mut H_vec[..];
        let mut a = &mut a_vec[..];
        let mut b = &mut b_vec[..];

        let mut n = G.len();

        // All of the input vectors must have the same length.
        assert_eq!(G.len(), n);
        assert_eq!(H.len(), n);
        assert_eq!(a.len(), n);
        assert_eq!(b.len(), n);
        assert_eq!(G_factors.len(), n);
        assert_eq!(H_factors.len(), n);

        // All of the input vectors must have a length that is a power of two.
        assert!(n.is_power_of_two());

        transcript.append_u64(b"ipp_n", n as u64);

        let lg_n = n.next_power_of_two().trailing_zeros() as usize;
        let mut L_vec = Vec::with_capacity(lg_n);
        let mut R_vec = Vec::with_capacity(lg_n);

        // If it's the first iteration, unroll the Hprime = H*y_inv scalar mults
        // into multiscalar muls, for performance.
        if n != 1 {
            n = n / 2;
            let (a_L, a_R) = a.split_at_mut(n);
            let (b_L, b_R) = b.split_at_mut(n);
            let (G_L, G_R) = G.split_at_mut(n);
            let (H_L, H_R) = H.split_at_mut(n);

            let c_L = inner_product(&a_L, &b_R);
            let c_R = inner_product(&a_R, &b_L);

            let L = vartime_multiscalar_mul(
                a_L.iter()
                    .zip(G_factors[n..2 * n].into_iter())
                    .map(|(a_L_i, g)| a_L_i * g)
                    .chain(
                        b_R.iter()
                            .zip(H_factors[0..n].into_iter())
                            .map(|(b_R_i, h)| b_R_i * h),
                    )
                    .chain(iter::once(c_L)),
                G_R.iter().chain(H_L.iter()).chain(iter::once(Q)),
            );

            let R = vartime_multiscalar_mul(
                a_R.iter()
                    .zip(G_factors[0..n].into_iter())
                    .map(|(a_R_i, g)| a_R_i * g)
                    .chain(
                        b_L.iter()
                            .zip(H_factors[n..2 * n].into_iter())
                            .map(|(b_L_i, h)| b_L_i * h),
                    )
                    .chain(iter::once(c_R)),
                G_L.iter().chain(H_R.iter()).chain(iter::once(Q)),
            );

            L_vec.push(L.clone());
            R_vec.push(R.clone());

            transcript.append_g1(b"L", &L);
            transcript.append_g1(b"R", &R);

            let u = transcript.challenge_scalar(b"u");
            let u_inv = u.invert().unwrap();

            for i in 0..n {
                a_L[i] = &a_L[i] * &u + &u_inv * &a_R[i];
                b_L[i] = &b_L[i] * &u_inv + &u * &b_R[i];
                G_L[i] = vartime_multiscalar_mul(
                    [&u_inv * &G_factors[i], &u * &G_factors[n + i]],
                    [&G_L[i], &G_R[i]],
                );
                H_L[i] = vartime_multiscalar_mul(
                    [&u * &H_factors[i], &u_inv * &H_factors[n + i]],
                    [&H_L[i], &H_R[i]],
                )
            }

            a = a_L;
            b = b_L;
            G = G_L;
            H = H_L;
        }

        while n != 1 {
            n = n / 2;
            let (a_L, a_R) = a.split_at_mut(n);
            let (b_L, b_R) = b.split_at_mut(n);
            let (G_L, G_R) = G.split_at_mut(n);
            let (H_L, H_R) = H.split_at_mut(n);

            let c_L = inner_product(&a_L, &b_R);
            let c_R = inner_product(&a_R, &b_L);

            let L = vartime_multiscalar_mul(
                a_L.iter().chain(b_R.iter()).chain(iter::once(&c_L)),
                G_R.iter().chain(H_L.iter()).chain(iter::once(Q)),
            );

            let R = vartime_multiscalar_mul(
                a_R.iter().chain(b_L.iter()).chain(iter::once(&c_R)),
                G_L.iter().chain(H_R.iter()).chain(iter::once(Q)),
            );

            L_vec.push(L.clone());
            R_vec.push(R.clone());

            transcript.append_g1(b"L", &L);
            transcript.append_g1(b"R", &R);

            let u = transcript.challenge_scalar(b"u");
            let u_inv = u.invert().unwrap();

            for i in 0..n {
                a_L[i] = &a_L[i] * &u + &u_inv * &a_R[i];
                b_L[i] = &b_L[i] * &u_inv + &u * &b_R[i];
                G_L[i] = vartime_multiscalar_mul([&u_inv, &u], [&G_L[i], &G_R[i]]);
                H_L[i] = vartime_multiscalar_mul([&u, &u_inv], [&H_L[i], &H_R[i]]);
            }

            a = a_L;
            b = b_L;
            G = G_L;
            H = H_L;
        }

        InnerProductProof {
            L_vec: L_vec,
            R_vec: R_vec,
            a: a[0].clone(),
            b: b[0].clone(),
        }
    }

    /// Computes three vectors of verification scalars \\([u\_{i}^{2}]\\), \\([u\_{i}^{-2}]\\) and \\([s\_{i}]\\) for combined multiscalar multiplication
    /// in a parent protocol. See [inner product protocol notes](index.html#verification-equation) for details.
    /// The verifier must provide the input length \\(n\\) explicitly to avoid unbounded allocation within the inner product proof.
    pub(crate) fn verification_scalars(
        &self,
        n: usize,
        transcript: &mut dyn TranscriptProtocol,
    ) -> Result<(Vec<Scalar>, Vec<Scalar>, Vec<Scalar>), ProofError> {
        let lg_n = self.L_vec.len();
        if lg_n >= 32 {
            // 4 billion multiplications should be enough for anyone
            // and this check prevents overflow in 1<<lg_n below.
            return Err(ProofError::VerificationError);
        }
        if n != (1 << lg_n) {
            return Err(ProofError::VerificationError);
        }

        transcript.append_u64(b"ipp_n", n as u64);

        // 1. Recompute x_k,...,x_1 based on the proof transcript

        let mut challenges = Vec::with_capacity(lg_n);
        for (L, R) in self.L_vec.iter().zip(self.R_vec.iter()) {
            if L.is_identity().into() || R.is_identity().into() {
                Err(ProofError::VerificationError)?;
            }

            transcript.append_g1(b"L", L);
            transcript.append_g1(b"R", R);
            challenges.push(transcript.challenge_scalar(b"u"));
        }

        // 2. Compute 1/(u_k...u_1) and 1/u_k, ..., 1/u_1

        let mut allinv = Scalar::one();
        let mut challenges_inv : Vec<Scalar> = challenges.iter().map(|c| {
            allinv = &allinv * c;
            c.invert().unwrap()
        }).collect();
        allinv = allinv.invert().unwrap();

        // 3. Compute u_i^2 and (1/u_i)^2

        for i in 0..lg_n {
            challenges[i] = challenges[i].square();
            challenges_inv[i] = challenges_inv[i].square();
        }
        let challenges_sq = challenges;
        let challenges_inv_sq = challenges_inv;

        // 4. Compute s values inductively.

        let mut s = Vec::with_capacity(n);
        s.push(allinv);
        for i in 1..n {
            let lg_i = (32 - 1 - (i as u32).leading_zeros()) as usize;
            let k = 1 << lg_i;
            // The challenges are stored in "creation order" as [u_k,...,u_1],
            // so u_{lg(i)+1} = is indexed by (lg_n-1) - lg_i
            let u_lg_i_sq = &challenges_sq[(lg_n - 1) - lg_i];
            s.push(&s[i - k] * u_lg_i_sq);
        }

        Ok((challenges_sq, challenges_inv_sq, s))
    }

    /// This method is for testing that proof generation work,
    /// but for efficiency the actual protocols would use `verification_scalars`
    /// method to combine inner product verification with other checks
    /// in a single multiscalar multiplication.
    #[allow(dead_code)]
    pub fn verify<IG, IH>(
        &self,
        n: usize,
        transcript: &mut Transcript,
        G_factors: IG,
        H_factors: IH,
        P: &G1Affine,
        Q: &G1Affine,
        G: &[G1Affine],
        H: &[G1Affine],
    ) -> Result<(), ProofError>
        where
            IG: IntoIterator,
            IG::Item: Borrow<Scalar>,
            IH: IntoIterator,
            IH::Item: Borrow<Scalar>,
    {
        let (u_sq, u_inv_sq, s) = self.verification_scalars(n, transcript)?;

        let g_times_a_times_s = G_factors
            .into_iter()
            .zip(s.iter())
            .map(|(g_i, s_i)| (&self.a * s_i) * g_i.borrow())
            .take(G.len());

        // 1/s[i] is s[!i], and !i runs from n-1 to 0 as i runs from 0 to n-1
        let inv_s = s.iter().rev();

        let h_times_b_div_s = H_factors
            .into_iter()
            .zip(inv_s)
            .map(|(h_i, s_i_inv)| (&self.b * s_i_inv) * h_i.borrow());

        let neg_u_sq = u_sq.iter().map(|ui| -ui);
        let neg_u_inv_sq = u_inv_sq.iter().map(|ui| -ui);

        let Ls = self
            .L_vec
            .iter()
            .map(|p| Ok(p.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let Rs = self
            .R_vec
            .iter()
            .map(|p| Ok(p.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let expect_P = vartime_multiscalar_mul(
            iter::once(&self.a * &self.b)
                .chain(g_times_a_times_s)
                .chain(h_times_b_div_s)
                .chain(neg_u_sq)
                .chain(neg_u_inv_sq),
            iter::once(Q)
                .chain(G.iter())
                .chain(H.iter())
                .chain(Ls.iter())
                .chain(Rs.iter()),
        );

        if expect_P == *P {
            Ok(())
        } else {
            Err(ProofError::VerificationError)
        }
    }

    /// Returns the size in bytes required to serialize the inner
    /// product proof.
    ///
    /// For vectors of length `n` the proof size is
    /// \\(32 \cdot (2\lg n+2)\\) bytes.
    pub fn serialized_size(&self) -> usize {
        (self.L_vec.len() * 2 + 2) * 32
    }

    /// Serializes the proof into a byte array of \\(2n+2\\) 32-byte elements.
    /// The layout of the inner product proof is:
    /// * \\(n\\) pairs of compressed Ristretto points \\(L_0, R_0 \dots, L_{n-1}, R_{n-1}\\),
    /// * two scalars \\(a, b\\).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.serialized_size());
        for (l, r) in self.L_vec.iter().zip(self.R_vec.iter()) {
            buf.extend_from_slice(&l.to_compressed());
            buf.extend_from_slice(&r.to_compressed());
        }
        buf.extend_from_slice(&self.a.to_bytes());
        buf.extend_from_slice(&self.b.to_bytes());
        buf
    }
}

/// Computes an inner product of two vectors
/// \\[
///    {\langle {\mathbf{a}}, {\mathbf{b}} \rangle} = \sum\_{i=0}^{n-1} a\_i \cdot b\_i.
/// \\]
/// Panics if the lengths of \\(\mathbf{a}\\) and \\(\mathbf{b}\\) are not equal.
pub fn inner_product(a: &[Scalar], b: &[Scalar]) -> Scalar {
    let mut out = Scalar::zero();
    if a.len() != b.len() {
        panic!("inner_product(a,b): lengths of vectors do not match");
    }
    for i in 0..a.len() {
        out = out + (&a[i] * &b[i]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::util;

    use ff::Field;
    use bls12_381::hash_to_curve::{HashToCurve, ExpandMsgXmd};

    fn test_helper_create(n: usize) {
        let G: Vec<G1Affine> = (0..n).map(|i| <G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(b"testG", &i.to_be_bytes()).to_affine()).collect();
        let H: Vec<G1Affine> = (0..n).map(|i| <G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(b"testH", &i.to_be_bytes()).to_affine()).collect();

        // Q would be determined upstream in the protocol, so we pick a random one.
        let Q = <G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(b"test", b"point").to_affine();

        // a and b are the vectors for which we want to prove c = <a,b>
        let a: Vec<_> = (0..n).map(|_| Scalar::random(rand::thread_rng())).collect();
        let b: Vec<_> = (0..n).map(|_| Scalar::random(rand::thread_rng())).collect();
        let c = inner_product(&a, &b);

        let G_factors: Vec<Scalar> = iter::repeat(Scalar::one()).take(n).collect();

        // y_inv is (the inverse of) a random challenge
        let y_inv = Scalar::random(rand::thread_rng());
        let H_factors: Vec<Scalar> = util::exp_iter(y_inv.clone()).take(n).collect();

        // P would be determined upstream, but we need a correct P to check the proof.
        //
        // To generate P = <a,G> + <b,H'> + <a,b> Q, compute
        //             P = <a,G> + <b',H> + <a,b> Q,
        // where b' = b \circ y^(-n)
        let b_prime = b.iter().zip(util::exp_iter(y_inv.clone())).map(|(bi, yi)| bi * yi);
        // a.iter() has Item=&Scalar, need Item=Scalar to chain with b_prime
        let a_prime = a.iter().cloned();

        let P = vartime_multiscalar_mul(
            a_prime.chain(b_prime).chain(iter::once(c)),
            G.iter().chain(H.iter()).chain(iter::once(&Q)),
        );

        let mut verifier = Transcript::new(b"innerproducttest");
        let proof = InnerProductProof::create(
            &mut verifier,
            &Q,
            &G_factors,
            &H_factors,
            G.clone(),
            H.clone(),
            a.clone(),
            b.clone(),
        );

        let mut verifier = Transcript::new(b"innerproducttest");
        assert!(proof
            .verify(
                n,
                &mut verifier,
                iter::repeat(Scalar::one()).take(n),
                util::exp_iter(y_inv).take(n),
                &P,
                &Q,
                &G,
                &H
            )
            .is_ok());
    }

    #[test]
    fn make_ipp_1() {
        test_helper_create(1);
    }

    #[test]
    fn make_ipp_2() {
        test_helper_create(2);
    }

    #[test]
    fn make_ipp_4() {
        test_helper_create(4);
    }

    #[test]
    fn make_ipp_32() {
        test_helper_create(32);
    }

    #[test]
    fn make_ipp_64() {
        test_helper_create(64);
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
}
