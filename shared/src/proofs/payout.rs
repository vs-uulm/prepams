#![allow(non_snake_case)]
use group::Curve;

use rand::seq::SliceRandom;

use serde::{Serialize, Deserialize};
use serde_with::serde_as;
use bls12_381::{G1Affine, Scalar};
use sha2::{Digest, Sha512};

use crate::types::credential::IssuerPublicKey;
use crate::pbss::{UnblindedSignature, PublicKey, Rerandomize, RerandomizedProof, RerandomizedWitness, RerandomizedProofResponse};
use crate::external::transcript::TranscriptProtocol;
use crate::proofs::generic::{Proof, Variables, Variable, Constraint, ConstraintType, ProofInput, Transcript};
use crate::external::util::{as_scalar, as_u32, sum_of_powers, exp_iter};

pub const MAX_INPUTS: usize = 10;

pub fn BINDING_G() -> G1Affine {
    G1Affine::from_compressed(&[182, 75, 166, 124, 162, 220, 249, 19, 0, 228, 164, 54, 26, 219, 4, 21, 221, 179, 19, 116, 142, 11, 175, 115, 205, 12, 241, 225, 22, 216, 143, 92, 70, 173, 178, 79, 50, 132, 88, 209, 56, 91, 91, 13, 43, 174, 117, 131]).unwrap()
}

pub fn PAYOUT_G1() -> G1Affine {
    G1Affine::from_compressed(&[185, 109, 132, 30, 129, 70, 218, 218, 250, 86, 44, 200, 115, 205, 234, 173, 238, 136, 86, 196, 46, 247, 171, 209, 59, 158, 246, 168, 17, 41, 242, 116, 187, 108, 182, 68, 214, 31, 66, 220, 240, 65, 39, 85, 146, 208, 220, 48]).unwrap()
}

pub fn PAYOUT_V0() -> G1Affine {
    G1Affine::from_compressed(&[183, 63, 201, 109, 80, 238, 199, 88, 108, 56, 60, 45, 14, 199, 182, 69, 12, 49, 120, 1, 66, 13, 148, 65, 38, 35, 63, 9, 219, 56, 70, 58, 76, 78, 153, 203, 160, 149, 201, 147, 149, 107, 57, 90, 94, 166, 88, 189]).unwrap()
}

pub fn PAYOUT_V1() -> G1Affine {
    G1Affine::from_compressed(&[132, 148, 112, 4, 19, 245, 186, 172, 88, 36, 203, 142, 222, 14, 239, 104, 118, 187, 114, 132, 92, 177, 149, 205, 44, 211, 51, 163, 26, 155, 14, 62, 134, 25, 231, 249, 114, 49, 47, 47, 203, 170, 208, 203, 22, 0, 234, 31]).unwrap()
}

pub struct PayoutProof {}

