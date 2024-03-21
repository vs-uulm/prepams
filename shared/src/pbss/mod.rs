#![allow(non_snake_case)]
#![allow(warnings)]

use ed25519_zebra::VerificationKey;
use ff::Field;
use group::Curve;
use rand::RngCore;
use wasm_bindgen::prelude::*;

use serde_with::serde_as;
use sha2::{Digest, Sha256};
use merlin::Transcript;
use serde::{Serialize, Deserialize};
use bls12_381::{pairing, G1Affine, G1Projective, G2Affine, G2Projective, Gt, Scalar};
use bls12_381::hash_to_curve::{HashToCurve, ExpandMsgXmd};
use simple_error::SimpleError;

use crate::external::transcript::TranscriptProtocol;
use crate::types::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct SecretKey {
    #[serde(with = "crate::serialization::Scalar")]
    pub x: Scalar
}

type BlindingFactor = Scalar;

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct PublicKey {
    pub tag: String,

    #[serde_as(as = "Vec<crate::serialization::SerializableG1Affine>")]
    pub U: Vec<G1Affine>,
    #[serde_as(as = "Vec<crate::serialization::SerializableG1Affine>")]
    pub V: Vec<G1Affine>,
    #[serde(with = "crate::serialization::G1Affine")]
    pub h: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub b: G1Affine,

    #[serde(with = "crate::serialization::G1Affine")]
    pub g1: G1Affine,
    #[serde(with = "crate::serialization::G2Affine")]
    pub g2: G2Affine,
    #[serde(with = "crate::serialization::Gt")]
    pub e: Gt
}

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct BlindedSignRequest {
    #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
    pub m: Vec<Scalar>,
    #[serde(with = "crate::serialization::G1Affine")]
    pub alpha: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub gamma: G1Affine,
    #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
    pub z1: Vec<Scalar>,
    #[serde(with = "crate::serialization::Scalar")]
    pub z2: Scalar,
}

impl std::hash::Hash for BlindedSignRequest {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    for r in &self.m {
        r.to_bytes().hash(state);
    }
    self.alpha.to_compressed().hash(state);
    self.gamma.to_compressed().hash(state);
    for z in &self.z1 {
        z.to_bytes().hash(state);
    }
    self.z2.to_bytes().hash(state);
  }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct BlindedSignature {
    #[serde(with = "crate::serialization::G1Affine")]
    sigma1: G1Affine,
    #[serde(with = "crate::serialization::G2Affine")]
    sigma2: G2Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    sigma3: G1Affine
}

impl std::hash::Hash for BlindedSignature {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.sigma1.to_compressed().hash(state);
    self.sigma2.to_compressed().hash(state);
    self.sigma3.to_compressed().hash(state);
  }
}

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Default)]
pub struct UnblindedSignature {
    #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
    pub s: Vec<Scalar>,
    #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
    pub m: Vec<Scalar>,
    #[serde(with = "crate::serialization::G1Affine")]
    sigma1: G1Affine,
    #[serde(with = "crate::serialization::G2Affine")]
    sigma2: G2Affine
}

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct RerandomizedProof {
    #[serde(with = "crate::serialization::G2Affine")]
    pub s2: G2Affine,
    #[serde(with = "crate::serialization::Gt")]
    pub e1: Gt,
    #[serde(with = "crate::serialization::G1Affine")]
    pub bp: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub vc: G1Affine
}

