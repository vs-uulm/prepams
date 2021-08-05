use ff::Field;
use group::Curve;
use rand::RngCore;
use merlin::Transcript;
use bls12_381::{G1Affine, G2Affine, Scalar, pairing};

use crate::types::*;
use crate::serialization::Gt::SerializableGt;

#[allow(non_snake_case)]
pub fn CREDENTIAL_H() -> G1Affine {
    G1Affine::from_compressed(&[173, 39, 104, 242, 132, 56, 63, 100, 55, 159, 36, 9, 54, 64, 32, 8, 182, 68, 211, 33, 42, 93, 59, 158, 48, 128, 163, 144, 34, 214, 216, 240, 163, 31, 30, 148, 90, 45, 55, 159, 104, 153, 37, 252, 235, 227, 104, 165]).unwrap()
}

#[allow(non_snake_case)]
pub fn CREDENTIAL_V() -> G1Affine {
    G1Affine::from_compressed(&[165, 208, 239, 1, 166, 213, 242, 68, 66, 213, 198, 206, 243, 202, 206, 218, 69, 142, 131, 81, 144, 250, 241, 111, 155, 7, 73, 154, 114, 130, 32, 208, 154, 151, 234, 174, 196, 80, 75, 44, 69, 220, 91, 163, 192, 6, 27, 197]).unwrap()
}

pub fn init(rng: impl RngCore) -> (IssuerPublicKey, IssuerSecretKey) {
    let sk = Scalar::random(rng);
    let pk = pairing(&G1Affine::generator(), &G2Affine::generator()) * &sk;

    (IssuerPublicKey {pk}, IssuerSecretKey{sk})
}

pub fn issue_request(rng: impl RngCore, pk: &IssuerPublicKey, id: &str) -> (IssueRequest, Credential) {
    let mut rng = rng;
    let sk = Scalar::random(&mut rng);
    let d = Scalar::random(&mut rng);

    let v = CREDENTIAL_V();
    let h = CREDENTIAL_H();

    let alpha = (&v * &sk + G1Affine::generator() * &d).to_affine();

    let b1 = Scalar::random(&mut rng);
    let b2 = Scalar::random(&mut rng);

    let gamma = (&v * &b1 + G1Affine::generator() * &b2).to_affine();

    let mut t = Transcript::new(b"issue-request");
    t.append_message(b"pk", &pk.pk.to_bytes());
    t.append_message(b"v", &v.to_compressed());
    t.append_message(b"h", &h.to_compressed());
    t.append_message(b"id", id.as_bytes());
    t.append_message(b"alpha", &alpha.to_compressed());
    t.append_message(b"gamma", &gamma.to_compressed());

    let mut c = [0; 64];
    t.challenge_bytes(b"c", &mut c);
    let c = Scalar::from_bytes_wide(&c);

    let z1 = &b1 + &c * &sk;
    let z2 = &b2 + &c * &d;

    (
        IssueRequest {
            id: id.to_string(),
            alpha,
            gamma,
            z1,
            z2
        },
        Credential {
            sk,
            d,
            sigma_1: None,
            sigma_2: None,
            sigma_3: None
        }
    )
}

pub fn issue(pk: &IssuerPublicKey, sk: &IssuerSecretKey, request: &IssueRequest) -> Result<IssueResponse, String> {
    let v = CREDENTIAL_V();
    let h = CREDENTIAL_H();

    let mut t = Transcript::new(b"issue-request");
    t.append_message(b"pk", &pk.pk.to_bytes());
    t.append_message(b"v", &v.to_compressed());
    t.append_message(b"h", &h.to_compressed());
    t.append_message(b"id", &request.id.as_bytes());
    t.append_message(b"alpha", &request.alpha.to_compressed());
    t.append_message(b"gamma", &request.gamma.to_compressed());

    let mut c = [0; 64];
    t.challenge_bytes(b"c", &mut c);
    let c = Scalar::from_bytes_wide(&c);

    let l = &v * &request.z1 + G1Affine::generator() * &request.z2;
    let r = &request.alpha * c + &request.gamma;

    if l != r {
        Err(String::from("Verification failed"))
    } else {
        let r = Scalar::random(rand::thread_rng());

        let sigma_1 = (G1Affine::generator() * sk.sk + &request.alpha * &r + &h * &r).to_affine();
        let sigma_2 = (G1Affine::generator() * &r).to_affine();
        let sigma_3 = (G2Affine::generator() * &r).to_affine();

        Ok(IssueResponse {sigma_1, sigma_2, sigma_3})
    }
}

