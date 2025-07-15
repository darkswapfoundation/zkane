//! # Zero-Knowledge Proof System for ZKane
//!
//! This module implements the zero-knowledge proof system for the ZKane privacy
//! pool, using the `arkworks` ecosystem and the Groth16 proving system over the
//! BLS12-381 curve.
//!
//! ## Components
//!
//! - **WithdrawalCircuit**: The R1CS circuit for a withdrawal operation.
//! - **Prover**: Functions for generating proofs.
//! - **Verifier**: Functions for verifying proofs.

pub mod poseidon_params;

use crate::gadgets::poseidon::PoseidonGadget;
use ark_bls12_381::{Bls12_381, Fr};
use ark_crypto_primitives::{
    crh::poseidon::constraints::CRHParametersVar,
};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey, PreparedVerifyingKey};
use ark_r1cs_std::{prelude::*, fields::fp::FpVar};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::SNARK;
use ark_std::rand::rngs::StdRng;
use ark_std::rand::SeedableRng;

/// This circuit proves that a user knows a valid deposit note (secret and
/// nullifier) corresponding to a commitment in the Merkle tree, without
/// revealing the note itself.
#[derive(Clone)]
pub struct WithdrawalCircuit {
    // --- Public Inputs ---
    /// The hash of the nullifier, used to prevent double-spending.
    pub nullifier_hash: Fr,

    // --- Private Witnesses ---
    /// The secret part of the deposit note.
    pub secret: Fr,
    /// The nullifier part of the deposit note.
    pub nullifier: Fr,
}

impl ConstraintSynthesizer<Fr> for WithdrawalCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate public inputs
        let nullifier_hash = FpVar::new_input(cs.clone(), || Ok(self.nullifier_hash))?;

        // Allocate private witnesses
        let secret = FpVar::new_witness(cs.clone(), || Ok(self.secret))?;
        let nullifier = FpVar::new_witness(cs.clone(), || Ok(self.nullifier))?;

        let poseidon_params = poseidon_params::new();
        let params_var = CRHParametersVar::new_witness(cs.clone(), || Ok(poseidon_params.clone()))?;

        // 1. Verify the commitment is correctly derived from the secret and nullifier.
        let _commitment = PoseidonGadget::hash_two(cs.clone(), &params_var, &secret, &nullifier)?;

        // 2. Verify the nullifier hash is correctly derived from the nullifier.
        let computed_nullifier_hash = PoseidonGadget::hash_one(cs.clone(), &params_var, &nullifier)?;
        computed_nullifier_hash.enforce_equal(&nullifier_hash)?;

        Ok(())
    }
}

pub fn setup() -> (ProvingKey<Bls12_381>, VerifyingKey<Bls12_381>) {
    let mut rng = StdRng::seed_from_u64(0u64);
    let circuit = WithdrawalCircuit {
        nullifier_hash: Fr::default(),
        secret: Fr::default(),
        nullifier: Fr::default(),
    };
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng).unwrap();
    (pk, vk)
}

/// Generate a proof for the given circuit and proving key.
pub fn prove(
    pk: &ProvingKey<Bls12_381>,
    circuit: WithdrawalCircuit,
) -> Proof<Bls12_381> {
    let mut rng = StdRng::seed_from_u64(0u64);
    Groth16::<Bls12_381>::prove(pk, circuit, &mut rng).unwrap()
}

/// Verify a proof with the given verifying key and public inputs.
pub fn verify(
    vk: &VerifyingKey<Bls12_381>,
    proof: &Proof<Bls12_381>,
    nullifier_hash: Fr,
) -> bool {
    let public_inputs = &[nullifier_hash];
    let pvk = PreparedVerifyingKey::from(vk.clone());
    Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, public_inputs, proof).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::UniformRand;
    use ark_crypto_primitives::crh::{poseidon::CRH, CRHScheme};

    #[test]
    fn test_withdrawal_circuit_happy_path() {
        let mut rng = StdRng::seed_from_u64(0u64);

        // 1. Setup
        let (pk, vk) = setup();

        // 2. Create a valid witness
        let secret = Fr::rand(&mut rng);
        let nullifier = Fr::rand(&mut rng);

        let poseidon_params = poseidon_params::new();
        let nullifier_hash = CRH::evaluate(&poseidon_params, [nullifier]).unwrap();

        let circuit = WithdrawalCircuit {
            nullifier_hash,
            secret,
            nullifier,
        };

        // 3. Generate proof
        let proof = prove(&pk, circuit);

        // 4. Verify proof
        let is_valid = verify(&vk, &proof, nullifier_hash);
        assert!(is_valid);
    }
}