impl RerandomizedProof {
    pub fn commit(&self, transcript: &mut Transcript) {
        transcript.append_g2(b"s2", &self.s2);
        transcript.append_gt(b"e1", &self.e1);
        transcript.append_g1(b"vc", &self.vc);
        transcript.append_g1(b"bp", &self.bp);
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RerandomizedWitness {
    pub s: UnblindedSignature,
    pub d: Scalar,
    pub r: Scalar,
    pub bb: Scalar,
    pub b1: Vec<Scalar>,
    pub b2: Vec<Scalar>,
    pub j1: G1Affine,
    pub s1: G1Affine
}

impl RerandomizedWitness {
    pub fn prove(&self, t: &mut Transcript) -> RerandomizedProofResponse {
        let c = t.challenge_scalar(b"c");

        // response
        let z1: Vec<Scalar> = self.s.s.iter().zip(self.b1.iter()).map(|(x, b)| b + &c * x).collect();
        let z2: Vec<Scalar> = self.s.m.iter().zip(self.b2.iter()).map(|(x, b)| b + &c * x).collect();
        let z3 = (&self.s1 * &c + &self.j1).to_affine();
        let zv = &self.bb + &c * &self.r;

        RerandomizedProofResponse { z1, z2, z3, zv }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct RerandomizedProofResponse {
    #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
    pub z1: Vec<Scalar>,
    #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
    pub z2: Vec<Scalar>,
    #[serde(with = "crate::serialization::G1Affine")]
    pub z3: G1Affine,
    #[serde(with = "crate::serialization::Scalar")]
    pub zv: Scalar
}

impl RerandomizedProofResponse {
    pub fn verify(&self, p: &RerandomizedProof, vk: &PublicKey, t: &mut Transcript) -> bool {
        let c = t.challenge_scalar(b"c");

        if vk.V.len() != self.z1.len() {
            return false;
        }

        if vk.U.len() != self.z2.len() {
            return false;
        }

        let tmp = vk.U.iter().zip(&self.z2).fold(
            vk.V.iter().zip(&self.z1).fold(
                G1Projective::identity(),
                |s, (v, z)| s + v * z.neg()
            ),
            |s, (u, z)| s + u * z.neg()
        );

        let l1 = p.e1 + (vk.e * &c) + (pairing(&vk.h, &p.s2) * &c);
        let r1 = pairing(&self.z3, &vk.g2) + pairing(&tmp.to_affine(), &p.s2);

        let l2 = p.vc * c + p.bp;
        let r2 = vk.U.iter().zip(&self.z2).fold(
            vk.V.iter().zip(&self.z1).fold(
                vk.b * self.zv,
                |s, (v, z)| s + v * z
            ),
            |s, (u, z)| s + u * z
        );

        l1 == r1 && l2 == r2
    }
}

pub fn Gen(mut rng: impl RngCore, secrets: usize, attributes: usize, tag: &str) -> (SecretKey, PublicKey) {
    let x = Scalar::random(&mut rng);

    let U: Vec<G1Affine> = (0..attributes).map(|i| <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<Sha256>>>::hash_to_curve((i as u32).to_be_bytes(), tag.as_bytes()).to_affine()).collect();
    let V: Vec<G1Affine> = (0..secrets).map(|i| <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<Sha256>>>::hash_to_curve(((i + attributes) as u32).to_be_bytes(), tag.as_bytes()).to_affine()).collect();
    let h: G1Affine = <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<Sha256>>>::hash_to_curve(((attributes + secrets + 0) as u32).to_be_bytes(), tag.as_bytes()).to_affine();
    let g1: G1Affine = <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<Sha256>>>::hash_to_curve(((attributes + secrets + 1) as u32).to_be_bytes(), tag.as_bytes()).to_affine();
    let g2: G2Affine = <bls12_381::G2Projective as HashToCurve<ExpandMsgXmd<Sha256>>>::hash_to_curve(((attributes + secrets + 2) as u32).to_be_bytes(), tag.as_bytes()).to_affine();
    let b: G1Affine = <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<Sha256>>>::hash_to_curve(((attributes + secrets + 3) as u32).to_be_bytes(), tag.as_bytes()).to_affine();

    let e = pairing(&g1, &g2) * x;

    let sk = SecretKey { x };
    let vk = PublicKey { tag: tag.to_string(), U, V, h, b, g1, g2, e };

    (sk, vk)
}

pub fn VerifyPublicKey(pk: &PublicKey) -> bool {
    let (_, vk) = Gen(rand::thread_rng(), pk.V.len(), pk.U.len(), &pk.tag);
    vk.U == pk.U && vk.V == pk.V && vk.h == pk.h && vk.g1 == pk.g1 && vk.g2 == pk.g2
}

pub fn Blind(vk: &PublicKey, m: &Vec<Scalar>, s: &Vec<Scalar>, d: &BlindingFactor, mut rng: impl RngCore) -> BlindedSignRequest {
    let m = m.clone();

    assert!(m.len() == vk.U.len());
    assert!(s.len() == vk.V.len());

    let alpha  = vk.V.iter()
        .zip(s.iter())
        .fold(
            vk.g1 * d,
            |a, (v, s)| a + v * s
        )
        .to_affine();

    let b1: Vec<Scalar> = vk.V.iter().map(|_| Scalar::random(&mut rng)).collect();
    let b2 = Scalar::random(&mut rng);

    let gamma  = vk.V.iter()
        .zip(b1.iter())
        .fold(
            vk.g1 * b2,
            |a, (v, b)| a + v * b
        )
        .to_affine();

    let mut t = Transcript::new(b"sign-request");
    t.append_message(b"tag", vk.tag.as_bytes());

    for (u, m) in vk.U.iter().zip(m.iter()) {
        t.append_g1(b"u", u);
        t.append_scalar(b"m", &m);
    }

    for v in &vk.V {
        t.append_g1(b"v", &v);
    }

    t.append_g1(b"h", &vk.h);
    t.append_g1(b"g1", &vk.g1);
    t.append_g2(b"g2", &vk.g2);
    t.append_g1(b"alpha", &alpha);
    t.append_g1(b"gamma", &gamma);

    let c = t.challenge_scalar(b"c");

    let z1: Vec<Scalar> = b1.iter()
        .zip(s.iter())
        .map(|(b, s)| b + c * s)
        .collect();

    let z2 = b2 + c * d;

    BlindedSignRequest { m, alpha, gamma, z1, z2 }
}

pub fn Sign(vk: &PublicKey, sk: &SecretKey, req: &BlindedSignRequest, mut rng: impl RngCore) -> Result<BlindedSignature, SimpleError> {
    let w = Scalar::random(&mut rng);
    let m = req.m.clone();

    if vk.U.len() != req.m.len() {
        Err(SimpleError::new("Invalid attributes supplied"))?;
    }

    if vk.V.len() != req.z1.len() {
        Err(SimpleError::new("Invalid responses supplied"))?;
    }

    let mut t = Transcript::new(b"sign-request");
    t.append_message(b"tag", vk.tag.as_bytes());

    for (u, m) in vk.U.iter().zip(m.iter()) {
        t.append_g1(b"u", u);
        t.append_scalar(b"m", &m);
    }

    for v in &vk.V {
        t.append_g1(b"v", &v);
    }

    t.append_g1(b"h", &vk.h);
    t.append_g1(b"g1", &vk.g1);
    t.append_g2(b"g2", &vk.g2);
    t.append_g1(b"alpha", &req.alpha);
    t.append_g1(b"gamma", &req.gamma);

    let c = t.challenge_scalar(b"c");

    let l1 = vk.V.iter()
        .zip(req.z1.iter())
        .fold(
            &vk.g1 * &req.z2,
            |a, (v, z)| a + v * z
        );

    let r1 = req.alpha * c + req.gamma;
    if l1.to_affine() != r1.to_affine() {
        Err(SimpleError::new("proof of knowledge failed"))?;
    }

    let sigma1 = (vk.g1 * sk.x + vk.U.iter().zip(req.m.iter()).fold(
        G1Projective::identity() + req.alpha + vk.h,
        |s, (u, m)| s + u * m
    ) * w).to_affine();
    let sigma2 = (vk.g2 * w).to_affine();
    let sigma3 = (vk.g1 * w).to_affine();

    Ok(BlindedSignature { sigma1, sigma2, sigma3 })
}

pub fn Unblind(vk: &PublicKey, sig: &BlindedSignature, m: &Vec<Scalar>, s: &Vec<Scalar>, d: &BlindingFactor) -> Result<UnblindedSignature, String> {
    let sigma1 = (sig.sigma1 - sig.sigma3 * d).to_affine();
    let sigma2 = sig.sigma2.clone();

    let m = m.clone();
    let s = s.clone();

    let res = UnblindedSignature { m, s, sigma1, sigma2 };

    Verify(&vk, &res)?;

    Ok(res)
} 

pub fn Verify(vk: &PublicKey, sig: &UnblindedSignature) -> Result<(), String> {
    let m = sig.m.clone();

    let lhs = pairing(&sig.sigma1, &vk.g2);

    let rhs = vk.e + pairing(&vk.U.iter().zip(sig.m.iter()).fold(
        vk.V.iter().zip(sig.s.iter()).fold(
            G1Projective::identity() + vk.h,
            |a, (v, s)| a + v * s
        ),
        |s, (u, m)| s + u * m
    ).to_affine(), &sig.sigma2);

    if lhs != rhs {
        Err(String::from("signature not valid"))?;
    }

    Ok(())
}

pub fn Rerandomize(s: &UnblindedSignature, vk: &PublicKey, mut rng: impl RngCore) -> (RerandomizedProof, RerandomizedWitness) {
    let d = Scalar::random(&mut rng);

    let u = vk.U.iter().zip(s.m.iter()).fold(
        G1Projective::identity(),
        |a, (u, m)| a + u * m
    );

    let v = vk.V.iter().zip(s.s.iter()).fold(
        G1Projective::identity(),
        |a, (v, s)| a + v * s
    );

    let r = Scalar::random(&mut rng);
    let vc = (u + v + &vk.b * r).to_affine();

    let bb = Scalar::random(&mut rng);
    let b1: Vec<Scalar> = s.s.iter().map(|_| Scalar::random(&mut rng)).collect();
    let b2: Vec<Scalar> = s.m.iter().map(|_| Scalar::random(&mut rng)).collect();

    let s1 = (s.sigma1 + (u + v + &vk.h) * &d).to_affine();
    let s2 = (s.sigma2 + vk.g2 * &d).to_affine();

    let vb = &vk.V.iter().zip(b1.iter()).fold(
        G1Projective::identity(),
        |a, (v, b)| a + v * b.neg()
    );

    let ub = &vk.U.iter().zip(b2.iter()).fold(
        G1Projective::identity(),
        |a, (u, b)| a + u * b.neg()
    );

    let bp = vk.U.iter().zip(b2.iter()).fold(
        vk.V.iter().zip(b1.iter()).fold(
            vk.b * bb,
            |a, (v, b)| a + v * b
        ),
        |a, (u, b)| a + u * b
    ).to_affine();

    let j1 = (vk.g1 * Scalar::random(&mut rng)).to_affine();
    let e1 = pairing(&j1, &vk.g2) + pairing(&(ub + vb).to_affine(), &s2);

    (
        RerandomizedProof { s2, e1, bp, vc },
        RerandomizedWitness {
            s: s.clone(),
            bb: bb,
            b1: b1,
            b2: b2,
            j1: j1,
            r: r,
            d: d,
            s1: s1
        }
    )
}


#[cfg(test)]
mod tests {
    use crate::external::transcript;

    use super::*;

    use ff::Field;
    use rand::Rng;
    use rand::thread_rng;
    use bls12_381::hash_to_curve::{HashToCurve, ExpandMsgXmd};

    #[test]
    fn KeyGeneration() {
        let mut csrng = thread_rng();
        let (_, pk) = Gen(&mut csrng, 1, 3, "test");
        assert!(VerifyPublicKey(&pk))
    }

    #[test]
    fn signature() {
        let mut csrng = thread_rng();
        let (sk, pk) = Gen(&mut csrng, 1, 3, "test");

        let s = vec![Scalar::random(&mut csrng)];
        let m = vec![Scalar::random(&mut csrng), Scalar::random(&mut csrng), Scalar::random(&mut csrng)];

        let d = Scalar::random(&mut csrng);

        let req = Blind(&pk, &m, &s, &d, &mut csrng);
        let blinded_sig = Sign(&pk, &sk, &req, &mut csrng).unwrap();
        let sig = Unblind(&pk, &blinded_sig, &m, &s, &d).unwrap();
    }

    #[test]
    fn prove() {
        let mut csrng = thread_rng();
        let (sk, pk) = Gen(&mut csrng, 2, 3, "test");

        let s = vec![Scalar::random(&mut csrng), Scalar::random(&mut csrng)];
        let m = vec![Scalar::random(&mut csrng), Scalar::random(&mut csrng), Scalar::random(&mut csrng)];

        let d = Scalar::random(&mut csrng);

        let req = Blind(&pk, &m, &s, &d, &mut csrng);
        let blinded_sig = Sign(&pk, &sk, &req, &mut csrng).unwrap();
        let sig = Unblind(&pk, &blinded_sig, &m, &s, &d).unwrap();

        let (p, w) = Rerandomize(&sig, &pk, &mut csrng);
        let mut prover_transcript = Transcript::new(b"prove_test");
        p.commit(&mut prover_transcript);
        let r = w.prove(&mut prover_transcript);

        let mut verifier_transcript = Transcript::new(b"prove_test");
        p.commit(&mut verifier_transcript);

        assert!(r.verify(&p, &pk, &mut verifier_transcript));
    }
}

