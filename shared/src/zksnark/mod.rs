#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use merlin::Transcript;
use core::iter;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};

use ff::Field;
use group::Curve;

use bls12_381::{G1Affine, Scalar};
use crate::external::error::ProofError;
use crate::external::inner_product_proof::{InnerProductProof, inner_product, vartime_multiscalar_mul};
use crate::external::util::{add_vec, smul_vec, mul_vec, sub_vec, VecPoly1};
use crate::external::transcript::TranscriptProtocol;
use crate::external::inner_product_proof;

#[derive(Debug, Clone)]
pub enum Constraint {
    Sum {
        left: HashMap<String, Scalar>,
        right: HashMap<String, Scalar>,
        result: Scalar
    },

    Mul {
        right: HashMap<String, Scalar>,
        result: Scalar
    },

    One {
        right: HashMap<String, Scalar>,
        result: Scalar
    },

    Dir {
        left: HashMap<String, Scalar>,
        result: Scalar
    },
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Variables {
    inner: BTreeMap<String, Variable>,
    scratch: BTreeMap<String, Variable>
}

impl std::fmt::Debug for Variables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\n{:^8}:  {:^64}   {:^64}   {:^64}\n", "id", "point (base64)", "cl (hex)", "cr (hex)"))?;
        f.write_fmt(format_args!("{:-<8}---{:-<64}---{:-<64}---{:-<64}\n", "", "", "", ""))?;
        for (_, v) in &self.inner {
            if let Variable::Inner { id, G, cl, cr } = v {
                f.write_fmt(format_args!("{:<8}:  {:<64}   {:<64}   {:<64}\n", &id, &base64::encode_config(G.to_compressed(), base64::URL_SAFE_NO_PAD), &cl.to_string()[2..], &cr.to_string()[2..]))?;
            }
        }

        f.write_fmt(format_args!("{:-<8}---{:-<64}---{:-<64}---{:-<64}\n", "", "", "", ""))?;
        
        for (_, v) in &self.scratch {
            if let Variable::Scratch { id, cl, cr } = v {
                f.write_fmt(format_args!("{:<8}:  {:<64}   {:<64}   {:<64}\n", &id, "", &cl.to_string()[2..], &cr.to_string()[2..]))?;
            }
        }

        Ok(())
    }
}

