#![allow(non_snake_case)]
use group::Curve;

use rand::seq::SliceRandom;

use sha3::{Digest, Sha3_256};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use serde::{Serialize, Deserialize};
use bls12_381::{G1Affine, Scalar};

use crate::types::*;
use crate::external::transcript::TranscriptProtocol;
use crate::zksnark::{Proof, Variables, Variable, Constraint, ConstraintType, ProofInput, Transcript};
use crate::external::util::sum_of_powers;

pub const MAX_INPUTS: usize = 10;
pub const ANONYMITY_SET: usize = 250;

pub fn PAYOUT_H() -> G1Affine {
    G1Affine::from_compressed(&[167, 202, 135, 255, 41, 15, 160, 185, 209, 167, 35, 219, 193, 203, 34, 119, 226, 16, 171, 181, 111, 37, 163, 50, 173, 225, 253, 4, 149, 32, 49, 78, 211, 82, 94, 55, 3, 55, 228, 102, 120, 57, 89, 217, 204, 111, 4, 74]).unwrap()
}

pub fn PAYOUT_A() -> G1Affine {
    G1Affine::from_compressed(&[133, 27, 116, 115, 193, 243, 11, 158, 100, 5, 152, 74, 154, 246, 37, 43, 159, 20, 148, 59, 152, 250, 113, 68, 29, 250, 3, 41, 94, 184, 122, 75, 186, 168, 96, 53, 80, 229, 124, 231, 31, 128, 15, 145, 238, 57, 149, 10]).unwrap()
}

pub fn derive_reward_key(credential: &Credential, id: &ResourceIdentifier) -> Scalar {
    let mut hasher = Sha3_256::new();
    hasher.update(b"reward");
    hasher.update(&credential.sk.to_bytes());
    hasher.update(&id.id.to_bytes());

    let seed = hasher.finalize().into();
    let mut rng = ChaCha20Rng::from_seed(seed);
    <bls12_381::Scalar as ff::Field>::random(&mut rng)
}

pub struct PayoutProof {}

pub struct PayoutProofSecrets {
    credential: Credential,
    spend: Vec<Transaction>
}

