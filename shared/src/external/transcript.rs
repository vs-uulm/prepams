use merlin::Transcript;

use group::Curve;
use bls12_381::{G1Affine, G1Projective, Scalar};
use bls12_381::hash_to_curve::{HashToCurve, ExpandMsgXmd};

use super::error::ProofError;

pub trait TranscriptProtocol {
    /// Append a domain separator for an `n`-bit, `m`-party range proof of different languages.
    fn vsigma_domain_sep(&mut self, n: u64);
    fn innerproduct_domain_sep(&mut self, n: u64);
    fn ringsig_domain_sep(&mut self, n: u64);
    fn sealsig_domain_sep(&mut self, n: u64, m: u64);

    /// Append a `scalar` with the given `label`.
    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar);

    /// Append a `point` with the given `label`.
    fn append_point(&mut self, label: &'static [u8], point: &G1Affine);

    /// Append a `byte` with the given `label`.
    fn append_byte(&mut self, label: &'static [u8], byte: u8);

    /// Check that a point is not the identity, then append it to the
    /// transcript.  Otherwise, return an error.
    fn validate_and_append_point(
        &mut self,
        label: &'static [u8],
        point: &G1Affine,
    ) -> Result<(), ProofError>;

    /// Compute a `label`ed challenge variable.
    fn challenge_scalar(&mut self, label: &'static [u8]) -> Scalar;
    fn challenge_point(&mut self, label: &'static [u8]) -> G1Affine;
}

impl TranscriptProtocol for Transcript {
    fn vsigma_domain_sep(&mut self, n: u64) {
        self.append_message(b"dom-sep", b"vecsigma v1");
        self.append_u64(b"n", n);
    }

    fn innerproduct_domain_sep(&mut self, n: u64) {
        self.append_message(b"dom-sep", b"ipp v1");
        self.append_u64(b"n", n);
    }

    fn ringsig_domain_sep(&mut self, n: u64) {
        self.append_message(b"dom-sep", b"ringsig v1");
        self.append_u64(b"n", n);
    }

    fn sealsig_domain_sep(&mut self, n: u64, m: u64) {
        self.append_message(b"dom-sep", b"sealsig v1");
        self.append_u64(b"n", n);
        self.append_u64(b"m", m);
    }

    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar) {
        self.append_message(label, &scalar.to_bytes());
    }

    fn append_point(&mut self, label: &'static [u8], point: &G1Affine) {
        self.append_message(label, &point.to_compressed());
    }

    fn append_byte(&mut self, label: &'static [u8], byte: u8) {
        self.append_message(label, &[byte]);
    }

    fn validate_and_append_point(
        &mut self,
        label: &'static [u8],
        point: &G1Affine,
    ) -> Result<(), ProofError> {
        if point.is_identity().into() {
            Err(ProofError::VerificationError)
        } else {
            Ok(self.append_message(label, &point.to_compressed()))
        }
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
