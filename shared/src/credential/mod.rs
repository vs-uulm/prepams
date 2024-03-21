use ff::Field;
use simple_error::SimpleError;
use std::ops::Neg;
use group::Curve;
use rand::RngCore;
use merlin::Transcript;
use bls12_381::{G1Affine, G2Affine, Scalar, pairing};
use bls12_381::hash_to_curve::{HashToCurve, ExpandMsgXmd};
use sha2::{Digest, Sha512};

use crate::types::credential::{AuthenticationRequest, Credential,IssueRequest, IssueResponse, IssuerPublicKey, IssuerSecretKey};
use crate::external::transcript::TranscriptProtocol;

#[allow(non_snake_case)]
pub fn CREDENTIAL_H() -> G1Affine {
    G1Affine::from_compressed(&[173, 39, 104, 242, 132, 56, 63, 100, 55, 159, 36, 9, 54, 64, 32, 8, 182, 68, 211, 33, 42, 93, 59, 158, 48, 128, 163, 144, 34, 214, 216, 240, 163, 31, 30, 148, 90, 45, 55, 159, 104, 153, 37, 252, 235, 227, 104, 165]).unwrap()
}

#[allow(non_snake_case)]
pub fn CREDENTIAL_V() -> G1Affine {
    G1Affine::from_compressed(&[165, 208, 239, 1, 166, 213, 242, 68, 66, 213, 198, 206, 243, 202, 206, 218, 69, 142, 131, 81, 144, 250, 241, 111, 155, 7, 73, 154, 114, 130, 32, 208, 154, 151, 234, 174, 196, 80, 75, 44, 69, 220, 91, 163, 192, 6, 27, 197]).unwrap()
}

#[allow(non_snake_case)]
pub fn CREDENTIAL_I() -> G1Affine {
    G1Affine::from_compressed(&[151, 76, 213, 240, 120, 154, 108, 6, 251, 180, 24, 213, 121, 69, 112, 125, 243, 66, 86, 23, 235, 72, 181, 62, 211, 44, 18, 116, 165, 144, 75, 204, 125, 59, 169, 139, 53, 7, 9, 41, 174, 12, 221, 69, 57, 236, 18, 125]).unwrap()
}

#[allow(non_snake_case)]
pub fn BINDING_G() -> G1Affine {
    G1Affine::from_compressed(&[137, 179, 142, 119, 0, 117, 198, 112, 161, 144, 244, 121, 238, 137, 146, 174, 168, 52, 175, 13, 243, 130, 119, 106, 120, 73, 178, 201, 79, 108, 162, 118, 249, 82, 58, 61, 113, 201, 168, 211, 109, 109, 71, 166, 55, 5, 66, 182]).unwrap()
}

pub fn init(rng: impl RngCore, attributes: usize) -> (IssuerPublicKey, IssuerSecretKey) {
    let sk = Scalar::random(rng);
    let pk = pairing(&G1Affine::generator(), &G2Affine::generator()) * &sk;
    let a = (0..(attributes)).map(|i| <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve((i as u32).to_be_bytes(), b"attr").to_affine()).collect();

    (IssuerPublicKey {pk, a}, IssuerSecretKey{sk})
}

pub fn issue_request(rng: impl RngCore, pk: &IssuerPublicKey, id: &str, attributes: Vec<Scalar>) -> (IssueRequest, Credential) {
    let mut rng = rng;
    let sk = Scalar::random(&mut rng);
    let d = Scalar::random(&mut rng);

    let mut hasher = Sha512::new();
    hasher.update(b"id");
    hasher.update(id.as_bytes());
    let hash: [u8;64] = hasher.finalize().into();
    let identity = Scalar::from_bytes_wide(&hash);

    assert!(attributes.len() == pk.a.len());

    let v = CREDENTIAL_V();
    let h = CREDENTIAL_H();
    let i = CREDENTIAL_I();

    let alpha = (&v * &sk + G1Affine::generator() * &d).to_affine();

    let b1 = Scalar::random(&mut rng);
    let b2 = Scalar::random(&mut rng);

    let gamma = (&v * &b1 + G1Affine::generator() * &b2).to_affine();

    let mut t = Transcript::new(b"issue-request");
    t.append_gt(b"pk", &pk.pk);
    t.append_g1(b"v", &v);
    t.append_g1(b"h", &h);
    t.append_message(b"id", id.as_bytes());
    t.append_g1(b"alpha", &alpha);
    t.append_g1(b"gamma", &gamma);

    t.append_g1(b"i", &i);
    t.append_scalar(b"identity", &identity);

    for (attr, val) in pk.a.iter().zip(&attributes) {
        t.append_g1(b"attr", attr);
        t.append_scalar(b"val", &val);
    }

    let c = t.challenge_scalar(b"c");

    let z1 = &b1 + &c * &sk;
    let z2 = &b2 + &c * &d;

    (
        IssueRequest {
            id: id.to_string(),
            alpha,
            gamma,
            z1,
            z2,
            attributes: attributes.clone()
        },
        Credential {
            sk,
            d,
            id: id.to_string(),
            identity: identity.clone(),
            sigma_1: None,
            sigma_2: None,
            sigma_3: None,
            attributes: pk.a.clone(),
            values: attributes
        }
    )
}

