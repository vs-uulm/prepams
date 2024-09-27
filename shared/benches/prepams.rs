extern crate prepams_shared;

use std::fmt;

use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use group::Curve;
use prepams_shared::bindings::{issuer::Issuer, organizer::Organizer};
use prepams_shared::bindings::participant::Participant;
use prepams_shared::types::{AttributeConstraint, ConfirmedParticipation, Participation, Resource};
use bls12_381::{Scalar, G1Affine};
use rand::{thread_rng, Rng, RngCore};
use rand::seq::SliceRandom;

struct ParticipationParams {
    num_qualifier: u32,
    tags_per_qualifier: u32,
    num_disqualifier: u32,
    tags_per_disqualifier: u32,
    num_range_constraints: u32,
    range_bit_length: u32,
    num_set_constraints: u32,
    set_size: u32
}

impl fmt::Display for ParticipationParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Q{}-{}/D{}-{}/R{}-{}/S{}-{}",
            self.num_qualifier,
            self.tags_per_qualifier,
            self.num_disqualifier,
            self.tags_per_disqualifier,
            self.num_range_constraints,
            self.range_bit_length,
            self.num_set_constraints,
            self.set_size
        )
    }
}

pub fn registration(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("Register");

    for num_attributes in 0..25 {
        let issuer = Issuer::new(num_attributes, &vec![]);

        let ipk = issuer.publicKey().ok().unwrap();
        let cvk = issuer.verificationKey().ok().unwrap();
        let lvk = issuer.ledgerVerificationKey().ok().unwrap();

        let experiment = format!("A-{}", num_attributes);

        group.bench_with_input(
            BenchmarkId::new("RegisterP1", &experiment),
            &num_attributes,
            |b, num_attributes| {
                b.iter_batched(
                    || {
                        let attributes: Vec<u32> = (0..*num_attributes).map(|_| rng.next_u32()).collect();
                        let mut seed: [u8; 32] = [0; 32];
                        rng.fill_bytes(&mut seed);

                        (seed, attributes)
                    },
                    |(seed, attributes)| {
                        let mut p = Participant::new(black_box("participant@example.org"), &attributes, &lvk);
                        p.requestCredential(&ipk, &cvk, black_box(&seed)).ok().unwrap();
                    },
                    criterion::BatchSize::SmallInput
                );
            }
        );

        group.bench_with_input(
            BenchmarkId::new("RegisterS", &experiment),
            &num_attributes,
            |b, num_attributes| {
                b.iter_batched(
                    || {
                        let attributes: Vec<u32> = (0..*num_attributes).map(|_| rng.next_u32()).collect();
                        let mut p = Participant::new(black_box("participant@example.org"), &attributes, &lvk);
                        let mut seed: [u8; 32] = [0; 32];
                        rng.fill_bytes(&mut seed);
                        let request = p.requestCredential(&ipk, &cvk, black_box(&seed));
                        request.ok().unwrap()
                    },
                    |request| issuer.issueCredential(&request).ok().unwrap(),
                    criterion::BatchSize::SmallInput
                );
            }
        );

        group.bench_with_input(
            BenchmarkId::new("RegisterP2", &experiment),
            &num_attributes,
            |b, num_attributes| {
                b.iter_batched(
                    || {
                        let attributes: Vec<u32> = (0..*num_attributes).map(|_| rng.next_u32()).collect();
                        let mut p = Participant::new(black_box("participant@example.org"), &attributes, &lvk);
                        let mut seed: [u8; 32] = [0; 32];
                        rng.fill_bytes(&mut seed);
                        let request = p.requestCredential(&ipk, &cvk, black_box(&seed)).ok().unwrap();
                        let response = issuer.issueCredential(&request).ok().unwrap();
                        (p, response)
                    },
                    |(mut p, response)| p.retrieveCredential(black_box(&response)).ok().unwrap(),
                    criterion::BatchSize::SmallInput
                );
            }
        );
    }
}