#[derive(Default)]
pub struct PayoutProofSecrets {
    witnesses: Vec<RerandomizedWitness>
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct PayoutProofInput {
    pub value: u8,
    pub target: String,
    pub recipient: String,
    pub ivk: IssuerPublicKey,
    pub cvk: PublicKey,
    pub inputs: Vec<RerandomizedProof>,
    #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
    pub nullifier: Vec<Scalar>,
}

impl ProofInput for PayoutProofInput {
    fn commit(&self, transcript: &mut Transcript) {
        transcript.append_byte(b"value", self.value);
        transcript.append_message(b"target", self.target.as_bytes());
        transcript.append_message(b"recipient", self.recipient.as_bytes());
        for input in &self.inputs {
            input.commit(transcript);
        }
        for nullifier in &self.nullifier {
            transcript.append_scalar(b"n", &nullifier);
        }
    }
}

impl PayoutProofInput {
    pub fn new(ivk: &IssuerPublicKey, cvk: &PublicKey, value: u8, target: &str, recipient: &str, spend: Vec<UnblindedSignature>, nulls: Vec<UnblindedSignature>) -> (PayoutProofInput, PayoutProofSecrets) {
        let mut rng = rand::thread_rng();
        let mut sample = spend;
        let mut nulls = nulls;

        // shuffle inputs
        sample.shuffle(&mut rng);
        // shuffle nulls
        nulls.shuffle(&mut rng);

        // append necessary amount of nulls to sample
        sample.append(&mut nulls);
        sample.truncate(MAX_INPUTS);

        // shuffle final sample
        sample.shuffle(&mut rng);

        // rerandomize coin signatures
        let (inputs, witnesses): (Vec<RerandomizedProof>, Vec<RerandomizedWitness>) = sample.iter()
            .map(|e| Rerandomize(e, &cvk, &mut rng))
            .unzip();

        let nullifier = witnesses.iter().map(|e| e.s.s[0].clone()).collect();

        (
            PayoutProofInput {
                value: value,
                target: target.to_string(),
                recipient: recipient.to_string(),
                ivk: ivk.clone(),
                cvk: cvk.clone(),
                inputs: inputs,
                nullifier: nullifier
            },
            PayoutProofSecrets {
                witnesses: witnesses
            }
        )
    }
}

impl Proof<PayoutProofInput, PayoutProofSecrets, Vec<RerandomizedProofResponse>> for PayoutProof {
    fn get_variables(inputs: &PayoutProofInput, secrets: &PayoutProofSecrets, u: &Scalar) -> Variables {
        let mut vars = Variables::new();

        let mut sep = Scalar::one();

        for (i, input) in inputs.inputs.iter().enumerate() {
            vars.add(Variable::Inner {
                id: format!("input_{}", i),
                G: (input.vc * &sep).to_affine(),
                cl: -Scalar::one(),
                cr: Scalar::zero()
            });
            vars.add(Variable::Inner {
                id: format!("input_{}_opening", i),
                G: (inputs.cvk.b * &sep).to_affine(),
                cl: secrets.witnesses.get(i).map_or(Scalar::zero(), |w| w.r.clone()),
                cr: Scalar::zero()
            });
            let witness = secrets.witnesses.get(i).map_or(UnblindedSignature::default(), |w| w.s.clone());
            for (j, u) in inputs.cvk.U.iter().enumerate() {
                vars.add(Variable::Inner {
                    id: format!("input_{}_u_{}", i, j),
                    G: (u * &sep).to_affine(),
                    cl: witness.m.get(j).map_or(Scalar::zero(), |v| v.clone()),
                    cr: Scalar::zero()
                });
            }
            for (j, v) in inputs.cvk.V.iter().enumerate() {
                vars.add(Variable::Inner {
                    id: format!("input_{}_v_{}", i, j),
                    G: (v * &sep).to_affine(),
                    cl: witness.s.get(j).map_or(Scalar::zero(), |v| v.clone()),
                    cr: Scalar::zero()
                });
            }

            sep = &sep * u;
        }

        let sum = secrets.witnesses.iter().fold(0, |acc, w| acc + as_u32(&w.s.m[0]));
        let remainder = match sum > 0 {
            true => sum - (inputs.value as u32),
            _ => 0
        } as u8;

        for i in 0..8 {
            let bit = as_scalar(((remainder & u8::pow(2, i)) >> i) as u32);
            vars.add(Variable::Scratch {
                id: format!("bit_{}", i),
                cl: bit,
                cr: bit - Scalar::one()
            });
        }

        vars
    }

    fn get_constraints(inputs: &PayoutProofInput, y: &Scalar) -> Vec<Constraint> {
        let mut hasher = Sha512::new();
        hasher.update(b"id");
        hasher.update(inputs.recipient.as_bytes());
        let hash: [u8;64] = hasher.finalize().into();
        let identity = Scalar::from_bytes_wide(&hash);

        // identity of all tx matches payout target
        let mut a = Constraint::new(
            ConstraintType::Dir,
            exp_iter(y.clone()).take(MAX_INPUTS).map(|v| v * identity).sum()
        );
        a.left_set("auth_identity", -sum_of_powers(y, MAX_INPUTS));
        let mut sep = Scalar::one();

        // show validity of nullifier
        let mut b = Constraint::new(
            ConstraintType::Dir,
            inputs.nullifier.iter().zip(exp_iter(y.clone())).map(|(v, a)| v * a).sum()
        );

        for i in 0..MAX_INPUTS {
            a.left_set(&format!("input_{}_v_1", i), sep);
            b.left_set(&format!("input_{}_v_0", i), sep);
            sep = sep * y;
        }

        // sum of all tx and remainder has to equal payout value
        let mut c = Constraint::new(ConstraintType::Dir, as_scalar(inputs.value as u32));

        // show that remainder bits are binary
        let mut d = Constraint::new(ConstraintType::Mul, Scalar::zero());
        let mut e = Constraint::new(ConstraintType::One, Scalar::zero());

        for i in 0..MAX_INPUTS {
            c.left_set(&format!("input_{}_u_0", i), Scalar::one());
        }

        let mut sep_binary = Scalar::one();

        for i in 0..8 {
            c.left_set(&format!("bit_{}", i), -as_scalar(u32::pow(2, i)));
            d.right_set(&format!("bit_{}", i), sep_binary.clone());
            e.left_set(&format!("bit_{}", i), sep_binary.clone());
            sep_binary = &sep_binary * y;
        }

        vec![a, b, c, d, e]
    }