pub fn issue(pk: &IssuerPublicKey, sk: &IssuerSecretKey, request: &IssueRequest) -> Result<IssueResponse, SimpleError> {
    let v = CREDENTIAL_V();
    let h = CREDENTIAL_H();
    let i = CREDENTIAL_I();

    if pk.a.len() != request.attributes.len() {
        Err(SimpleError::new("Invalid attributes supplied"))?;
    }

    let mut hasher = Sha512::new();
    hasher.update(b"id");
    hasher.update(request.id.as_bytes());
    let hash: [u8;64] = hasher.finalize().into();
    let identity = Scalar::from_bytes_wide(&hash);

    let mut t = Transcript::new(b"issue-request");
    t.append_gt(b"pk", &pk.pk);
    t.append_g1(b"v", &v);
    t.append_g1(b"h", &h);
    t.append_message(b"id", request.id.as_bytes());
    t.append_g1(b"alpha", &request.alpha);
    t.append_g1(b"gamma", &request.gamma);

    t.append_g1(b"i", &i);
    t.append_scalar(b"identity", &identity);

    for (attr, val) in pk.a.iter().zip(&request.attributes) {
        t.append_g1(b"attr", attr);
        t.append_scalar(b"val", val);
    }

    let c = t.challenge_scalar(b"c");

    let l = &v * &request.z1 + G1Affine::generator() * &request.z2;
    let r = &request.alpha * c + &request.gamma;

    if l != r {
        Err(SimpleError::new("Verification failed"))
    } else {
        let r = Scalar::random(rand::thread_rng());

        let tmp = pk.a.iter().zip(&request.attributes).fold(
            &i * &identity + &request.alpha + &h,
            |s, (g, e)| s + g * e
        );

        let sigma_1 = (G1Affine::generator() * sk.sk + tmp * &r).to_affine();
        let sigma_2 = (G1Affine::generator() * &r).to_affine();
        let sigma_3 = (G2Affine::generator() * &r).to_affine();

        Ok(IssueResponse {sigma_1, sigma_2, sigma_3})
    }
}

pub fn get_credential(pk: &IssuerPublicKey, response: &IssueResponse, credential: &mut Credential) -> Result<(), SimpleError> {
    let v = CREDENTIAL_V();
    let h = CREDENTIAL_H();
    let i = CREDENTIAL_I();

    let tmp = pk.a.iter().zip(&credential.values).fold(
        &v * &credential.sk + G1Affine::generator() * &credential.d + &h + &i * &credential.identity,
        |s, (g, e)| s + g * e
    ).to_affine();

    let l = pairing(&response.sigma_1, &G2Affine::generator());
    let r = pairing(&tmp, &response.sigma_3) + pk.pk;

    if l != r {
        Err(SimpleError::new("Verification failed"))
    } else {
        credential.sigma_1 = Some((&response.sigma_1 + &response.sigma_2 * &credential.d.neg()).to_affine());
        credential.sigma_2 = Some(response.sigma_2.clone());
        credential.sigma_3 = Some(response.sigma_3.clone());

        Ok(())
    }
}

