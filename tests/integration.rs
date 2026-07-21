// Copyright (c) 2026 Ronnie Andrews, Jr. (Team Xcelerator Inc.(R))
// All rights reserved. See LICENSE file for terms.

//! Integration tests for the ccm-reproduction binary.
//!
//! These tests verify that the toolkit dependency is correctly wired
//! and the binary produces expected results for known configurations.

use xc_spectral::ccm::{prime_powers_up_to, CcmParams};

/// CcmParams should produce correct values for the headline config.
#[test]
fn headline_config_params() {
    let params = CcmParams::from_lambda_sq_integer(13, 120);
    assert!((params.lambda_squared() - 13.0).abs() < 1e-12);
    assert_eq!(params.matrix_size(), 241);
}

/// Prime powers up to lambda^2=13 should give 9 entries (the CCM headline).
#[test]
fn prime_powers_lambda_sq_13() {
    let pp = prime_powers_up_to(13);
    assert_eq!(pp.len(), 9, "lambda^2=13 should have 9 prime powers");
}

/// The toolkit-owned canonical reference zeros should be loadable.
#[test]
fn bundled_reference_zeros_loadable() {
    let strings = xc_zeta::zeros::bundled_first_n_strings(10).unwrap();
    let zeros: Vec<f64> = strings
        .iter()
        .map(|zero| zero.parse::<f64>().unwrap())
        .collect();
    assert_eq!(zeros.len(), 10);
    // First zero should be ~14.13
    assert!((zeros[0] - 14.134725).abs() < 0.001);
}

/// f64 tier should run without panicking at small N.
#[test]
fn f64_tier_runs() {
    let params = CcmParams::from_lambda_sq_integer(13, 5);
    let result = xc_spectral::ccm::run_f64(&params).unwrap();
    assert!(!result.eigenvalues_pos.is_empty());
    assert!(result.elapsed_seconds > 0.0);
}

/// The paper harness binds the finalized reference-free and sector APIs
/// without executing an expensive HP calculation during ordinary tests.
#[cfg(feature = "hp")]
#[test]
fn finalized_hp_research_apis_are_available() {
    use xc_spectral::ccm::hp::{CcmSectorGapHp, HighPrecConfig, HighPrecResult};
    use xc_spectral::ccm::window::ZeroTarget;

    let target = ZeroTarget::IndexRange {
        first: 10,
        last: 12,
    };
    assert_eq!(
        target,
        ZeroTarget::IndexRange {
            first: 10,
            last: 12
        }
    );

    let config = HighPrecConfig::for_decimal_digits(200);
    assert_eq!(config.n_eigenvalues, 50);

    let _: fn(&CcmParams, &HighPrecConfig, &ZeroTarget) -> anyhow::Result<HighPrecResult> =
        xc_spectral::ccm::hp::run_independent;
    let _: fn(&CcmParams, &HighPrecConfig, usize) -> anyhow::Result<CcmSectorGapHp> =
        xc_spectral::ccm::hp::analyze_sector_gap;
}