fn gen_participation(issuer: &Issuer, p: &ParticipationParams) -> (Participant, Resource) {
    let mut rng = thread_rng();
    let ipk = issuer.publicKey().ok().unwrap();
    let cvk = issuer.verificationKey().ok().unwrap();
    let lvk = issuer.ledgerVerificationKey().ok().unwrap();

    let constraints: Vec<AttributeConstraint> = (0..(p.num_range_constraints + p.num_set_constraints))
        .map(|i| {
            if i < p.num_range_constraints {
                let mask = (1 << p.range_bit_length) - 1;
                let lower = rng.next_u32() & !mask;
                AttributeConstraint::Range(i, lower, lower + mask)
            } else {
                let elements = (0..p.set_size).map(|_| rng.next_u32()).collect();
                AttributeConstraint::Element(i, elements)
            }
        })
        .collect();

    let attributes: Vec<u32> = constraints.iter().map(|c| match c {
        AttributeConstraint::Range(_, lower, upper) => rng.gen_range(*lower..*upper),
        AttributeConstraint::Element(_, set) => set.choose(&mut rng).unwrap().clone()
    }).collect();
    let mut participant = Participant::new(black_box("p@example.org"), &attributes, &lvk);
    let request = participant.requestCredential(&ipk, &cvk, &[0; 32]).ok().unwrap();
    let response = issuer.issueCredential(&request).ok().unwrap();
    participant.retrieveCredential(&response).ok().unwrap();

    let mut study = Resource::random(&mut rng);

    for _ in 0..p.num_qualifier {
        let qid = <bls12_381::Scalar as ff::Field>::random(&mut rng);
        let tag = participant.credential().unwrap().derive_tag(&qid).unwrap();
        let mut tags: Vec<G1Affine> = (0..(p.tags_per_qualifier.max(1) - 1))
            .map(|_| (G1Affine::generator() * <Scalar as ff::Field>::random(&mut rng)).to_affine())
            .collect();
        tags.push(tag);

        study.addQualifier(qid, tags);
    }

    for _ in 0..p.num_disqualifier {
        let qid = <bls12_381::Scalar as ff::Field>::random(&mut rng);

        let tags: Vec<G1Affine> = (0..p.tags_per_disqualifier)
            .map(|_| (G1Affine::generator() * <Scalar as ff::Field>::random(&mut rng)).to_affine())
            .collect();

        study.addDisqualifier(qid, tags);
    }

    for constraint in constraints {
        study.addConstraint(constraint)
    }

    (participant, study)
}

fn bench_participation(e: &str, v: u32, group: &mut BenchmarkGroup<'_, WallTime>, p: ParticipationParams) {
    let mut issuer = Issuer::new((p.num_set_constraints + p.num_range_constraints) as usize, &vec![]);
    let issuer_ro = Issuer::deserialize(&issuer.serialize().ok().unwrap()).ok().unwrap();
    let ipk = issuer.publicKey().ok().unwrap();
    let organizer = Organizer::new("o@example.org", &ipk, &[0; 32]).ok().unwrap();
    let opk = organizer.publicKey();

    let experiment = format!("{}-{}", e, v);

    group.bench_function(
        BenchmarkId::new("ParticipateP", &experiment),
        |b| {
            b.iter_batched(
                || gen_participation(&issuer, &p),
                |(participant, study)| participant.participate(&study).ok().unwrap(),
                criterion::BatchSize::SmallInput
            );
        }
    );

    group.bench_function(
        BenchmarkId::new("ParticipateO", &experiment),
        |b| {
            b.iter_batched(
                || {
                    let (participant, study) = gen_participation(&issuer, &p);
                    let participation = participant.participate(&study).ok().unwrap();
                    Participation::deserialize(&participation).ok().unwrap()
                },
                |participation| {
                    participation.verify().ok().unwrap();
                    organizer.confirmParticipation(&participation, black_box(String::default())).ok().unwrap();
                },
                criterion::BatchSize::SmallInput
            );
        }
    );

    group.bench_function(
        BenchmarkId::new("ParticipateS", &experiment),
        |b| {
            b.iter_batched(
                || {
                    let (participant, study) = gen_participation(&issuer_ro, &p);
                    let participation = participant.participate(&study).ok().unwrap();
                    let participation = Participation::deserialize(&participation).ok().unwrap();

                    let confirmed = organizer.confirmParticipation(&participation, String::default()).ok().unwrap();
                    ConfirmedParticipation::deserialize(&confirmed).ok().unwrap()
                },
                |confirmed| issuer.issueReward(&confirmed, black_box(&opk), 1).ok().unwrap(),
                criterion::BatchSize::SmallInput
            );
        }
    );
}

