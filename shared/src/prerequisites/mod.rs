#![allow(non_snake_case)]
use ff::Field;
use group::Curve;

use serde::{Serialize, Deserialize};
use bls12_381::{G1Projective, G1Affine, Scalar};

use crate::types::*;
use crate::external::transcript::TranscriptProtocol;
use crate::zksnark::{Proof, Variables, Variable, Constraint, ConstraintType, ProofInput, Transcript};
use crate::external::util::{exp_iter, sum_of_powers};
use crate::payout::derive_reward_key;

pub struct PrerequisitesProof {}

pub struct PrerequisitesProofSecrets {
    credential: Credential,
    disqualifier_random: Scalar,
    reward_sk: Scalar
}

impl Default for PrerequisitesProofSecrets {
    fn default() -> Self {
        Self {
            credential: Credential::default(),
            disqualifier_random: Scalar::zero(),
            reward_sk: Scalar::zero()
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PrerequisitesProofInput {
    pub tag: Tag,
    pub study: ResourceIdentifier,

    #[serde(with = "crate::serialization::G1Affine")]
    pub reward_pk: G1Affine,

    pub qualifiers: Vec<Qualifier>,
    pub disqualifiers: Vec<Qualifier>,
    pub randomized_disqualifiers: Vec<RandomizedDisqualifier>,
}

impl ProofInput for PrerequisitesProofInput {
    fn commit(&self, transcript: &mut Transcript) {
        transcript.append_scalar(b"study", &self.study);
        transcript.append_point(b"tag", &self.tag);
        transcript.append_point(b"reward_pk", &self.reward_pk);

        for qualifier in &self.qualifiers {
            transcript.append_scalar(b"qid", &qualifier.id.id);
            for tag in &qualifier.tags {
                transcript.append_point(b"qtag", &tag.tag);
            }
        }

        for disqualifier in &self.disqualifiers {
            transcript.append_scalar(b"did", &disqualifier.id.id);
            for tag in &disqualifier.tags {
                transcript.append_point(b"dtag", &tag.tag);
            }
        }

        for disqualifier in &self.randomized_disqualifiers {
            transcript.append_scalar(b"rid", &disqualifier.id);
            transcript.append_point(b"rtag", &disqualifier.randomized_tag.tag);
            for tag in &disqualifier.tags {
                transcript.append_point(b"rtag", &tag.tag);
            }
        }
    }
}

impl PrerequisitesProofInput {
    pub fn new(credential: &Credential, study: &ResourceIdentifier, qualifiers: &[Qualifier], disqualifiers: &[Qualifier]) -> (PrerequisitesProofInput, PrerequisitesProofSecrets) {
        let tag = Tag::new(&credential, &study).unwrap();
        let rD = Scalar::random(&mut rand::thread_rng());
        let randomized_disqualifiers: Vec<RandomizedDisqualifier> = disqualifiers
            .iter()
            .map(|d| RandomizedDisqualifier {
                id: d.id,
                randomized_tag: Tag::new(&credential, &d.id).unwrap().randomize(&rD),
                tags: d.tags.iter().map(|t| Tag::from(t.tag * rD)).collect()
            })
            .collect();

        let study = study.clone();
        let credential = credential.clone();
        let qualifiers = qualifiers.to_vec();
        let disqualifiers = disqualifiers.to_vec();

        let reward_sk = derive_reward_key(&credential, &study);
        let reward_pk = (G1Affine::generator() * reward_sk).to_affine();

        (
            PrerequisitesProofInput {
                tag,
                study,
                reward_pk,
                qualifiers,
                disqualifiers,
                randomized_disqualifiers
            },
            PrerequisitesProofSecrets {
                credential,
                reward_sk,
                disqualifier_random: rD,
            }
        )
    }
}

impl Proof<PrerequisitesProofInput, PrerequisitesProofSecrets> for PrerequisitesProof {
    fn get_variables(inputs: &PrerequisitesProofInput, secrets: &PrerequisitesProofSecrets, u: &Scalar) -> Variables {
        let mut vars = Variables::new();

        let mut sep = Scalar::one();

        vars.add(Variable::Inner {
            id: "g".to_string(),
            G: (G1Affine::generator() * &sep).to_affine(),
            cl: -Scalar::one(),
            cr: Scalar::zero()
        });
        vars.add(Variable::Inner {
            id: "tag".to_string(),
            G: (inputs.tag.tag * &sep).to_affine(),
            cl: secrets.credential.sk + inputs.study.id,
            cr: Scalar::zero()
        });
        vars.add(Variable::Scratch {
            id: "sk".to_string(),
            cl: secrets.credential.sk.clone(),
            cr: Scalar::zero()
        });

        sep = &sep * u;

        vars.add(Variable::Inner {
            id: "reward_pk".to_string(),
            G: (inputs.reward_pk * &sep).to_affine(),
            cl: Scalar::one(),
            cr: Scalar::zero()
        });

        vars.add(Variable::Inner {
            id: "reward_sk".to_string(),
            G: (G1Affine::generator() * &sep).to_affine(),
            cl: -secrets.reward_sk,
            cr: Scalar::zero()
        });

        sep = &sep * u;

        for (i, qualifier) in inputs.qualifiers.iter().enumerate() {
            let q = Tag::new(&secrets.credential, &qualifier.id).unwrap();

            vars.add(Variable::Inner {
                id: format!("qg_{}", i),
                G: (G1Affine::generator() * &sep).to_affine(),
                cl: -(secrets.credential.sk + qualifier.id.id).invert().unwrap(),
                cr: Scalar::zero()
            });
            vars.add(Variable::Scratch {
                id: format!("qsk_{}", i),
                cl: secrets.credential.sk + qualifier.id.id,
                cr: (secrets.credential.sk + qualifier.id.id).invert().unwrap()
            });

            for (j, tag) in qualifier.tags.iter().enumerate() {
                vars.add(Variable::Inner {
                    id: format!("qtag_{}_{}", i, j),
                    G: (tag.tag * &sep).to_affine(),
                    cl: match tag == &q {
                        true => Scalar::one(),
                        false => Scalar::zero()
                    },
                    cr: match tag == &q {
                        true => Scalar::zero(),
                        false => -Scalar::one()
                    }
                });
            }

            sep = &sep * u;
        }

        for (i, (disqualifier, randomized)) in inputs.disqualifiers.iter().zip(&inputs.randomized_disqualifiers).enumerate() {
            let mut sep_d = sep.clone();
            let mut sep_r = sep.clone();
            vars.add(Variable::Inner {
                id: format!("dg_{}", i),
                G: (G1Affine::generator() * &sep).to_affine(),
                cl: -secrets.disqualifier_random,
                cr: Scalar::zero()
            });
            vars.add(Variable::Inner {
                id: format!("dtag_{}", i),
                G: (randomized.randomized_tag.tag * &sep).to_affine(),
                cl: secrets.credential.sk + disqualifier.id.id,
                cr: Scalar::zero()
            });
            vars.add(Variable::Inner {
                id: format!("dtags_{}", i),
                G: disqualifier.tags.iter().map(|tag| {
                    sep_d = &sep_d * u;
                    tag.tag * &sep_d
                }).sum::<G1Projective>().to_affine(),
                cl: secrets.disqualifier_random.clone(),
                cr: Scalar::zero()
            });
            vars.add(Variable::Inner {
                id: format!("drandomized_{}", i),
                G: randomized.tags.iter().map(|rtag| {
                    sep_r = &sep_r * u;
                    rtag.tag * &sep_r
                }).sum::<G1Projective>().to_affine(),
                cl: -Scalar::one(),
                cr: Scalar::zero()
            });
            sep = &sep_d * u;
        }

        vars
    }

    fn get_constraints(inputs: &PrerequisitesProofInput, y: &Scalar) -> Vec<Constraint> {
        // show that tag - sk = study
        let mut v1 = Constraint::new(ConstraintType::Dir, inputs.study.id.clone());
        v1.left_set("tag", Scalar::one());
        v1.left_set("sk", -Scalar::one());

        // show that there is a -1 below g
        let mut v2 = Constraint::new(ConstraintType::Dir, -Scalar::one());
        v2.left_set("g", Scalar::one());

        // show that one tag is selected per qualifier separated by y
        let mut v3 = Constraint::new(ConstraintType::Dir, sum_of_powers(y, inputs.qualifiers.len()));

        // show that all cl/cr values for qualifier/disqualifiers are binary
        let mut v4a = Constraint::new(ConstraintType::One, Scalar::zero());
        let mut v4b = Constraint::new(ConstraintType::Mul, Scalar::zero());

        // show that qsk matches sk for all qualifiers
        let mut vq = Constraint::new(ConstraintType::Dir, inputs.qualifiers.iter().zip(exp_iter(y.clone())).map(|(a, b)| a.id.id * b).sum());
        vq.left_set("sk", -sum_of_powers(y, inputs.qualifiers.len()));

        // show that qsk's cr is inverse of qsk's cl
        let mut vqa = Constraint::new(ConstraintType::Mul, sum_of_powers(y, inputs.qualifiers.len()));
        let mut vqb = Constraint::new(ConstraintType::Sum, Scalar::zero());

        let mut sep = Scalar::one();
        let mut sep_binary = Scalar::one();

        for (i, qualifier) in inputs.qualifiers.iter().enumerate() {
            vq.left_set(&format!("qsk_{}", i), sep.clone());
            vqa.right_set(&format!("qsk_{}", i), sep.clone());
            vqb.left_set(&format!("qg_{}", i), sep.clone());
            vqb.right_set(&format!("qsk_{}", i), sep.clone());

            for (j, _) in qualifier.tags.iter().enumerate() {
                v3.left_set(&format!("qtag_{}_{}", i, j), sep.clone());
                v4a.left_set(&format!("qtag_{}_{}", i, j), sep_binary.clone());
                v4b.right_set(&format!("qtag_{}_{}", i, j), sep_binary.clone());
                sep_binary = &sep_binary * y;
            }
            sep = &sep * y;
        }

        // schauen das study id von randomized disqualifier stimmt
        let mut v5 = Constraint::new(ConstraintType::Dir, inputs.disqualifiers.iter().zip(exp_iter(y.clone())).map(|(a, b)| a.id.id * b).sum());
        v5.left_set("sk", -sum_of_powers(y, inputs.disqualifiers.len()));

        // zeigen das die randomness in dg und dtags gleich ist
        let mut v5a = Constraint::new(ConstraintType::Dir, Scalar::zero());

        // zeigen das die in drandomized eine -1 steht
        let mut v5b = Constraint::new(ConstraintType::Dir, -sum_of_powers(y, inputs.disqualifiers.len()));

        let mut sep = Scalar::one();
        for (i, _) in inputs.disqualifiers.iter().enumerate() {
            v5.left_set(&format!("dtag_{}", i), sep.clone());

            v5a.left_set(&format!("dg_{}", i), sep.clone());
            v5a.left_set(&format!("dtags_{}", i), sep.clone());

            v5b.left_set(&format!("drandomized_{}", i), sep.clone());
            sep = &sep * y;
        }

        // show that there is a one beneath the reward pk
        let mut v6 = Constraint::new(ConstraintType::Dir, Scalar::one());
        v6.left_set("reward_pk", Scalar::one());

        vec![v1, v2, v3, v4a, v4b, vq, vqa, vqb, v5, v5a, v5b, v6]
    }

    fn additional_checks(inputs: &PrerequisitesProofInput) -> bool {
        inputs.randomized_disqualifiers
            .iter()
            .all(|disqualifier| !disqualifier.tags.contains(&disqualifier.randomized_tag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zksnark::GenericProof;

    use rand::thread_rng;
    use merlin::Transcript;

    #[test]
    fn prerequisites() {
        let mut csrng = thread_rng();

        // generate test data
        let credential = Credential::default();
        let study = ResourceIdentifier::new();

        let qid = ResourceIdentifier::new();
        let qualifier = Qualifier {
            id: ResourceIdentifier::from(qid),
            tags: vec![
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                Tag::new(&credential, &qid).unwrap(),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
            ]
        };

        let qid2 = ResourceIdentifier::new();
        let qualifier2 = Qualifier {
            id: ResourceIdentifier::from(qid2),
            tags: vec![
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                Tag::new(&credential, &qid2).unwrap(),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
            ]
        };

        let dqid = ResourceIdentifier::new();
        let dqualifier = Qualifier {
            id: ResourceIdentifier::from(dqid),
            tags: vec![
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                // Tag::new(&credential, &dqid).unwrap(),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
            ]
        };

        let dqid2 = ResourceIdentifier::new();
        let dqualifier2 = Qualifier {
            id: ResourceIdentifier::from(dqid2),
            tags: vec![
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
                Tag::from(G1Affine::generator() * &Scalar::random(&mut csrng)),
            ]
        };

        let (inputs, secrets) = PrerequisitesProofInput::new(
            &credential,
            &study,
            &vec![qualifier, qualifier2],
            &vec![dqualifier, dqualifier2]
        );

        // constraints
        let mut prover_transcript = Transcript::new(b"test example");
        let proof = GenericProof::<PrerequisitesProofInput>::proove::<PrerequisitesProofSecrets, PrerequisitesProof>(&mut prover_transcript, inputs, secrets).unwrap();

        let mut verifier_transcript = Transcript::new(b"test example");
        let s = proof.verify::<PrerequisitesProofSecrets, PrerequisitesProof>(&mut verifier_transcript);

        assert!(s.is_ok());
    }
}
