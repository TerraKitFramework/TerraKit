//! Deterministic seed values and domain-based seed derivation.

/// Stable root seed used by deterministic TerraKit generation algorithms.
///
/// `GenerationSeed` is a transparent wrapper around `u64` so callers can keep
/// explicit seed boundaries without relying on platform-dependent hashers,
/// execution order, or mutable global state. It is not cryptographic.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenerationSeed(
    /// Raw deterministic seed value.
    pub u64,
);

impl GenerationSeed {
    /// Creates a generation seed from its raw integer value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw integer seed value.
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Derives a deterministic child seed for a stage or algorithm domain.
    ///
    /// The derivation uses explicit wrapping arithmetic and a SplitMix64-style
    /// finalizer. It depends only on this seed and the supplied domain, leaves
    /// the parent seed unchanged, and is stable across supported platforms.
    pub fn derive(self, domain: SeedDomain) -> Self {
        let value = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15)
            ^ domain.0.wrapping_mul(0xBF58_476D_1CE4_E5B9);

        Self(mix_u64(value))
    }
}

/// Stable domain identifier for deriving independent deterministic seeds.
///
/// Use separate domains for unrelated stages or algorithms that share a parent
/// `GenerationSeed`. Domains are deterministic identifiers only; they are not
/// plugin identifiers and are not cryptographic salts.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeedDomain(
    /// Raw deterministic domain value.
    pub u64,
);

impl SeedDomain {
    /// Creates a seed domain from its raw integer value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw integer domain value.
    pub const fn value(self) -> u64 {
        self.0
    }
}

fn mix_u64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_seeds_are_deterministic() {
        let seed = GenerationSeed::new(123);
        let domain = SeedDomain::new(7);

        assert_eq!(seed.derive(domain), seed.derive(domain));
    }

    #[test]
    fn different_domains_normally_produce_different_seeds() {
        let seed = GenerationSeed::new(123);

        assert_ne!(
            seed.derive(SeedDomain::new(7)),
            seed.derive(SeedDomain::new(8))
        );
    }

    #[test]
    fn deriving_seed_does_not_mutate_parent_seed() {
        let seed = GenerationSeed::new(123);
        let original = seed;

        let _ = seed.derive(SeedDomain::new(7));

        assert_eq!(seed, original);
    }

    #[test]
    fn zero_and_max_seed_values_can_be_derived() {
        let zero = GenerationSeed::new(0).derive(SeedDomain::new(0));
        let max = GenerationSeed::new(u64::MAX).derive(SeedDomain::new(u64::MAX));

        assert_eq!(zero, GenerationSeed::new(0).derive(SeedDomain::new(0)));
        assert_eq!(
            max,
            GenerationSeed::new(u64::MAX).derive(SeedDomain::new(u64::MAX))
        );
    }
}