pub fn participation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Participation");

    for num_qualifier in 0..65 {
        bench_participation("Q", num_qualifier, &mut group, ParticipationParams {
            num_qualifier: num_qualifier,
            tags_per_qualifier: 1,
            num_disqualifier: 0,
            tags_per_disqualifier: 0,
            num_range_constraints: 0,
            range_bit_length: 0,
            num_set_constraints: 0,
            set_size: 0
        });
    }

    for tags_per_qualifier in 0..65 {
        bench_participation("QT", tags_per_qualifier, &mut group, ParticipationParams {
            num_qualifier: 1,
            tags_per_qualifier: tags_per_qualifier,
            num_disqualifier: 0,
            tags_per_disqualifier: 0,
            num_range_constraints: 0,
            range_bit_length: 0,
            num_set_constraints: 0,
            set_size: 0
        });
    }

    for num_disqualifier in 0..65 {
        bench_participation("D", num_disqualifier, &mut group, ParticipationParams {
            num_qualifier: 0,
            tags_per_qualifier: 0,
            num_disqualifier: num_disqualifier,
            tags_per_disqualifier: 1,
            num_range_constraints: 0,
            range_bit_length: 0,
            num_set_constraints: 0,
            set_size: 0
        });
    }

    for tags_per_disqualifier in 0..65 {
        bench_participation("DT", tags_per_disqualifier, &mut group, ParticipationParams {
            num_qualifier: 0,
            tags_per_qualifier: 0,
            num_disqualifier: 1,
            tags_per_disqualifier: tags_per_disqualifier,
            num_range_constraints: 0,
            range_bit_length: 0,
            num_set_constraints: 0,
            set_size: 0
        });
    }

    for num in 1..65 {
        bench_participation("R", num, &mut group, ParticipationParams {
            num_qualifier: 0,
            tags_per_qualifier: 0,
            num_disqualifier: 0,
            tags_per_disqualifier: 0,
            num_range_constraints: num,
            range_bit_length: 8,
            num_set_constraints: 0,
            set_size: 0
        });
    }

    for i in 1..65 {
        bench_participation("RL", i, &mut group, ParticipationParams {
            num_qualifier: 0,
            tags_per_qualifier: 0,
            num_disqualifier: 0,
            tags_per_disqualifier: 0,
            num_range_constraints: 1,
            range_bit_length: i,
            num_set_constraints: 0,
            set_size: 0
        });
    }

    for num in 1..65 {
        bench_participation("S", num, &mut group, ParticipationParams {
            num_qualifier: 0,
            tags_per_qualifier: 0,
            num_disqualifier: 0,
            tags_per_disqualifier: 0,
            num_range_constraints: 0,
            range_bit_length: 0,
            num_set_constraints: num,
            set_size: 1
        });
    }

    for size in 1..65 {
        bench_participation("SL", size, &mut group, ParticipationParams {
            num_qualifier: 0,
            tags_per_qualifier: 0,
            num_disqualifier: 0,
            tags_per_disqualifier: 0,
            num_range_constraints: 0,
            range_bit_length: 0,
            num_set_constraints: 1,
            set_size: size
        });
    }
}

