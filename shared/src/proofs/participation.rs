#![allow(non_snake_case)]
use ff::Field;
use group::Curve;

use serde::{Serialize, Deserialize};
use bls12_381::{G1Projective, G1Affine, Scalar};
use serde_with::serde_as;

use crate::credential;
use crate::credential::BINDING_G;
use crate::credential::CREDENTIAL_I;
use crate::external::util::as_scalar;
use crate::external::util::as_u32;
use crate::pbss::BlindedSignRequest;
use crate::types::AttributeConstraint;
use crate::types::credential::*;
use crate::types::prerequisites::*;
use crate::external::transcript::TranscriptProtocol;
use crate::proofs::generic::{Proof, Variables, Variable, Constraint, ConstraintType, ProofInput, Transcript};
use crate::external::util::{exp_iter, sum_of_powers};
use crate::types::Resource;
use crate::proofs::payout::{PAYOUT_G1, PAYOUT_V0, PAYOUT_V1};
use crate::pbss;

pub struct ParticipationProof {}

#[derive(Debug)]
pub struct ParticipationProofSecrets {
    credential: Credential,
    disqualifier_random: Scalar,
    randomness: Scalar,
    reward_s: Scalar,
    reward_d: Scalar
}

impl Default for ParticipationProofSecrets {
    fn default() -> Self {
        Self {
            credential: Credential::default(),
            disqualifier_random: Scalar::zero(),
            randomness: Scalar::zero(),
            reward_s: Scalar::zero(),
            reward_d: Scalar::zero()
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParticipationProofInput {
    #[serde(with = "crate::serialization::G1Affine")]
    pub tag: G1Affine,
    #[serde(with = "crate::serialization::Scalar")]
    pub study: Scalar,
    #[serde_as(as = "Vec<crate::serialization::SerializableG1Affine>")]
    pub attributes: Vec<G1Affine>,

    #[serde(with = "crate::serialization::Scalar")]
    pub reward: Scalar,
    pub reward_request: BlindedSignRequest,

    pub ipk: IssuerPublicKey,
    pub auth_request: AuthenticationRequest,

    pub qualifiers: Vec<Qualifier>,
    pub disqualifiers: Vec<Qualifier>,
    pub randomized_disqualifiers: Vec<RandomizedDisqualifier>,
    pub constraints: Vec<AttributeConstraint>,

    #[serde(with = "crate::serialization::G1Affine")]
    pub commitment: G1Affine,
}

impl ProofInput for ParticipationProofInput {
    fn commit(&self, transcript: &mut Transcript) {
        transcript.append_scalar(b"study", &self.study);
        transcript.append_g1(b"tag", &self.tag);

        for attribute in &self.attributes {
            transcript.append_g1(b"attribute", &attribute);
        }

        transcript.append_scalar(b"reward", &self.reward);
        transcript.append_g1(b"reward_alpha", &self.reward_request.alpha);

        for qualifier in &self.qualifiers {
            transcript.append_scalar(b"qid", &qualifier.id);
            for tag in &qualifier.tags {
                transcript.append_g1(b"qtag", &tag);
            }
        }

        for disqualifier in &self.disqualifiers {
            transcript.append_scalar(b"did", &disqualifier.id);
            for tag in &disqualifier.tags {
                transcript.append_g1(b"dtag", &tag);
            }
        }

        for disqualifier in &self.randomized_disqualifiers {
            transcript.append_scalar(b"rid", &disqualifier.id);
            transcript.append_g1(b"rtag", &disqualifier.randomized_tag);
            for tag in &disqualifier.tags {
                transcript.append_g1(b"rtag", &tag);
            }
        }
    }
}

impl ParticipationProofInput {
    pub fn new(ipk: &IssuerPublicKey, cvk: &pbss::PublicKey, credential: &Credential, resource: &Resource) -> (ParticipationProofInput, ParticipationProofSecrets) {
        let (auth_request, (randomness, commitment)) = credential::authenticate(&credential, &resource.id);
        let tag = credential.derive_tag(&resource.id).unwrap();
        let rD = Scalar::random(&mut rand::thread_rng());

        let ipk = ipk.clone();
        let study = resource.id.clone();
        let attributes = credential.attributes.clone();
        let credential = credential.clone();
        let qualifiers = resource.qualifier.to_vec();
        let disqualifiers = resource.disqualifier.to_vec();
        let constraints = resource.constraints.to_vec();

        let randomized_disqualifiers: Vec<RandomizedDisqualifier> = disqualifiers
            .iter()
            .map(|d| RandomizedDisqualifier {
                id: d.id,
                randomized_tag: (&credential.derive_tag(&d.id).unwrap() * &rD).to_affine(),
                tags: d.tags.iter().map(|t| (t * rD).to_affine()).collect()
            })
            .collect();

        let mut prng = credential.derive_reward_rng(&study);
        let s = <bls12_381::Scalar as ff::Field>::random(&mut prng);
        let d = <bls12_381::Scalar as ff::Field>::random(&mut prng);
        let reward = as_scalar(resource.reward.into());
        let reward_request = pbss::Blind(cvk, &vec![reward], &vec![s, credential.identity], &d, &mut prng);

        (
            ParticipationProofInput {
                tag,
                study,
                attributes,
                reward,
                reward_request,
                ipk,
                auth_request,
                qualifiers,
                disqualifiers,
                randomized_disqualifiers,
                commitment,
                constraints
            },
            ParticipationProofSecrets {
                credential,
                randomness,
                disqualifier_random: rD,
                reward_s: s.clone(),
                reward_d: d.clone()
            }
        )
    }
}

impl Proof<ParticipationProofInput, ParticipationProofSecrets, ()> for ParticipationProof {
    fn get_variables(inputs: &ParticipationProofInput, secrets: &ParticipationProofSecrets, u: &Scalar) -> Variables {
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
            G: (inputs.tag * &sep).to_affine(),
            cl: secrets.credential.sk + inputs.study,
            cr: Scalar::zero()
        });
        vars.add(Variable::Scratch {
            id: "sk".to_string(),
            cl: secrets.credential.sk.clone(),
            cr: Scalar::zero()
        });

        sep = &sep * u;

        vars.add(Variable::Inner {
            id: format!("vc"),
            G: (inputs.commitment * &sep).to_affine(),
            cl: -Scalar::one(),
            cr: Scalar::zero()
        });
        vars.add(Variable::Inner {
            id: "vr".to_string(),
            G: (BINDING_G() * &sep).to_affine(),
            cl: secrets.randomness.clone(),
            cr: Scalar::zero()
        });

        vars.add(Variable::Inner {
            id: "auth_identity".to_string(),
            G: (CREDENTIAL_I() * &sep).to_affine(),
            cl: secrets.credential.identity,
            cr: Scalar::zero()
        });

        for (i, attr_u) in inputs.attributes.iter().enumerate() {
            vars.add(Variable::Inner {
                id: format!("attr_{}", i),
                G: (attr_u * &sep).to_affine(),
                cl: secrets.credential.values.get(i).unwrap_or(&Scalar::default()).clone(),
                cr: Scalar::zero()
            });
        }

        sep = &sep * u;

        vars.add(Variable::Inner {
            id: "reward_s".to_string(),
            G: (PAYOUT_V0() * &sep).to_affine(),
            cl: secrets.reward_s,
            cr: Scalar::zero()
        });

        vars.add(Variable::Inner {
            id: "reward_identity".to_string(),
            G: (PAYOUT_V1() * &sep).to_affine(),
            cl: secrets.credential.identity.clone(),
            cr: Scalar::zero()
        });

        vars.add(Variable::Inner {
            id: "reward_d".to_string(),
            G: (PAYOUT_G1() * &sep).to_affine(),
            cl: secrets.reward_d,
            cr: Scalar::zero()
        });

        vars.add(Variable::Inner {
            id: "reward_alpha".to_string(),
            G: (inputs.reward_request.alpha.clone() * &sep).to_affine(),
            cl: -Scalar::one(),
            cr: Scalar::zero()
        });

        sep = &sep * u;

        for (i, qualifier) in inputs.qualifiers.iter().enumerate() {
            let q = secrets.credential.derive_tag(&qualifier.id).unwrap();

            vars.add(Variable::Inner {
                id: format!("qg_{}", i),
                G: (G1Affine::generator() * &sep).to_affine(),
                cl: -(secrets.credential.sk + qualifier.id).invert().unwrap(),
                cr: Scalar::zero()
            });
            vars.add(Variable::Scratch {
                id: format!("qsk_{}", i),
                cl: secrets.credential.sk + qualifier.id,
                cr: (secrets.credential.sk + qualifier.id).invert().unwrap()
            });

            for (j, tag) in qualifier.tags.iter().enumerate() {
                vars.add(Variable::Inner {
                    id: format!("qtag_{}_{}", i, j),
                    G: (tag * &sep).to_affine(),
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
                G: (randomized.randomized_tag * &sep).to_affine(),
                cl: secrets.credential.sk + disqualifier.id,
                cr: Scalar::zero()
            });
            vars.add(Variable::Inner {
                id: format!("dtags_{}", i),
                G: disqualifier.tags.iter().map(|tag| {
                    sep_d = &sep_d * u;
                    tag * &sep_d
                }).sum::<G1Projective>().to_affine(),
                cl: secrets.disqualifier_random.clone(),
                cr: Scalar::zero()
            });
            vars.add(Variable::Inner {
                id: format!("drand_{}", i),
                G: randomized.tags.iter().map(|rtag| {
                    sep_r = &sep_r * u;
                    rtag * &sep_r
                }).sum::<G1Projective>().to_affine(),
                cl: -Scalar::one(),
                cr: Scalar::zero()
            });
            sep = &sep_d * u;
        }

        for (cid, constraint) in inputs.constraints.iter().enumerate() {
            match constraint {
                AttributeConstraint::Element(i, options) => {
                    vars.add(Variable::Inner {
                        id: format!("cstr_{}", cid),
                        G: (G1Affine::generator() * &sep).to_affine(),
                        cl: -secrets.credential.values.get(*i as usize).unwrap_or(&Scalar::zero()),
                        cr: Scalar::zero()
                    });
                    for (j, option) in options.iter().enumerate() {
                        let o = as_scalar(*option);
                        vars.add(Variable::Inner {
                            id: format!("cstr_{}_{}", cid, j),
                            G: (G1Affine::generator() * &o * &sep).to_affine(),
                            cl: match &o == secrets.credential.values.get(*i as usize).unwrap_or(&Scalar::zero()) {
                                true => Scalar::one(),
                                false => Scalar::zero()
                            },
                            cr: match &o == secrets.credential.values.get(*i as usize).unwrap_or(&Scalar::zero()) {
                                true => Scalar::zero(),
                                false => -Scalar::one()
                            }
                        });
                    }
                },
                AttributeConstraint::Range(i, from, to) => {
                    let value = as_u32(&secrets.credential.values.get(*i as usize).unwrap_or(&as_scalar(*from)));

                    assert!(value >= *from && value <= *to);

                    let diff = value - from;
                    let diff2 = to - value;

                    let bits = (to - from).next_power_of_two().trailing_zeros() + 1;

                    for j in 0..bits {
                        let bit = Scalar::from(((diff & u32::pow(2, j)) >> j) as u64);
                        vars.add(Variable::Scratch {
                            id: format!("cstr_{}_{}_1", cid, j),
                            cl: bit,
                            cr: bit - Scalar::one()
                        });
                        let bit2 = Scalar::from(((diff2 & u32::pow(2, j)) >> j) as u64);
                        vars.add(Variable::Scratch {
                            id: format!("cstr_{}_{}_2", cid, j),
                            cl: bit2,
                            cr: bit2 - Scalar::one()
                        });
                    }
                },
            }

            sep = &sep * u;
        }

        vars
    }

    fn get_constraints(inputs: &ParticipationProofInput, y: &Scalar) -> Vec<Constraint> {
        // show that tag - sk = study
        let mut v1 = Constraint::new(ConstraintType::Dir, inputs.study.clone());
        v1.left_set("tag", Scalar::one());
        v1.left_set("sk", -Scalar::one());

        let mut vr = Constraint::new(ConstraintType::Dir, Scalar::zero());
        vr.left_set("auth_identity", -Scalar::one());
        vr.left_set("reward_identity", Scalar::one());

        // show that there is a -1 below g
        let mut v2 = Constraint::new(ConstraintType::Dir, -sum_of_powers(y, 3));
        v2.left_set("g", Scalar::one());
        v2.left_set("vc", y.clone());
        v2.left_set("reward_alpha", y * y);

        // show that one tag is selected per qualifier separated by y
        let mut v3 = Constraint::new(ConstraintType::Dir, sum_of_powers(y, inputs.qualifiers.len()));

        // show that all cl/cr values for qualifier/disqualifiers are binary
        let mut v_one = Constraint::new(ConstraintType::One, Scalar::zero());
        let mut v_mul = Constraint::new(ConstraintType::Mul, Scalar::zero());

        // show that qsk matches sk for all qualifiers
        let mut vq = Constraint::new(ConstraintType::Dir, inputs.qualifiers.iter().zip(exp_iter(y.clone())).map(|(a, b)| a.id * b).sum());
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
                v_one.left_set(&format!("qtag_{}_{}", i, j), sep_binary.clone());
                v_mul.right_set(&format!("qtag_{}_{}", i, j), sep_binary.clone());
                sep_binary = &sep_binary * y;
            }
            sep = &sep * y;
        }

        // ensure that the study id of randomized disqualifier is correct
        let mut v5 = Constraint::new(ConstraintType::Dir, inputs.disqualifiers.iter().zip(exp_iter(y.clone())).map(|(a, b)| a.id * b).sum());
        v5.left_set("sk", -sum_of_powers(y, inputs.disqualifiers.len()));

        // show that the randomness in dg and dtags is the same
        let mut v5a = Constraint::new(ConstraintType::Dir, Scalar::zero());

        // show that there is a -1 in drandomized
        let mut v5b = Constraint::new(ConstraintType::Dir, -sum_of_powers(y, inputs.disqualifiers.len()));

        let mut sep = Scalar::one();
        for (i, _) in inputs.disqualifiers.iter().enumerate() {
            v5.left_set(&format!("dtag_{}", i), sep.clone());

            v5a.left_set(&format!("dg_{}", i), sep.clone());
            v5a.left_set(&format!("dtags_{}", i), sep.clone());

            v5b.left_set(&format!("drand_{}", i), sep.clone());
            sep = &sep * y;
        }

        let mut constraints = vec![v1, vr, v2, v3, vq, vqa, vqb, v5, v5a, v5b];
        
        // proove attribute constraints
        for (cid, constraint) in inputs.constraints.iter().enumerate() {
            match constraint {
                AttributeConstraint::Element(i, options) => {
                    // show that one tag is selected per attribute separated by y
                    let mut v3 = Constraint::new(ConstraintType::Dir, sep.clone());

                    // mul/one = 0

                    for (j, _) in options.iter().enumerate() {
                        v3.left_set(&format!("cstr_{}_{}", cid, j), sep.clone());

                        v_mul.right_set(&format!("cstr_{}_{}", cid, j), sep_binary.clone());
                        v_one.left_set(&format!("cstr_{}_{}", cid, j), sep_binary.clone());

                        sep_binary = &sep_binary * y;
                    }

                    // show that attribute is the same
                    let mut va = Constraint::new(ConstraintType::Dir, Scalar::zero());
                    va.left_set(&format!("cstr_{}", cid), Scalar::one());
                    va.left_set(&format!("attr_{}", i), Scalar::one());

                    constraints.push(v3);
                    constraints.push(va);
                },
                AttributeConstraint::Range(i, from, to) => {
                    let mut vd = Constraint::new(ConstraintType::Dir, as_scalar(*from));
                    vd.left_set(&format!("attr_{}", i), Scalar::one());

                    let mut vk = Constraint::new(ConstraintType::Dir, as_scalar(*to));
                    vk.left_set(&format!("attr_{}", i), Scalar::one());

                    let bits = 32 - (to - from).leading_zeros() + 1;
                    for j in 0..bits {
                        v_mul.right_set(&format!("cstr_{}_{}_1", cid, j), sep_binary.clone());
                        v_one.left_set(&format!("cstr_{}_{}_1", cid, j), sep_binary.clone());

                        vd.left_set(&format!("cstr_{}_{}_1", cid, j), -Scalar::from(u64::pow(2, j)));

                        v_mul.right_set(&format!("cstr_{}_{}_2", cid, j), sep_binary.clone());
                        v_one.left_set(&format!("cstr_{}_{}_2", cid, j), sep_binary.clone());

                        vk.left_set(&format!("cstr_{}_{}_2", cid, j), Scalar::from(u64::pow(2, j)));

                        sep_binary = &sep_binary * y;
                    }

                    constraints.push(vd);
                    constraints.push(vk);
                },
            }

            sep = &sep * y;
        }

        constraints.push(v_one);
        constraints.push(v_mul);
        return constraints;
    }