pub fn get_credential(pk: &IssuerPublicKey, response: &IssueResponse, credential: &mut Credential) -> Result<(), String> {
    let v = CREDENTIAL_V();
    let h = CREDENTIAL_H();

    let l = pairing(&response.sigma_1, &G2Affine::generator());
    let r = pairing(&(&v * &credential.sk + G1Affine::generator() * &credential.d + &h).to_affine(), &response.sigma_3) + pk.pk;

    if l != r {
        Err(String::from("Verification failed"))
    } else {
        credential.sigma_1 = Some((&response.sigma_1 + &response.sigma_2 * &credential.d.neg()).to_affine());
        credential.sigma_2 = Some(response.sigma_2.clone());
        credential.sigma_3 = Some(response.sigma_3.clone());

        Ok(())
    }
}

pub fn authenticate(credential: &Credential, id: &ResourceIdentifier) -> AuthenticationRequest {
    let v = CREDENTIAL_V();
    let h = CREDENTIAL_H();

    let token = (G1Affine::generator() * (&credential.sk + id.id).invert().unwrap()).to_affine();
    let d = Scalar::random(rand::thread_rng());
    let s1 = (credential.sigma_1.as_ref().unwrap() + (&v * &credential.sk + &h) * &d).to_affine();
    let s2 = (credential.sigma_3.as_ref().unwrap() + G2Affine::generator() * &d).to_affine();

    // proof of knowledge
    let b = Scalar::random(rand::thread_rng());
    let j = (G1Affine::generator() * Scalar::random(rand::thread_rng())).to_affine();

    let e1 = pairing(&j, &G2Affine::generator()) + pairing(&(&v * &b.neg()).to_affine(), &s2);
    let e2 = pairing(&token, &(G2Affine::generator() * &b).to_affine());

    // challenge
    let mut t = Transcript::new(b"authentication");
    t.append_message(b"id", &id.id.to_bytes());
    t.append_message(b"token", &token.to_compressed());
    t.append_message(b"s1", &s1.to_compressed());
    t.append_message(b"s2", &s2.to_compressed());
    t.append_message(b"e1", &e1.to_bytes());
    t.append_message(b"e2", &e2.to_bytes());

    let mut c = [0; 64];
    t.challenge_bytes(b"c", &mut c);
    let c = Scalar::from_bytes_wide(&c);

    // response
    let z1 = &b + &c * &credential.sk;
    let z2 = (&s1 * &c + &j).to_affine();

    AuthenticationRequest {
        id: id.clone(),
        token: token,
        s1: s1,
        s2: s2,
        e1: e1,
        e2: e2,
        z1: z1,
        z2: z2
    }
}

pub fn verify(pk: &IssuerPublicKey, request: &AuthenticationRequest) -> bool {
    let v = CREDENTIAL_V();
    let h = CREDENTIAL_H();

    // challenge
    let mut t = Transcript::new(b"authentication");
    t.append_message(b"id", &request.id.id.to_bytes());
    t.append_message(b"token", &request.token.to_compressed());
    t.append_message(b"s1", &request.s1.to_compressed());
    t.append_message(b"s2", &request.s2.to_compressed());
    t.append_message(b"e1", &request.e1.to_bytes());
    t.append_message(b"e2", &request.e2.to_bytes());

    let mut c = [0; 64];
    t.challenge_bytes(b"c", &mut c);
    let c = Scalar::from_bytes_wide(&c);

    let l1 = &request.e1 + (pk.pk * &c) + (pairing(&h, &request.s2) * &c);
    let r1 = pairing(&request.z2, &G2Affine::generator()) + pairing(&(&v * &request.z1.neg()).to_affine(), &request.s2);

    let l2 = &request.e2 + pairing(&G1Affine::generator(), &G2Affine::generator()) * &c + pairing(&request.token, &G2Affine::generator()) * (&c * &request.id.id).neg();
    let r2 = pairing(&request.token, &G2Affine::generator()) * &request.z1;

    l1 == r1 && l2 == r2
}

#[cfg(test)]
mod tests {
    use super::*;

    use bls12_381::hash_to_curve::{HashToCurve, ExpandMsgXmd};

    #[test]
    fn credential() {
        let mut rng = rand::thread_rng();

        let identity = "user@example.com";
        let (ipk, isk) = init(&mut rng);
        let ipk : IssuerPublicKey = postcard::from_bytes(&postcard::to_allocvec(&ipk).unwrap()).unwrap();
        let (request, mut credential) = issue_request(&mut rng, &ipk, &identity);
        let response = issue(&ipk, &isk, &request).unwrap();
        get_credential(&ipk, &response, &mut credential).unwrap();

        let resource = ResourceIdentifier::new();
        let request = authenticate(&credential, &resource);

        assert!(verify(&ipk, &request));
    }

    #[test]
    #[allow(non_snake_case)]
    fn check_H() {
        let H = <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(b"credential_H", b"credential_H").to_affine();
        assert_eq!(H, CREDENTIAL_H());
    }

    #[test]
    #[allow(non_snake_case)]
    fn check_V() {
        let V = <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(b"credential_V", b"credential_V").to_affine();
        assert_eq!(V, CREDENTIAL_V());
    }
}