pub fn payout(c: &mut Criterion) {
    let mut group = c.benchmark_group("Payout");

    let mut rng = thread_rng();

    let mut issuer = Issuer::new(0, &vec![]);
    let ipk = issuer.publicKey().ok().unwrap();
    let cvk = issuer.verificationKey().ok().unwrap();
    let lvk = issuer.ledgerVerificationKey().ok().unwrap();

    let mut participant = Participant::new(black_box("p@example.org"), &vec![], &lvk);
    let request = participant.requestCredential(&ipk, &cvk, &[0; 32]).ok().unwrap();
    let response = issuer.issueCredential(&request).ok().unwrap();
    participant.retrieveCredential(&response).ok().unwrap();

    group.bench_function(
        BenchmarkId::new("PaddingP1", "L-10"),
        |b| b.iter(|| participant.requestNulls().ok().unwrap())
    );

    let nulls = participant.requestNulls().ok().unwrap();
    let request = nulls.request().ok().unwrap();
    group.bench_function(
        BenchmarkId::new("PaddingS", "L-10"),
        |b| b.iter(|| issuer.issueNulls(&request).ok().unwrap())
    );

    let response = issuer.issueNulls(&request).ok().unwrap();
    group.bench_function(
        BenchmarkId::new("PaddingP2", "L-10"),
        |b| b.iter(|| nulls.clone().unblind(&response).ok().unwrap())
    );

    let organizer = Organizer::new("o@example.org", &ipk, &[0; 32]).ok().unwrap();
    let opk = organizer.publicKey();

    let mut participations: Vec<ConfirmedParticipation> = Vec::new();

    for i in 0..=50 {
        let ledger = issuer.ledger().ok().unwrap();
        let experiment = format!("L-{}", i * 10);
        group.bench_with_input(
            BenchmarkId::new("GetBalance", &experiment),
            &ledger,
            |b, ledger| b.iter(|| participant.getBalance(&ledger).ok().unwrap())
        );

        for _ in 0..10 {
            let study = Resource::random(&mut rng);
            let participation = participant.participate(&study).ok().unwrap();
            let participation = Participation::deserialize(&participation).ok().unwrap();
            let confirmed_participation = organizer.confirmParticipation(&participation, String::default()).ok().unwrap();
            let confirmed_participation = ConfirmedParticipation::deserialize(&confirmed_participation).ok().unwrap();
            issuer.issueReward(&confirmed_participation, &opk, 1).ok().unwrap();
            participations.push(confirmed_participation);
        }
    }

    let issuer_secret = issuer.serialize().ok().unwrap();
    let entries: Vec<&ConfirmedParticipation> = participations.iter().collect();

    for inputs in 1..=10 {
        let experiment = format!("L-{}", inputs);
        group.bench_with_input(
            BenchmarkId::new("PayoutP", &experiment),
            &inputs,
            |b, inputs| {
                b.iter_batched(
                    || {
                        let mut issuer = Issuer::deserialize(&issuer_secret).ok().unwrap();
                        let mut participations: Vec<&ConfirmedParticipation> = entries.clone();
                        participations.shuffle(&mut rng);
                        for participation in participations {
                            issuer.issueReward(&participation, &opk, 1).ok().unwrap();
                        }

                        let nulls = participant.requestNulls().ok().unwrap();
                        let request = nulls.request().ok().unwrap();
                        let response = issuer.issueNulls(&request).ok().unwrap();
                        let nulls = nulls.unblind(&response).ok().unwrap();

                        let ledger = issuer.ledger().ok().unwrap();
                        (ledger, nulls)
                    },
                    |(ledger, nulls)| participant.requestPayout(*inputs, "test", "p@example.org", &nulls, &ledger),
                    criterion::BatchSize::SmallInput
                );
            }
        );

        group.bench_with_input(
            BenchmarkId::new("PayoutS", &experiment),
            &inputs,
            |b, inputs| {
                b.iter_batched(
                    || {
                        let mut issuer = Issuer::deserialize(&issuer_secret).ok().unwrap();
                        let mut participations: Vec<&ConfirmedParticipation> = entries.clone();
                        participations.shuffle(&mut rng);
                        for participation in participations {
                            issuer.issueReward(&participation, &opk, 1).ok().unwrap();
                        }

                        let nulls = participant.requestNulls().ok().unwrap();
                        let request = nulls.request().ok().unwrap();
                        let response = issuer.issueNulls(&request).ok().unwrap();
                        let nulls = nulls.unblind(&response).ok().unwrap();

                        let ledger = issuer.ledger().ok().unwrap();
                        let request = participant.requestPayout(*inputs, "test", "p@example.org", &nulls, &ledger).ok().unwrap();
                        let proof = request.proof().ok().unwrap();
                        (issuer, proof)
                    },
                    |(mut issuer, request)| issuer.checkPayoutRequest(&request).ok().unwrap(),
                    criterion::BatchSize::SmallInput
                );
            }
        );
    }
}

criterion_group!(benches, registration, participation, payout);
criterion_main!(benches);
