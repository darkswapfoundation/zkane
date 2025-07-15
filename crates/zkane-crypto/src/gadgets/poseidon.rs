//! # Poseidon Hash Gadget
//!
//! This module provides R1CS constraints for the Poseidon hash function.

use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::fields::fp::FpVar;
use ark_ff::PrimeField;
use ark_crypto_primitives::{
    crh::{
        poseidon::constraints::{CRHGadget, CRHParametersVar},
        CRHSchemeGadget,
    },
    sponge::Absorb,
};

/// A gadget for the Poseidon hash function.
pub struct PoseidonGadget;

impl PoseidonGadget {
    /// Hashes two field elements.
    pub fn hash_two<F: PrimeField + Absorb>(
        _cs: ConstraintSystemRef<F>,
        params: &CRHParametersVar<F>,
        left: &FpVar<F>,
        right: &FpVar<F>,
    ) -> Result<FpVar<F>, SynthesisError> {
        let input = &[left.clone(), right.clone()];
        CRHGadget::evaluate(params, input)
    }

    /// Hashes a single field element.
    pub fn hash_one<F: PrimeField + Absorb>(
        _cs: ConstraintSystemRef<F>,
        params: &CRHParametersVar<F>,
        input: &FpVar<F>,
    ) -> Result<FpVar<F>, SynthesisError> {
        let input = &[input.clone()];
        CRHGadget::evaluate(params, input)
    }
}