impl Default for PayoutProofSecrets {
    fn default() -> Self {
        Self {
            credential: Credential::default(),
            spend: (0..MAX_INPUTS).map(|_| Transaction::default()).collect()
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PayoutProofInput {
    pub value: u8,
    pub target: String,
    pub recipient: String,
    pub transactions: Vec<Transaction>,
    pub nullifier: Vec<Tag>
}

impl ProofInput for PayoutProofInput {
    fn commit(&self, transcript: &mut Transcript) {
        transcript.append_byte(b"value", self.value);
        transcript.append_message(b"target", self.target.as_bytes());
        transcript.append_message(b"recipient", self.recipient.as_bytes());
        for tx in &self.transactions {
            transcript.append_byte(b"value", tx.value);
            transcript.append_point(b"pk", &tx.pk);
        }

        for tag in &self.nullifier {
            transcript.append_point(b"tag", tag);
        }
    }
}

impl PayoutProofInput {
    pub fn new(credential: &Credential, value: u8, target: &str, recipient: &str, spend: Vec<Transaction>, transactions: Vec<Transaction>) -> (PayoutProofInput, PayoutProofSecrets) {
        let mut sample = match transactions.len() > ANONYMITY_SET + spend.len() {
            false => transactions,
            true => {
                let mut rng = rand::thread_rng();
                let mut sample: Vec<Transaction> = spend.iter()
                    .chain(
                        transactions
                            .choose_multiple(&mut rng, ANONYMITY_SET + spend.len())
                            .filter(|k| !spend.contains(k))
                    )
                    .take(ANONYMITY_SET)
                    .cloned()
                    .collect();
                sample.shuffle(&mut rng);
                sample
            }
        };

        let mut padding: Vec<Transaction> = (0..MAX_INPUTS).map(|_| {
            let id = ResourceIdentifier::new();
            let pk = (G1Affine::generator() * derive_reward_key(&credential, &id)).to_affine();

            Transaction {
                id: id,
                pk: pk,
                value: 0,
            }
        }).collect();

        let mut i = 0;
        let mut spend: Vec<Transaction> = spend;
        while spend.len() < MAX_INPUTS {
            spend.push(padding[i].clone());
            i += 1;
        }

        sample.append(&mut padding);

        (
            PayoutProofInput {
                value: value,
                target: target.to_string(),
                recipient: recipient.to_string(),
                transactions: sample,
                nullifier: spend.iter().map(|input| Tag::from(PAYOUT_A() * derive_reward_key(&credential, &input.id).invert().unwrap())).collect()
            },
            PayoutProofSecrets {
                spend: spend,
                credential: credential.clone()
            }
        )
    }
}

impl Proof<PayoutProofInput, PayoutProofSecrets> for PayoutProof {
    fn get_variables(inputs: &PayoutProofInput, secrets: &PayoutProofSecrets, u: &Scalar) -> Variables {
        let mut vars = Variables::new();
        let mut sep = u.clone();

        for (i, input) in secrets.spend.iter().enumerate() {
            for (j, tx) in inputs.transactions.iter().enumerate() {
                vars.add(Variable::Inner {
                    id: format!("tx_{}_{}", i, j),
                    G: ((tx.pk + (PAYOUT_H() * Scalar::from(tx.value as u64))) * &sep).to_affine(),
                    cl: match input == tx {
                        true => Scalar::one(),
                        false => Scalar::zero()
                    },
                    cr: match input == tx {
                        true => Scalar::zero(),
                        false => -Scalar::one()
                    }
                });
            }

            let sk = derive_reward_key(&secrets.credential, &input.id);
            let sk_inv = sk.invert().unwrap();

            vars.add(Variable::Inner {
                id: format!("g_{}", i),
                G: (G1Affine::generator() * &sep).to_affine(),
                cl: -sk,
                cr: -sk_inv
            });
            vars.add(Variable::Inner {
                id: format!("h_{}", i),
                G: (PAYOUT_H() * &sep).to_affine(),
                cl: -Scalar::from(input.value as u64),
                cr: Scalar::zero()
            });
            vars.add(Variable::Inner {
                id: format!("A_{}", i),
                G: (PAYOUT_A() * &sep).to_affine(),
                cl: -sk_inv,
                cr: Scalar::zero()
            });
            vars.add(Variable::Inner {
                id: format!("T_{}", i),
                G: (inputs.nullifier[i].tag * &sep).to_affine(),
                cl: Scalar::one(),
                cr: Scalar::zero()
            });

            sep = &sep * u;
        }

        let sum = secrets.spend.iter().fold(0, |acc, tx| acc + tx.value);
        let remainder = match sum > 0 {
            true => sum - inputs.value,
            _ => 0
        };

        for i in 0..8 {
            let bit = Scalar::from(((remainder & u8::pow(2, i)) >> i) as u64);
            vars.add(Variable::Scratch {
                id: format!("bit_{}", i),
                cl: bit,
                cr: bit - Scalar::one()
            });
        }

        vars
    }

    fn get_constraints(inputs: &PayoutProofInput, y: &Scalar) -> Vec<Constraint> {
        // sum of all tx and remainder has to equal payout value
        let mut a = Constraint::new(ConstraintType::Dir, -Scalar::from(inputs.value as u64));

        // show that remainder bits are binary
        let mut b = Constraint::new(ConstraintType::Mul, Scalar::zero());
        let mut c = Constraint::new(ConstraintType::One, Scalar::zero());

        // cl / cr of tx selection has to be binary
        let mut d = Constraint::new(ConstraintType::Mul, Scalar::zero());
        let mut e = Constraint::new(ConstraintType::One, Scalar::zero());

        // show that only one tx per input is selected
        let mut f = Constraint::new(ConstraintType::Dir, sum_of_powers(y, MAX_INPUTS));

        // show that there is a one in cl of double spend tag
        let mut h =  Constraint::new(ConstraintType::Dir, sum_of_powers(y, MAX_INPUTS));

        // show that same sk is used to open tx and to generate double spend tag
        let mut u =  Constraint::new(ConstraintType::Mul, sum_of_powers(y, MAX_INPUTS));
        let mut v =  Constraint::new(ConstraintType::Sum, Scalar::zero());

        let mut sep_binary = Scalar::one();
        let mut sep = Scalar::one();

        for i in 0..MAX_INPUTS {
            for (j, _) in inputs.transactions.iter().enumerate() {
                d.right_set(&format!("tx_{}_{}", i, j), sep_binary.clone());
                e.left_set(&format!("tx_{}_{}", i, j), sep_binary.clone());
                f.left_set(&format!("tx_{}_{}", i, j), sep.clone());
                sep_binary = &sep_binary * y;
            }

            a.left_set(&format!("h_{}", i), Scalar::one());

            h.left_set(&format!("T_{}", i), sep.clone());

            v.left_set(&format!("A_{}", i), -sep.clone());
            v.right_set(&format!("g_{}", i), sep.clone());

            u.right_set(&format!("g_{}", i), sep.clone());

            sep = &sep * y;
        }

        let mut sep_binary = Scalar::one();

        for i in 0..8 {
            a.left_set(&format!("bit_{}", i), Scalar::from(u64::pow(2, i)));
            b.right_set(&format!("bit_{}", i), sep_binary.clone());
            c.left_set(&format!("bit_{}", i), sep_binary.clone());
            sep_binary = &sep_binary * y;
        }

        vec![a, b, c, d, e, f, h, u, v]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zksnark::GenericProof;

    use ff::Field;
    use rand::Rng;
    use rand::thread_rng;
    use merlin::Transcript;
    use bls12_381::hash_to_curve::{HashToCurve, ExpandMsgXmd};

    #[test]
    fn payout() {
        let mut csrng = thread_rng();

        // generate test data
        let credential = Credential::default();
        let resources: Vec<(ResourceIdentifier, u8)> = vec![
            (ResourceIdentifier::new(), 5),
            (ResourceIdentifier::new(), 25),
            (ResourceIdentifier::new(), 2),
            (ResourceIdentifier::new(), 10),
        ];

        // let transactions: &[usize] = &[0,1,1,0,1,2,1,0,3,2,2,1,3,0];
        let distribution = rand::distributions::Uniform::new_inclusive(0, resources.len() - 1);
        let transactions: Vec<usize> = (&mut csrng).sample_iter(distribution).take(10000).collect();
        let mut transactions: Vec<Transaction> = transactions.iter().map(|i| Transaction {
            pk: (G1Affine::generator() * &Scalar::random(&mut csrng)).to_affine(),
            value: resources[*i].1,
            id:  resources[*i].0
        }).collect();

        let spend: &[usize] = &[0, 1, 3];
        let spend: Vec<Transaction> = spend.iter().map(|i| Transaction {
            pk: (G1Affine::generator() * derive_reward_key(&credential, &resources[*i].0)).to_affine(),
            value: resources[*i].1,
            id:  resources[*i].0
        }).collect();

        transactions.insert(3, spend[0].clone());
        transactions.insert(5, spend[1].clone());
        transactions.insert(8, spend[2].clone());

        let value = 30;

        let (inputs, secrets) = PayoutProofInput::new(
            &credential,
            value,
            "test",
            "test@example.com",
            spend,
            transactions
        );

        // create proof
        let mut prover_transcript = Transcript::new(b"payout test");
        let proof = GenericProof::<PayoutProofInput>::proove::<PayoutProofSecrets, PayoutProof>(&mut prover_transcript, inputs, secrets).unwrap();

        // verify proof
        let mut verifier_transcript = Transcript::new(b"payout test");
        let s = proof.verify::<PayoutProofSecrets, PayoutProof>(&mut verifier_transcript);

        assert!(s.is_ok());
    }

    #[test]
    fn check_H() {
        let H = <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(b"payout_H", b"payout_H").to_affine();
        assert_eq!(H, PAYOUT_H());
    }

    #[test]
    fn check_A() {
        let A = <bls12_381::G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(b"payout_A", b"payout_A").to_affine();
        assert_eq!(A, PAYOUT_A());
    }
}
