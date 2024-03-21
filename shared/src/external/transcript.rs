use merlin::Transcript;

use group::Curve;
use bls12_381::{G1Affine, G2Affine, Gt, G1Projective, Scalar};
use bls12_381::hash_to_curve::{HashToCurve, ExpandMsgXmd};

use crate::serialization::SerializableGt;

pub trait TranscriptProtocol {
    // Append a `u64` with the given `label`.
    fn append_u64(&mut self, label: &'static [u8], value: u64);

    // Append a `scalar` with the given `label`.
    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar);

    // Append a `point` on G1 with the given `label`.
    fn append_g1(&mut self, label: &'static [u8], point: &G1Affine);

    // Append a `point` on G2 with the given `label`.
    fn append_g2(&mut self, label: &'static [u8], point: &G2Affine);

    // Append a `point` on GT with the given `label`.
    fn append_gt(&mut self, label: &'static [u8], point: &Gt);

    // Append a `byte` with the given `label`.
    fn append_byte(&mut self, label: &'static [u8], byte: u8);

    // Compute a `label`ed challenge variable.
    fn challenge_scalar(&mut self, label: &'static [u8]) -> Scalar;

    // Compute a `label`ed challenge point.
    fn challenge_point(&mut self, label: &'static [u8]) -> G1Affine;
}

impl TranscriptProtocol for Transcript {
    fn append_u64(&mut self, label: &'static [u8], value: u64) {
        self.append_u64(label, value);
    }

    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar) {
        self.append_message(label, &scalar.to_bytes());
    }

    fn append_g1(&mut self, label: &'static [u8], point: &G1Affine) {
        self.append_message(label, &point.to_compressed());
    }

    fn append_g2(&mut self, label: &'static [u8], point: &G2Affine) {
        self.append_message(label, &point.to_compressed());
    }

    fn append_gt(&mut self, label: &'static [u8], point: &Gt) {
        self.append_message(label, &SerializableGt::to_bytes(point));
    }

    fn append_byte(&mut self, label: &'static [u8], byte: u8) {
        self.append_message(label, &[byte]);
    }

    fn challenge_scalar(&mut self, label: &'static [u8]) -> Scalar {
        let mut buf = [0u8; 64];
        self.challenge_bytes(label, &mut buf);

        Scalar::from_bytes_wide(&buf)
    }

    fn challenge_point(&mut self, label: &'static [u8]) -> G1Affine {
        let mut buf = [0u8; 64];
        self.challenge_bytes(label, &mut buf);

        <G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(&buf, b"challenge_point").to_affine()
    }
}