    fn additional_data(_inputs: &PayoutProofInput, secrets: &PayoutProofSecrets, transcript: &mut Transcript) -> Vec<RerandomizedProofResponse> {
        secrets.witnesses.iter().map(|w| w.prove(transcript)).collect()
    }

    fn additional_checks(inputs: &PayoutProofInput, data: &Vec<RerandomizedProofResponse>, transcript: &mut Transcript) -> bool {
        if inputs.inputs.len() != data.len() {
            return false;
        }

        inputs.inputs.iter()
            .zip(data.iter())
            .all(|(proof, response)| response.verify(proof, &inputs.cvk, transcript))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external::util::as_scalar;
    use crate::pbss;
    use crate::types::NullRequest;
    use crate::proofs::generic::GenericProof;
    use crate::external::util::{rand_scalar, assert_point, assert_generators};
    use crate::credential::{init, issue_request, issue, get_credential};

    use rand::thread_rng;
    use merlin::Transcript;

    #[test]
    #[allow(non_snake_case)]
    fn generators() {
        let (_csk, cvk) = pbss::Gen(&mut thread_rng(), 2, 1, "payment");

        assert_generators("payout", std::collections::HashMap::from([
            ("G", BINDING_G())
        ]));

        assert_point(&[
            ("G1", cvk.g1, PAYOUT_G1()),
            ("V0", cvk.V[0], PAYOUT_V0()),
            ("V1", cvk.V[1], PAYOUT_V1()),
        ]);
    }

    #[test]
    fn proof() {
        let mut csrng = thread_rng();
        let (csk, cvk) = pbss::Gen(&mut csrng, 2, 1, "payment");

        // generate test data
        let identity = "user@example.com";
        let (ipk, isk) = init(&mut csrng, 0);
        let attrs = vec![];
        let (request, mut credential) = issue_request(&mut csrng, &ipk, &identity, attrs);
        let response = issue(&ipk, &isk, &request).unwrap();
        get_credential(&ipk, &response, &mut credential).unwrap();

        let resources: Vec<(Scalar, u8)> = vec![
            (rand_scalar(), 5),
            (rand_scalar(), 25),
            (rand_scalar(), 2),
            (rand_scalar(), 10),
        ];

        let inputs: Vec<UnblindedSignature> = vec![resources[3], resources[1]].iter()
            .map(|(id, v)| {
                let mut rng = credential.derive_reward_rng(&id);
                let s = vec![<bls12_381::Scalar as ff::Field>::random(&mut rng), credential.identity];
                let d = <bls12_381::Scalar as ff::Field>::random(&mut rng);
                let m = vec![as_scalar(*v as u32)];
                let req = pbss::Blind(&cvk, &m, &s, &d, &mut rng);
                let sig = pbss::Sign(&cvk, &csk, &req, &mut csrng).unwrap();
                pbss::Unblind(&cvk, &sig, &m, &s, &d).unwrap()
            })
            .collect();

        let value = 30;

        let nr = NullRequest::new(&cvk, &credential);
        let m = vec![Scalar::zero()];
        let nulls: Vec<UnblindedSignature> = nr.request.iter().enumerate()
            .map(|(i, req)| {
                let sig = pbss::Sign(&cvk, &csk, &req, &mut csrng).unwrap();
                let s = vec![nr.s[2 * i], nr.s[2 * i + 1]];
                let d = nr.d[i];
                pbss::Unblind(&cvk, &sig, &m, &s, &d).unwrap()
            })
            .collect();

        let (inputs, secrets) = PayoutProofInput::new(
            &ipk,
            &cvk,
            value,
            "test",
            "user@example.com",
            inputs,
            nulls
        );

        // create proof
        let mut prover_transcript = Transcript::new(b"payout test");
        let proof = GenericProof::<PayoutProofInput, Vec<RerandomizedProofResponse>>::proove::<PayoutProofSecrets, PayoutProof>(&mut prover_transcript, inputs, secrets).unwrap();

        // verify proof
        let mut verifier_transcript = Transcript::new(b"payout test");
        let s = proof.verify::<PayoutProofSecrets, PayoutProof>(&mut verifier_transcript);

        assert!(s.is_ok());
    }
}