    fn additional_checks(inputs: &ParticipationProofInput, _: &(), _: &mut Transcript) -> bool {
        let b1 = inputs.randomized_disqualifiers.iter()
            .all(|disqualifier| !disqualifier.tags.contains(&disqualifier.randomized_tag));
        let b2 = crate::credential::verify(&inputs.ipk, &inputs.auth_request);
        let b3 = inputs.auth_request.token == inputs.tag;

        b1 && b2 && b3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential::{init, issue_request, issue, get_credential};
    use crate::types::{Resource, AttributeConstraint};
    use crate::proofs::generic::GenericProof;
    use crate::external::util::rand_scalar;

    use rand::Rng;
    use merlin::Transcript;

    #[test]
    fn participation() {
        let mut rng = rand::thread_rng();

        let identity = "user@example.com";
        let (ipk, isk) = init(&mut rng, 3);
        let (_csk, cvk) = pbss::Gen(&mut rng, 2, 1, "payment");

        let attrs = vec![Scalar::from(2 as u64), Scalar::from(1985 as u64), rand_scalar()];
        let (request, mut credential) = issue_request(&mut rng, &ipk, &identity, attrs);
        let response = issue(&ipk, &isk, &request).unwrap();
        get_credential(&ipk, &response, &mut credential).unwrap();

        let qid = rand_scalar();
        let qid2 = rand_scalar();
        let dqid = rand_scalar();
        let dqid2 = rand_scalar();

        let resource: Resource = Resource {
            id: rand_scalar(),
            name: "name".to_string(),
            summary: "summary".to_string(),
            description: "description".to_string(),
            duration: "duration".to_string(),
            reward: rng.gen(),
            webBased: false,
            studyUrl: None,
            qualifier: vec![
                Qualifier {
                    id: qid,
                    tags: vec![
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        credential.derive_tag(&qid).unwrap(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                    ]
                },
                Qualifier {
                    id: qid2,
                    tags: vec![
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        credential.derive_tag(&qid2).unwrap(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                    ]
                }
            ],
            disqualifier: vec![
                Qualifier {
                    id: dqid,
                    tags: vec![
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        //credential.derive_tag(&dqid).unwrap(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                    ]
                },
                Qualifier {
                    id: dqid2,
                    tags: vec![
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                        (G1Affine::generator() * &Scalar::random(&mut rng)).to_affine(),
                    ]
                }
            ],
            constraints: vec![
                AttributeConstraint::Range(1, 1980, 1990),
                AttributeConstraint::Element(0, vec![0, 2, 3])
            ]
        };

        // generate test data
        let (inputs, secrets) = ParticipationProofInput::new(&ipk, &cvk, &credential, &resource);

        // constraints
        let mut prover_transcript = Transcript::new(b"test example");
        let proof = GenericProof::<ParticipationProofInput, ()>::proove::<ParticipationProofSecrets, ParticipationProof>(&mut prover_transcript, inputs, secrets).unwrap();

        let mut verifier_transcript = Transcript::new(b"test example");
        let s = proof.verify::<ParticipationProofSecrets, ParticipationProof>(&mut verifier_transcript);

        assert!(s.is_ok());
    }
}
