// Copyright (c) 2026 Ronnie Andrews, Jr. (Team Xcelerator Inc.(R))
// All rights reserved. See LICENSE file for terms.
//
// This source code is provided for verification and study purposes only.
// Modification, redistribution, and commercial use are prohibited
// without explicit written permission.

//! CCM Zeta Spectral Triple — Reproduction and Convergence Analysis
//!
//! Independent implementation of the Connes-Consani-Moscovici operator
//! construction (arxiv 2511.22755) for empirical study of its convergence
//! properties.
//!
//! Author: Ronnie Andrews, Jr. (Team Xcelerator Inc.(R))

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;

use xc_spectral::ccm::{self, CcmParams, CcmResult};

/// Path to the canonical reference zeros file (1000 zeros at 1000 digits).
const ZEROS_PATH: &str = "data/zeta_zeros.json";

#[derive(Parser)]
#[command(name = "ccm-reproduction", about = "CCM Zeta Spectral Triple — reproduction and convergence analysis")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the CCM construction at given (λ², N) and report eigenvalues
    /// vs Riemann zeros.
    Run {
        /// λ² value. Primes p ≤ λ² enter the Weil form (e.g. 13, 100, 1000).
        #[arg(long, default_value_t = 13_u64)]
        lambda_sq: u64,
        /// Mode cutoff N. Matrix size is 2N+1.
        #[arg(long, default_value_t = 120)]
        n_modes: usize,
        /// How many positive eigenvalues to print.
        #[arg(long, default_value_t = 20)]
        top: usize,
        /// Working precision in decimal digits (requires --features hp).
        #[arg(long, default_value_t = 200)]
        precision_digits: u32,
        /// Significant digits to show per HP value in the eigenvalue table.
        /// Defaults to 16 (slightly above f64 precision); set higher for
        /// publication output (e.g. 50 at HP-1000 to show convergence
        /// past the f64 underflow boundary).
        #[arg(long, default_value_t = 16)]
        display_digits: usize,
        /// Use f64 tier (fast, ~13 digits max) instead of HP. Smoke-test
        /// mode only — f64 cannot reach the precisions needed for the
        /// paper's publication-grade convergence claims.
        #[arg(long, default_value_t = false)]
        f64_only: bool,
        /// Disable the forced-even projection during inverse iteration.
        /// When set, the natural (unprojected) smallest eigenvector is
        /// used to build R(t) and find zeros. If the natural eigenvector
        /// is even (as conjectured), results are identical to the default
        /// forced-even path.
        #[arg(long, default_value_t = false)]
        no_force_even: bool,
    },
    /// Measure the natural evenness of the smallest Weil eigenvector
    /// (Claim 4: symmetry breakdown at large λ).
    CheckEvenness {
        /// λ² value (e.g. 13, 100, 1000, 1200).
        #[arg(long, default_value_t = 13_u64)]
        lambda_sq: u64,
        /// Mode cutoff N.
        #[arg(long, default_value_t = 120)]
        n_modes: usize,
        /// Working precision in decimal digits.
        #[arg(long, default_value_t = 1000)]
        precision_digits: u32,
        /// Significant digits to show per HP value (default 12).
        #[arg(long, default_value_t = 12)]
        display_digits: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run { lambda_sq, n_modes, top, precision_digits, display_digits, f64_only, no_force_even } => {
            if lambda_sq < 2 {
                anyhow::bail!("lambda_sq must be >= 2 (got {lambda_sq})");
            }
            let params = CcmParams::from_lambda_sq_integer(lambda_sq, n_modes);
            let primes = ccm::prime_powers_up_to(params.lambda_sq_int());
            println!(
                "CCM operator: λ²={}, N={}, matrix_size={}",
                lambda_sq, params.n_modes, params.matrix_size()
            );
            println!(
                "  prime powers k ≤ {}: {} entries",
                lambda_sq, primes.len()
            );

            if f64_only {
                let _ = display_digits;
                let _ = no_force_even;
                let result = ccm::run_f64(&params)?;
                print_results_f64(&result, top)?;
            } else {
                #[cfg(feature = "hp")]
                {
                    println!("  precision: {} decimal digits", precision_digits);
                    let mut cfg = ccm::hp::HighPrecConfig::for_decimal_digits(precision_digits);
                    if no_force_even {
                        cfg.force_even = false;
                        println!("  forced-even projection: DISABLED (natural eigenvector)");
                    }
                    match std::env::var("XC_CACHE_MODE").as_deref() {
                        Ok("off") => {
                            cfg.cache_mode = xc_numerics::quadrature::CacheMode::Off;
                            println!("  cache mode: OFF (no read, no write — pure compute)");
                        }
                        Ok("local") => {
                            cfg.cache_mode = xc_numerics::quadrature::CacheMode::JsonZip;
                            println!("  cache mode: LOCAL (read/write local disk, no network)");
                        }
                        Ok("fetch") => {
                            cfg.cache_mode = xc_numerics::quadrature::CacheMode::DynamicFetch;
                            println!("  cache mode: FETCH (local disk + remote fetch, default)");
                        }
                        Ok(other) => {
                            eprintln!("  WARNING: unknown XC_CACHE_MODE '{}'; using default (fetch)", other);
                        }
                        _ => {} // default = DynamicFetch
                    }

                    // Load reference zeros at HP precision for Newton seeding.
                    let zero_strings = xc_zeta::zeros::first_n_strings(
                        Path::new(ZEROS_PATH),
                        cfg.n_eigenvalues.max(top),
                    )?;
                    let prec = cfg.precision_bits;
                    let zero_seeds: Vec<rug::Float> = zero_strings.iter()
                        .map(|s| rug::Float::with_val(prec, rug::Float::parse(s).unwrap()))
                        .collect();

                    let hp_result = if let Ok(cache_root) = std::env::var("XC_TYPED_CACHE_ROOT") {
                        use xc_cache::{
                            ArtifactCacheContext, ArtifactExecutionCacheMode, CacheLayer,
                            CachePolicy, CacheQuality, CacheResolver, CacheVisibility,
                            DirectoryArtifactProductionSink, ToolkitVersion,
                            ZipJsonFilesystemCacheStore,
                        };
                        if cache_root.trim().is_empty() {
                            anyhow::bail!("XC_TYPED_CACHE_ROOT must not be empty");
                        }
                        let resolver = CacheResolver::new(vec![CacheLayer {
                            precedence: 0,
                            store: Box::new(ZipJsonFilesystemCacheStore::new(
                                "workstation",
                                &cache_root,
                                true,
                                CacheVisibility::Local,
                            )),
                        }]);
                        let policy = CachePolicy {
                            current_toolkit_version: ToolkitVersion::parse("0.13.0")?,
                            minimum_quality: CacheQuality::Validated,
                            accepted_schema_versions: vec![1],
                            allow_deprecated: false,
                            allow_quarantined: false,
                            allowed_visibilities: vec![CacheVisibility::Local],
                        };
                        let production_sink = std::env::var("XC_PUBLICATION_QUEUE")
                            .ok()
                            .map(DirectoryArtifactProductionSink::new)
                            .transpose()?;
                        let cache = ArtifactCacheContext {
                            resolver: Some(&resolver),
                            acceptance: Some(&policy),
                            ordered_overlays: vec!["workstation".to_owned()],
                            mode: ArtifactExecutionCacheMode::PreferReuse,
                            write_on_miss: true,
                            write_visibility: CacheVisibility::Local,
                            production_sink: production_sink
                                .as_ref()
                                .map(|sink| sink as &dyn xc_cache::ArtifactProductionSink),
                        };
                        println!("  typed cache root: {}", cache_root);
                        if let Some(sink) = &production_sink {
                            println!(
                                "  publication queue: {} (local staging only)",
                                sink.root().display()
                            );
                        }
                        ccm::hp::run_via_cache(&params, &cfg, &zero_seeds, &cache)?
                    } else {
                        if std::env::var_os("XC_PUBLICATION_QUEUE").is_some() {
                            anyhow::bail!(
                                "XC_PUBLICATION_QUEUE requires XC_TYPED_CACHE_ROOT"
                            );
                        }
                        ccm::hp::run(&params, &cfg, &zero_seeds)?
                    };

                    // ε_N is displayed in HP — at λ² >= 100 it routinely
                    // underflows f64 (10^-308). All downstream display stays
                    // in HP via xc_numerics::fmt helpers.
                    println!(
                        "  built and solved in {:.3}s, smallest Weil eigenvalue ε_N = {}",
                        hp_result.elapsed_seconds,
                        xc_numerics::fmt::display_hp(&hp_result.weil_min_eigenvalue, 6)
                    );

                    // HP-native eigenvalue table.
                    let n_compare = top.min(hp_result.eigenvalues_pos.len());
                    let ref_strings = xc_zeta::zeros::first_n_strings(
                        Path::new(ZEROS_PATH), n_compare,
                    )?;
                    let cmp_prec = hp_result.precision_bits * 2;
                    // Enough sig digits to resolve e.g. 999.4 at HP-1000.
                    let column_digits = ((precision_digits as f64).log10().ceil() as usize + 2).max(5);

                    println!(
                        "\n{:>4}  {:>22}  {:>22}  {:>14}  {:>14}",
                        "k", "computed eigenvalue", "Riemann zero t_k",
                        "abs error", "matching digits"
                    );
                    println!("{}", "-".repeat(82));

                    for (k, (eig_full, ref_str)) in hp_result.eigenvalues_pos.iter()
                        .zip(ref_strings.iter()).enumerate().take(n_compare)
                    {
                        let ref_val = rug::Float::with_val(cmp_prec,
                            rug::Float::parse(ref_str).unwrap());
                        use xc_spectral::ccm::hp::EigenvalueResult;
                        match eig_full {
                            EigenvalueResult::Converged(eig)
                            | EigenvalueResult::Approximate(eig) => {
                                let is_approx = matches!(eig_full, EigenvalueResult::Approximate(_));
                                let eig_hp = rug::Float::with_val(cmp_prec, eig);
                                let mut diff = eig_hp.clone();
                                diff -= &ref_val;
                                let abs_err = diff.abs();
                                let abs_err_str = if abs_err.is_zero() {
                                    "0".to_string()
                                } else {
                                    xc_numerics::fmt::display_hp(&abs_err, column_digits)
                                };
                                let matching = if abs_err.is_zero() {
                                    format!(">={}", cmp_prec / 3)
                                } else {
                                    let m = xc_numerics::fmt::matching_digits(&eig_hp, &ref_val);
                                    xc_numerics::fmt::display_hp(&m, column_digits)
                                };
                                // Prefix "~" on the computed eigenvalue when Approximate
                                // (step limit hit — value may still be close, but not
                                // certified to HP precision).
                                let eig_display = xc_numerics::fmt::display_hp(&eig_hp, display_digits);
                                let eig_str = if is_approx {
                                    format!("~{}", eig_display)
                                } else {
                                    eig_display
                                };
                                println!(
                                    "{:>4}  {:>22}  {:>22}  {:>14}  {:>14}",
                                    k + 1,
                                    eig_str,
                                    xc_numerics::fmt::display_hp(&ref_val, display_digits),
                                    abs_err_str,
                                    matching
                                );
                            }
                            EigenvalueResult::Failed => {
                                println!(
                                    "{:>4}  {:>22}  {:>22}  {:>14}  {:>14}",
                                    k + 1, "solver failed",
                                    xc_numerics::fmt::display_hp(&ref_val, display_digits),
                                    "N/A", "N/A"
                                );
                            }
                        }
                    }
                }
                #[cfg(not(feature = "hp"))]
                {
                    let _ = precision_digits;
                    let _ = no_force_even;
                    anyhow::bail!(
                        "High-precision tier requires --features hp at build time.\n\
                         Build with: cargo build --release --features hp"
                    );
                }
            }
        }

        Command::CheckEvenness { lambda_sq, n_modes, precision_digits, display_digits } => {
            #[cfg(not(feature = "hp"))]
            {
                let _ = (lambda_sq, n_modes, precision_digits, display_digits);
                anyhow::bail!("check-evenness requires --features hp at build time");
            }
            #[cfg(feature = "hp")]
            {
                if lambda_sq < 2 {
                    anyhow::bail!("lambda_sq must be >= 2 (got {lambda_sq})");
                }
                let params = CcmParams::from_lambda_sq_integer(lambda_sq, n_modes);
                let cfg = ccm::hp::HighPrecConfig::for_decimal_digits(precision_digits);
                println!(
                    "Measuring evenness: λ²={}, N={}, precision={} digits",
                    lambda_sq, n_modes, precision_digits
                );

                let result = ccm::hp::measure_evenness(&params, &cfg)?;

                // Pure HP display — no f64 conversion.
                use xc_numerics::fmt::{display_hp, sign_of, relative_difference, Sign};
                let prec = result.natural_eigenvalue.prec();

                println!("  ‖ξ - γξ‖ / ‖ξ‖              = {}",
                    display_hp(&result.evenness_deviation, display_digits));
                println!("  natural smallest eigenvalue     = {}",
                    display_hp(&result.natural_eigenvalue, display_digits));
                println!("  forced-even smallest eigenvalue = {}",
                    display_hp(&result.forced_eigenvalue, display_digits));

                let nat_sign = sign_of(&result.natural_eigenvalue);
                let forced_sign = sign_of(&result.forced_eigenvalue);
                println!("  natural sign = {}, forced-even sign = {}",
                    nat_sign.as_str(), forced_sign.as_str());

                if nat_sign != forced_sign && nat_sign != Sign::Zero && forced_sign != Sign::Zero {
                    println!("  => natural and forced-even smallest eigenvalues have OPPOSITE SIGNS");
                }

                let one_eminus_ten = rug::Float::with_val(prec,
                    rug::Float::parse("1e-10").unwrap());
                let one_eminus_two = rug::Float::with_val(prec,
                    rug::Float::parse("1e-2").unwrap());
                let dev = &result.evenness_deviation;

                if *dev < one_eminus_ten {
                    println!("  => Eigenvector is essentially even (deviation < 1e-10)");
                } else if *dev < one_eminus_two {
                    println!("  => Eigenvector is approximately even (small deviation)");
                } else {
                    println!("  => Eigenvector is NOT even (significant deviation)");
                    if let Some(rel) = relative_difference(
                        &result.natural_eigenvalue, &result.forced_eigenvalue
                    ) {
                        println!("  |natural − forced| / |forced| = {}",
                            display_hp(&rel, display_digits));
                    } else {
                        println!("  forced-even eigenvalue is exactly zero; relative difference undefined");
                    }
                }
            }
        }
    }
    Ok(())
}