pub fn authenticate(credential: &Credential, id: &Scalar) -> (AuthenticationRequest, (Scalar, G1Affine)) {
    let mut rng = rand::thread_rng();

    let v = CREDENTIAL_V();
    let h = CREDENTIAL_H();
    let i = CREDENTIAL_I();

    let token = (G1Affine::generator() * (&credential.sk + id).invert().unwrap()).to_affine();
    let d = Scalar::random(&mut rng);

    let u = &credential.attributes.iter().zip(&credential.values).fold(
        i * &credential.identity,
        |s, (g, e)| s + g * e
    );

    // vector commitment
    let g = BINDING_G();
    let r = Scalar::random(&mut rng);
    let vc = credential.attributes.iter().zip(credential.values.iter()).fold(
        g * r + i * &credential.identity,
        |s, (u, v)| s + u * v
    ).to_affine();

    let s1 = (credential.sigma_1.as_ref().unwrap() + (u + &v * &credential.sk + &h) * &d).to_affine();
    let s2 = (credential.sigma_3.as_ref().unwrap() + G2Affine::generator() * &d).to_affine();

    // proof of knowledge
    let b1 = Scalar::random(&mut rng);
    let b2 = Scalar::random(&mut rng);
    let bu: Vec<Scalar> = credential.attributes.iter().map(|_| Scalar::random(&mut rng)).collect();
    let j1 = (G1Affine::generator() * Scalar::random(&mut rng)).to_affine();

    let ub = &credential.attributes.iter().zip(bu.iter()).fold(
        i * b2.neg(),
        |s, (u, b)| s + u * b.neg()
    );

    let s = Scalar::random(&mut rng);
    let bp = &credential.attributes.iter().zip(bu.iter()).fold(
        BINDING_G() * s + i * b2,
        |s, (u, b)| s + u * b
    ).to_affine();

    let e1 = pairing(&j1, &G2Affine::generator()) + pairing(&(ub + &v * &b1.neg()).to_affine(), &s2);
    let e2 = pairing(&token, &(G2Affine::generator() * &b1).to_affine());

    // challenge
    let mut t = Transcript::new(b"authentication");
    t.append_scalar(b"id", &id);
    t.append_g1(b"token", &token);
    t.append_g2(b"s2", &s2);
    t.append_gt(b"e1", &e1);
    t.append_gt(b"e2", &e2);
    t.append_g1(b"vc", &vc);
    t.append_g1(b"bp", &bp);

    let c = t.challenge_scalar(b"c");

    // response
    let z1 = &b1 + &c * &credential.sk;
    let z2 = &b2 + &c * &credential.identity;
    let za: Vec<Scalar> = credential.values.iter().zip(bu.iter()).map(|(x, b)| b + &c * x).collect();
    let z3 = (&s1 * &c + &j1).to_affine();
    let zv = &s + &c * &r;

    (AuthenticationRequest {
        id: id.clone(),
        token: token,
        s2: s2,
        e1: e1,
        e2: e2,
        z1: z1,
        z2: z2,
        za: za,
        z3: z3,
        vc: vc.clone(),
        bp: bp.clone(),
        zv: zv
    }, (r, vc))
}

pub fn verify(pk: &IssuerPublicKey, request: &AuthenticationRequest) -> bool {
    let v = CREDENTIAL_V();
    let h = CREDENTIAL_H();
    let i = CREDENTIAL_I();

    if pk.a.len() != request.za.len() {
        return false;
    }

    // challenge
    let mut t = Transcript::new(b"authentication");
    t.append_scalar(b"id", &request.id);
    t.append_g1(b"token", &request.token);
    t.append_g2(b"s2", &request.s2);
    t.append_gt(b"e1", &request.e1);
    t.append_gt(b"e2", &request.e2);
    t.append_g1(b"vc", &request.vc);
    t.append_g1(b"bp", &request.bp);

    let c = t.challenge_scalar(b"c");

    let tmp = pk.a.iter().zip(&request.za).fold(
        &v * &request.z1.neg() + &i * &request.z2.neg(),
        |s, (u, z)| s + u * z.neg()
    );

    let l1 = &request.e1 + (pk.pk * &c) + pairing(&(h * c).to_affine(), &request.s2);
    let r1 = pairing(&request.z3, &G2Affine::generator()) + pairing(&tmp.to_affine(), &request.s2);

    let l2 = &request.e2 + pairing(&(G1Affine::generator() * c).to_affine(), &G2Affine::generator()) + pairing(&(request.token * (c * request.id).neg()).to_affine(), &G2Affine::generator());
    let r2 = pairing(&(request.token * request.z1).to_affine(), &G2Affine::generator());

    let l3 = request.vc * c + request.bp;
    let r3 = pk.a.iter().zip(&request.za).fold(
        BINDING_G() * request.zv + i * request.z2,
        |s, (u, z)| s + u * z
    );

    l1 == r1 && l2 == r2 && l3 == r3
}

#[cfg(test)]
mod tests {
    use crate::external::util::assert_generators;

    use super::*;

    #[test]
    fn credential() {
        let mut rng = rand::thread_rng();

        let identity = "user@example.com";
        let (ipk, isk) = init(&mut rng, 3);

        let attrs: Vec<Scalar> = ipk.a.iter().map(|_| Scalar::random(&mut rng)).collect();
        let (request, mut credential) = issue_request(&mut rng, &ipk, &identity, attrs.clone());
        let response = issue(&ipk, &isk, &request).unwrap();
        get_credential(&ipk, &response, &mut credential).unwrap();

        let resource = Scalar::random(rng);
        let (request, _) = authenticate(&credential, &resource);

        assert!(verify(&ipk, &request));
    }

    #[test]
    #[allow(non_snake_case)]
    fn generators() {
        assert_generators("credential", std::collections::HashMap::from([
            ("H", CREDENTIAL_H()),
            ("V", CREDENTIAL_V()),
            ("I", CREDENTIAL_I()),
            ("G'", BINDING_G()),
        ]));
    }
}