impl Variables {
    pub fn new() -> Variables {
        Variables {
            inner: BTreeMap::new(),
            scratch: BTreeMap::new()
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&Variable> {
        match self.inner.get(key) {
            Some(var) => Some(var),
            _ => self.scratch.get(key)
        }
    }

    pub fn add(&mut self, var: Variable) {
        let id = var.get_id().to_string();
        if var.is_scratch() {
            self.scratch.insert(id, var);
        } else {
            self.inner.insert(id, var);
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len() + self.scratch.len()
    }

    pub fn get_cl(&self) -> Vec<Scalar> {
        self.inner.iter().map(|v| match v {
            (_, Variable::Inner { id: _, G: _, cl, cr: _ }) => cl.clone(),
            _ => panic!()
        }).chain(self.scratch.iter().map(|v| match v {
            (_, Variable::Scratch { id: _, cl, cr: _ }) => cl.clone(),
            _ => panic!()
        })).collect()
    }

    pub fn get_cr(&self) -> Vec<Scalar> {
        self.inner.iter().map(|v| match v {
            (_, Variable::Inner { id: _, G: _, cl: _, cr }) => cr.clone(),
            _ => panic!()
        }).chain(self.scratch.iter().map(|v| match v {
            (_, Variable::Scratch { id: _, cl: _, cr }) => cr.clone(),
            _ => panic!()
        })).collect()
    }

    pub fn get_G(&self, w: &Scalar, P: &Vec<G1Affine>, Gprime: &Vec<G1Affine>) -> Vec<G1Affine> {
        let mut Gw: Vec<G1Affine> = self.inner.iter().zip(P).map(|(v, iP)| match v {
            (_, Variable::Inner { id: _, G: gP, cl: _, cr: _ }) => (gP * w + iP).to_affine(),
            _ => panic!()
        }).collect();

        Gw.extend(Gprime);
        Gw
    }

    pub fn get_challenge(&self, transcript: &mut Transcript) -> Vec<G1Affine> {
        self.inner
            .keys()
            .map(|_| transcript.challenge_point("blinding G".as_bytes()))
            .collect()
    }

    pub fn blind(mut self) -> Self {
        self.inner.iter_mut().for_each(|mut var| {
            match &mut var {
                (_, Variable::Inner { id: _, G: _, ref mut cl, ref mut cr }) => {
                    *cl = Scalar::zero();
                    *cr = Scalar::zero();
                },
                _ => panic!()
            };
        });
        self.scratch.iter_mut().for_each(|mut var| {
            match &mut var {
                (_, Variable::Scratch { id: _, ref mut cl, ref mut cr }) => {
                    *cl = Scalar::zero();
                    *cr = Scalar::zero();
                }
                _ => panic!()
            };
        });

        self
    }

    pub fn get_constraints(&self, constraints: &[Constraint], z: &Scalar) -> (Vec<Scalar>, Vec<Scalar>, Vec<Scalar>, Vec<Scalar>, Vec<Scalar>, Vec<Scalar>, Vec<Scalar>, Scalar) {
        let width = self.len();
        let mut delta = Scalar::zero();
        let mut theta = vec![Scalar::zero(); width];
        let mut mu = vec![Scalar::zero(); width];
        let mut nu = vec![Scalar::zero(); width];
        let mut omega = vec![Scalar::zero(); width];

        let mut sep = z.clone();
        for constraint in constraints {
            match constraint {
                Constraint::Dir { left, result } => {
                    let lvec: Vec<Scalar> = self.inner
                        .keys()
                        .map(|e| match left.get(e) {
                            Some(x) => x.clone(),
                            None => Scalar::zero()
                        })
                        .chain(
                            self.scratch
                                .keys()
                                .map(|e| match left.get(e) {
                                    Some(x) => x.clone(),
                                    None => Scalar::zero()
                                })
                        )
                        .collect();
                    
                    mu = add_vec(&mu, &smul_vec(&sep, &lvec));
                    delta = delta + sep * result;
                },
                Constraint::Mul { right, result } => {
                    let rvec: Vec<Scalar> = self.inner
                        .keys()
                        .map(|e| match right.get(e) {
                            Some(x) => x.clone(),
                            None => Scalar::zero()
                        })
                        .chain(
                            self.scratch
                                .keys()
                                .map(|e| match right.get(e) {
                                    Some(x) => x.clone(),
                                    None => Scalar::zero()
                                })
                        )
                        .collect();

                    theta = add_vec(&theta, &smul_vec(&sep, &rvec));
                    delta = delta + sep * result;
                },
                Constraint::One { right, result } => {
                    let rvec: Vec<Scalar> = self.inner
                        .keys()
                        .map(|e| match right.get(e) {
                            Some(x) => x.clone(),
                            None => Scalar::zero()
                        })
                        .chain(
                            self.scratch
                                .keys()
                                .map(|e| match right.get(e) {
                                    Some(x) => x.clone(),
                                    None => Scalar::zero()
                                })
                        )
                        .collect();
                    mu = add_vec(&mu, &smul_vec(&sep, &rvec));
                    nu = add_vec(&nu, &smul_vec(&sep, &rvec));
                    delta = delta + sep * result;
                },
                Constraint::Sum { left, right, result } => {
                    let lvec: Vec<Scalar> = self.inner
                        .keys()
                        .map(|e| match left.get(e) {
                            Some(x) => x.clone(),
                            None => Scalar::zero()
                        })
                        .chain(
                            self.scratch
                                .keys()
                                .map(|e| match left.get(e) {
                                    Some(x) => x.clone(),
                                    None => Scalar::zero()
                                })
                        )
                        .collect();
                    let rvec: Vec<Scalar> = self.inner
                        .keys()
                        .map(|e| match right.get(e) {
                            Some(x) => x.clone(),
                            None => Scalar::zero()
                        })
                        .chain(
                            self.scratch
                                .keys()
                                .map(|e| match right.get(e) {
                                    Some(x) => x.clone(),
                                    None => Scalar::zero()
                                })
                        )
                        .collect();
                    mu = add_vec(&mu, &smul_vec(&sep, &lvec));
                    omega = add_vec(&omega, &smul_vec(&sep, &rvec));
                    delta = delta + sep * result;
                }
            }
            sep = &sep * z;
        }

        let inv_theta: Vec<Scalar> = theta.iter().map(|v| v.invert().unwrap_or(Scalar::zero())).collect();

        let alpha = mul_vec(&inv_theta, &sub_vec(&omega, &nu));
        let beta = mul_vec(&inv_theta, &mu);
        delta = delta + inner_product(&alpha, &mu) + inner_product(&vec![Scalar::one(); width], &nu);

        (theta, inv_theta, mu, nu, omega, alpha, beta, delta)
    }

}

pub enum ConstraintType {
    Sum,
    Mul,
    One,
    Dir
}

impl Constraint {
    pub fn new(constraint_type: ConstraintType, result: Scalar) -> Constraint {
        match constraint_type {
            ConstraintType::Sum => {
                Constraint::Sum {
                    left: HashMap::new(),
                    right: HashMap::new(),
                    result: result 
                }
            },
            ConstraintType::Mul => {
                Constraint::Mul {
                    right: HashMap::new(),
                    result: result 
                }
            },
            ConstraintType::One => {
                Constraint::One {
                    right: HashMap::new(),
                    result: result 
                }
            },
            ConstraintType::Dir => {
                Constraint::Dir {
                    left: HashMap::new(),
                    result: result 
                }
            }
        }
    }

    pub fn left_set(&mut self, k: &str, v: Scalar) {
        match self {
            Constraint::Sum { left, right: _, result: _ } => {
                left.insert(k.to_string(), v);
            },
            Constraint::Dir { left, result: _ } => {
                left.insert(k.to_string(), v);
            },
            _ => {}
        }
    }

    pub fn right_set(&mut self, k: &str, v: Scalar) {
        match self {
            Constraint::Sum { left: _, right, result: _ } => {
                right.insert(k.to_string(), v);
            },
            Constraint::Mul { right, result: _ } => {
                right.insert(k.to_string(), v);
            },
            Constraint::One { right, result: _ } => {
                right.insert(k.to_string(), v);
            },
            _ => {}
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Variable {
    Inner {
        id: String,
        #[serde(with = "crate::serialization::G1Affine")]
        G: G1Affine,
        #[serde(skip)]
        cl: Scalar,
        #[serde(skip)]
        cr: Scalar
    },

    Scratch {
        id: String,
        #[serde(skip)]
        cl: Scalar,
        #[serde(skip)]
        cr: Scalar,
    }
}
impl Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Variable::Inner { id, G, cl, cr } => {
                id.hash(state);
                G.to_uncompressed().hash(state);
                cl.to_bytes().hash(state);
                cr.to_bytes().hash(state);
            },
            Variable::Scratch { id, cl, cr } => {
                id.hash(state);
                cl.to_bytes().hash(state);
                cr.to_bytes().hash(state);
            }
        };
    }
}

impl Variable {
    pub fn get_id(&self) -> &str {
        match self {
            Variable::Scratch { id, cl: _, cr: _} => id,
            Variable::Inner { id, G: _, cl: _, cr: _} => id,
        }
    }

    pub fn is_scratch(&self) -> bool {
        match self {
            Variable::Scratch {id: _, cl: _, cr: _} => true,
            _ => false,
        }
    }
}

pub trait ProofInput {
    fn commit(&self, transcript: &mut Transcript);
}

pub trait Proof<P: ProofInput + Clone, S: Default> {
    fn get_variables(inputs: &P, secrets: &S, u: &Scalar) -> Variables;
    fn get_constraints(inputs: &P, y: &Scalar) -> Vec<Constraint>;
    fn additional_checks(_inputs: &P) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericProof<P: ProofInput + Clone> {
    pub vars: Variables,
    pub inputs: P,
    #[serde(with = "crate::serialization::G1Affine")]
    pub A: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub S: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub T1: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub T2: G1Affine,
    #[serde(with = "crate::serialization::Scalar")]
    pub tau: Scalar,
    #[serde(with = "crate::serialization::Scalar")]
    pub r: Scalar,
    #[serde(with = "crate::serialization::Scalar")]
    pub t: Scalar,
    pub ipp_proof: InnerProductProof
}

impl<P: ProofInput + Clone> GenericProof<P> {
    pub fn proove<S: Default, F: Proof<P, S>>(transcript: &mut Transcript, inputs: P, secrets: S) -> Result<GenericProof<P>, ProofError> {
        let mut csrng = rand::thread_rng();

        // get temporary vars
        let vars = F::get_variables(&inputs, &secrets, &Scalar::zero());

        let m = vars.len();
        transcript.append_u64(b"m", m as u64);

        // commit public inputs
        inputs.commit(transcript);

        let u = transcript.challenge_scalar(b"u for exponents");
        let F = transcript.challenge_point(b"F for vec-com");

        let vars = F::get_variables(&inputs, &secrets, &u);

        let GP: Vec<G1Affine> = vars.get_challenge(transcript);

        let Gprime: Vec<G1Affine> = (0..(vars.len() - GP.len())).map(|_| transcript.challenge_point(b"Gtypes")).collect();
        let G0 = vars.get_G(&Scalar::zero(), &GP, &Gprime);
        let H: Vec<G1Affine> = G0.iter().map(|_| transcript.challenge_point(b"blinding Ps")).collect();

        let cl: Vec<Scalar> = vars.get_cl();
        let cr: Vec<Scalar> = vars.get_cr();

        let rA = Scalar::random(&mut csrng);
        let A = (F * rA + vartime_multiscalar_mul(&cl, &G0) + vartime_multiscalar_mul(&cr, &H)).to_affine();
        transcript.append_point(b"A commitment", &A);

        if cfg!(debug_assertions) {
            let G_test = vars.get_G(&Scalar::random(&mut csrng), &GP, &Gprime);
            let A_test = (F * rA + vartime_multiscalar_mul(&cl, &G_test) + vartime_multiscalar_mul(&cr, &H)).to_affine();
            assert_eq!(A_test, A, "A should be independent of w");
        }

        let w = transcript.challenge_scalar(b"w");
        let Gw = vars.get_G(&w, &GP, &Gprime);

        let sl: Vec<Scalar> = (0..Gw.len()).map(|_| Scalar::random(&mut csrng)).collect();
        let sr: Vec<Scalar> = cr.iter().map(|c| match c == &Scalar::zero() {
            true => Scalar::zero(),
            false => Scalar::random(&mut csrng)
        }).collect();

        let rS = Scalar::random(&mut csrng);
        let S = (F * rS + vartime_multiscalar_mul(&sl, &Gw) + vartime_multiscalar_mul(&sr, &H)).to_affine();
        transcript.append_point(b"S commitment", &S);

        let y = transcript.challenge_scalar(b"y");
        let z = transcript.challenge_scalar(b"z");

        // blind vars
        let vars = vars.blind();

        let constraints = F::get_constraints(&inputs, &y);
        let (theta, inv_theta, mu, _nu, _omega, alpha, beta, delta) = vars.get_constraints(&constraints, &z);

        let l_x = VecPoly1(add_vec(&cl,&alpha),sl.clone());
        let r_x = VecPoly1(add_vec(&mul_vec(&theta, &cr),&mu),mul_vec(&theta,&sr));

        let t_x = l_x.inner_product(&r_x);

        let tau_1 = Scalar::random(&mut csrng);
        let tau_2 = Scalar::random(&mut csrng);

        let T1 = (G1Affine::identity() * t_x.1 + F * tau_1).to_affine();
        let T2 = (G1Affine::identity() * t_x.2 + F * tau_2).to_affine();

        transcript.append_point(b"T1 commitment", &T1);
        transcript.append_point(b"T2 commitment", &T2);

        let x = transcript.challenge_scalar(b"x");

        let tau = tau_1*x+tau_2*x*x;
        let r = rA + x*rS;
        let lvec = l_x.eval(x);
        let rvec = r_x.eval(x);
        let t = t_x.eval(x);

        transcript.append_scalar(b"tau", &tau);
        transcript.append_scalar(b"r", &r);
        transcript.append_scalar(b"t", &t);

        if cfg!(debug_assertions) {
            let lhs = (F * r + vartime_multiscalar_mul(&lvec, &Gw) + vartime_multiscalar_mul(&mul_vec(&inv_theta, &rvec), &H)).to_affine();
            let rhs = (A + S * x + vartime_multiscalar_mul(&alpha, &Gw) + vartime_multiscalar_mul(&beta, &H)).to_affine();
            println!("lhs: {:#?}", lhs);
            println!("rhs: {:#?}", rhs);
            println!("t_x: {:#?}", t_x.0);
            println!("delta: {:#?}", delta);
            assert_eq!(lhs, rhs, "verification eq should hold");
            assert_eq!(t_x.0, delta, "offset should be delta");
            assert_eq!(t, t_x.0 + x*t_x.1 + x*x*t_x.2, "polynomial holds");
        }

        // Get a challenge value to combine statements for the IPP
        let ippw = transcript.challenge_scalar(b"ippw");
        let Q = (G1Affine::identity() * ippw).to_affine();

        // pad to next power of two
        let padlen = m.next_power_of_two() - m;

        let mut G_factors: Vec<Scalar> = iter::repeat(Scalar::one()).take(m).collect();
        G_factors.extend(vec![Scalar::zero(); padlen]);
        let mut H_factors: Vec<Scalar> = inv_theta;
        H_factors.extend(vec![Scalar::zero(); padlen]);

        let mut lvec = lvec.clone();
        let mut rvec = rvec.clone();
        lvec.extend(vec![Scalar::zero(); padlen]);
        rvec.extend(vec![Scalar::zero(); padlen]);

        let mut Gw = Gw;
        let mut H = H;
        for _ in 0..padlen {
            Gw.push(transcript.challenge_point(b"padding G"));
            H.push(transcript.challenge_point(b"padding H"));
        }

        let ipp_proof = inner_product_proof::InnerProductProof::create(
            transcript,
            &Q,
            &G_factors,
            &H_factors,
            Gw.clone(),
            H.clone(),
            lvec.clone(),
            rvec.clone(),
        );

        Ok(GenericProof { A, S, T1, T2, tau, r, ipp_proof, t, vars, inputs })
    }

    pub fn verify<S: Default, F: Proof<P, S>>(&self, transcript: &mut Transcript) -> Result<(), ProofError> {
        let m = self.vars.len();
        transcript.append_u64(b"m", m as u64);

        // commit public inputs
        self.inputs.commit(transcript);

        let u = transcript.challenge_scalar(b"u for exponents");
        let F = transcript.challenge_point(b"F for vec-com");

        let vars = F::get_variables(&self.inputs, &S::default(), &u).blind();
        if vars != self.vars {
            return Err(ProofError::VerificationError)
        }
        let vars = &self.vars;

        let GP: Vec<G1Affine> = vars.get_challenge(transcript);

        let Gprime: Vec<G1Affine> = (0..(self.vars.len() - GP.len())).map(|_| transcript.challenge_point(b"Gtypes")).collect();
        let G0 = vars.get_G(&Scalar::zero(), &GP, &Gprime);
        let H: Vec<G1Affine> = G0.iter().map(|_| transcript.challenge_point(b"blinding Ps")).collect();

        transcript.append_point(b"A commitment", &self.A);

        let w = transcript.challenge_scalar(b"w");

        transcript.append_point(b"S commitment", &self.S);

        let y = transcript.challenge_scalar(b"y");
        let z = transcript.challenge_scalar(b"z");

        transcript.append_point(b"T1 commitment", &self.T1);
        transcript.append_point(b"T2 commitment", &self.T2);

        let x = transcript.challenge_scalar(b"x");
        let Gw = vars.get_G(&w, &GP, &Gprime);

        transcript.append_scalar(b"tau", &self.tau);
        transcript.append_scalar(b"r", &self.r);
        transcript.append_scalar(b"t", &self.t);

        let constraints = F::get_constraints(&self.inputs, &y);
        let (_theta, inv_theta, _mu, _nu, _omega, alpha, beta, delta) = vars.get_constraints(&constraints, &z);

        let ippw = transcript.challenge_scalar(b"ippw");
        let Q = (G1Affine::identity() * ippw).to_affine();

        let ipPmQ = vartime_multiscalar_mul(
            iter::once(&Scalar::one())
                .chain(iter::once(&x))
                .chain(iter::once(&-self.r))
                .chain(iter::once(&self.t))
                .chain(alpha.iter())
                .chain(beta.iter()),
            iter::once(&self.A)
                .chain(iter::once(&self.S))
                .chain(iter::once(&F))
                .chain(iter::once(&Q))
                .chain(Gw.iter())
                .chain(H.iter()));

        // pad to next power of two
        let padlen = m.next_power_of_two() - m;

        let mut G_factors: Vec<Scalar> = iter::repeat(Scalar::one()).take(m).collect();
        G_factors.extend(vec![Scalar::zero(); padlen]);
        let mut H_factors: Vec<Scalar> = inv_theta;
        H_factors.extend(vec![Scalar::zero(); padlen]);

        let mut Gw = Gw;
        let mut H = H;
        for _ in 0..padlen {
            Gw.push(transcript.challenge_point(b"padding G"));
            H.push(transcript.challenge_point(b"padding H"));
        }

        if self.ipp_proof.verify(Gw.len(), transcript, G_factors, H_factors, &ipPmQ, &Q, &Gw, &H).is_err() {
            return Err(ProofError::VerificationError)
        }

        let lnd = G1Affine::identity() * self.t + F * self.tau;
        let rnd = G1Affine::identity() * delta + self.T1 * x + self.T2 * x * x;

        if lnd.to_affine() != rnd.to_affine() {
            return Err(ProofError::VerificationError)
        }

        if !F::additional_checks(&self.inputs) {
            return Err(ProofError::VerificationError)
        }


        Ok(())
    }
}