/// Print f64-tier results (used by --f64-only flag). Eigenvalues, abs error
/// and rel error are all f64 — accurate to ~15 digits, sufficient for
/// quick smoke-tests but not for publication-grade convergence claims.
fn print_results_f64(result: &CcmResult, top: usize) -> Result<()> {
    println!(
        "  built and solved in {:.3}s, smallest Weil eigenvalue ε_N = {:.6e}",
        result.elapsed_seconds, result.weil_min_eigenvalue
    );
    let zeros = xc_zeta::zeros::first_n_f64(Path::new(ZEROS_PATH), top.max(50))?;
    println!(
        "\n{:>4}  {:>20}  {:>20}  {:>14}  {:>10}",
        "k", "computed eigenvalue", "Riemann zero t_k", "abs error", "rel error"
    );
    println!("{}", "-".repeat(78));
    let n_show = top.min(result.eigenvalues_pos.len()).min(zeros.len());
    for k in 0..n_show {
        let computed = result.eigenvalues_pos[k];
        let truth = zeros[k];
        let abs_err = (computed - truth).abs();
        let rel_err = abs_err / truth.abs();
        println!(
            "{:>4}  {:>20.10}  {:>20.10}  {:>14.4e}  {:>10.4e}",
            k + 1, computed, truth, abs_err, rel_err
        );
    }
    Ok(())
}
