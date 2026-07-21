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
use clap::{Parser, Subcommand, ValueEnum};

use xc_spectral::ccm::{self, CcmParams, CcmResult};

#[derive(Parser)]
#[command(
    name = "ccm-reproduction",
    about = "CCM Zeta Spectral Triple - reproduction and convergence analysis",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum ResearchCapture {
    /// Only the roots requested for the claim and artifacts naturally produced while finding them.
    Claim,
    /// The complete finite positive root window, without separate sector analysis.
    Research,
    /// Research capture plus natural-evenness evidence and the two lowest eigenpairs per sector.
    Gap,
    /// Maximum capture, including a configurable low spectrum from both parity sectors.
    Maximum,
}

impl ResearchCapture {
    fn captures_complete_roots(self) -> bool {
        self != Self::Claim
    }

    fn sector_eigenpairs(self, maximum_count: usize) -> Option<usize> {
        match self {
            Self::Claim | Self::Research => None,
            Self::Gap => Some(2),
            Self::Maximum => Some(maximum_count),
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Run the CCM construction at given (lambda^2, N) and report eigenvalues
    /// vs Riemann zeros.
    Run {
        /// lambda^2 value. Primes p <= lambda^2 enter the Weil form (e.g. 13, 100, 1000).
        #[arg(long, default_value_t = 13_u64)]
        lambda_sq: u64,
        /// Mode cutoff N. Matrix size is 2N+1.
        #[arg(long, default_value_t = 120)]
        n_modes: usize,
        /// How many positive eigenvalues to print.
        #[arg(long, default_value_t = 20)]
        top: usize,
        /// One-based index of the first positive CCM root to discover.
        /// The default reproduces the ordinary first-K prefix; values above
        /// one target a later finite window without using reference zeros.
        #[arg(long, default_value_t = 1)]
        first_root_index: usize,
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
        /// mode only - f64 cannot reach the precisions needed for the
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
        /// Controls additional research capture without changing arithmetic or convergence rules.
        #[arg(long, value_enum, default_value_t = ResearchCapture::Claim)]
        research_capture: ResearchCapture,
        /// Number of low eigenpairs retained in each parity sector in maximum mode.
        #[arg(long, default_value_t = 8)]
        research_sector_eigenpairs: usize,
    },
    /// Measure the natural evenness of the smallest Weil eigenvector
    /// (Claim 4: symmetry breakdown at large lambda).
    CheckEvenness {
        /// lambda^2 value (e.g. 13, 100, 1000, 1200).
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
        /// Controls additional root and sector capture without changing the evenness computation.
        #[arg(long, value_enum, default_value_t = ResearchCapture::Claim)]
        research_capture: ResearchCapture,
        /// Number of low eigenpairs retained in each parity sector in maximum mode.
        #[arg(long, default_value_t = 8)]
        research_sector_eigenpairs: usize,
    },
    /// Analyze the low even and odd CCM parity sectors and report GapLog.
    /// This is separate from ordinary root reproduction because it computes
    /// and caches additional sector matrices, spectra, and gap evidence.
    SectorGap {
        /// lambda^2 value.
        #[arg(long, default_value_t = 13_u64)]
        lambda_sq: u64,
        /// Mode cutoff N. The odd sector has dimension N.
        #[arg(long, default_value_t = 120)]
        n_modes: usize,
        /// Working precision in decimal digits.
        #[arg(long, default_value_t = 200)]
        precision_digits: u32,
        /// Number of low eigenpairs retained in each parity sector.
        #[arg(long, default_value_t = 2)]
        eigenpairs: usize,
        /// Significant digits shown for HP values.
        #[arg(long, default_value_t = 16)]
        display_digits: usize,
    },
}

fn print_runtime_parallelism() {
    let rayon_workers = rayon::current_num_threads();
    let available_cpus = std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1);
    let configured = std::env::var("RAYON_NUM_THREADS").unwrap_or_else(|_| "<unset>".to_owned());
    println!(
        "  runtime parallelism: Rayon workers={}, available CPUs={}, RAYON_NUM_THREADS={}",
        rayon_workers, available_cpus, configured
    );
    if rayon_workers == 1 {
        eprintln!(
            "  WARNING: Rayon has one worker; expensive CCM computation and publication will run effectively serially"
        );
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run {
            lambda_sq,
            n_modes,
            top,
            first_root_index,
            precision_digits,
            display_digits,
            f64_only,
            no_force_even,
            research_capture,
            research_sector_eigenpairs,
        } => {
            print_runtime_parallelism();
            if lambda_sq < 2 {
                anyhow::bail!("lambda_sq must be >= 2 (got {lambda_sq})");
            }
            if top == 0 {
                anyhow::bail!("top must be positive");
            }
            if first_root_index == 0 {
                anyhow::bail!("first_root_index is one-based and must be positive");
            }
            if research_capture != ResearchCapture::Claim && f64_only {
                anyhow::bail!("research and sector artifact capture requires the HP tier");
            }
            validate_research_capture(n_modes, research_capture, research_sector_eigenpairs)?;
            let params = CcmParams::from_lambda_sq_integer(lambda_sq, n_modes);
            let primes = ccm::prime_powers_up_to(params.lambda_sq_int());
            println!(
                "CCM operator: lambda^2={}, N={}, matrix_size={}",
                lambda_sq,
                params.n_modes,
                params.matrix_size()
            );
            println!(
                "  prime powers k <= {}: {} entries",
                lambda_sq,
                primes.len()
            );

            if f64_only {
                if first_root_index != 1 {
                    anyhow::bail!("indexed root windows require the HP independent-discovery tier");
                }
                let _ = display_digits;
                let _ = no_force_even;
                let result = ccm::run_f64(&params)?;
                print_results_f64(&result, top)?;
            } else {
                #[cfg(feature = "hp")]
                {
                    if top > n_modes {
                        anyhow::bail!("requested root count cannot exceed N");
                    }
                    let requested_last_root_index =
                        first_root_index.checked_add(top - 1).ok_or_else(|| {
                            anyhow::anyhow!("requested root index range overflows usize")
                        })?;
                    if !research_capture.captures_complete_roots()
                        && requested_last_root_index > n_modes
                    {
                        anyhow::bail!("requested root window exceeds the finite CCM reach N");
                    }
                    println!("  precision: {} decimal digits", precision_digits);
                    let mut cfg = ccm::hp::HighPrecConfig::for_decimal_digits(precision_digits);
                    cfg.n_eigenvalues = top;
                    // The toolkit independently discovers pole-aware MPFR
                    // starting points and uses its production default,
                    // Halley's method, for full-precision refinement.
                    if no_force_even {
                        cfg.force_even = false;
                        println!("  forced-even projection: DISABLED (natural eigenvector)");
                    }

                    // Discover the requested finite CCM roots independently.
                    // Reference zeros are loaded only after this result is
                    // complete and are used solely for the report below.
                    let target = if research_capture.captures_complete_roots() {
                        full_finite_positive_window(&params, cfg.precision_bits)?
                    } else if first_root_index == 1 {
                        ccm::window::ZeroTarget::FirstK { count: top }
                    } else {
                        ccm::window::ZeroTarget::IndexRange {
                            first: first_root_index,
                            last: requested_last_root_index,
                        }
                    };
                    if research_capture.captures_complete_roots() {
                        println!(
                            "  root acquisition: complete independent positive finite-source window"
                        );
                    } else {
                        println!(
                            "  root acquisition: independent CCM discovery (indices {}..={})",
                            first_root_index, requested_last_root_index
                        );
                    }
                    let run_started = std::time::Instant::now();
                    println!(
                        "  research capture: {}",
                        research_capture_label(research_capture, research_sector_eigenpairs)
                    );
                    let sector_eigenpairs = research_capture
                        .sector_eigenpairs(research_sector_eigenpairs)
                        .map(|count| count.min(n_modes));
                    let (hp_result, captured_evenness, captured_sectors) =
                        if let Some(sector_eigenpairs) = sector_eigenpairs {
                            let sector_analysis = if research_capture == ResearchCapture::Maximum {
                                ccm::hp::CcmSectorAnalysisOptions::maximum(sector_eigenpairs)
                            } else {
                                ccm::hp::CcmSectorAnalysisOptions::selected(sector_eigenpairs)
                            };
                            let captured = ccm::hp::run_independent_with_research_capture(
                                &params,
                                &cfg,
                                &target,
                                ccm::hp::CcmResearchCaptureOptions {
                                    capture_evenness: true,
                                    sector_analysis: Some(sector_analysis),
                                },
                            )?;
                            (captured.primary, captured.evenness, captured.sector_gap)
                        } else {
                            (
                                ccm::hp::run_independent(&params, &cfg, &target)?,
                                None,
                                None,
                            )
                        };

                    // ε_N is displayed in HP — at λ² >= 100 it routinely
                    // underflows f64 (10^-308). All downstream display stays
                    // in HP via xc_numerics::fmt helpers.
                    println!(
                        "  built and solved in {:.3}s, smallest Weil eigenvalue epsilon_N = {}",
                        hp_result.elapsed_seconds,
                        xc_numerics::fmt::display_hp(&hp_result.weil_min_eigenvalue, 6)
                    );

                    // HP-native eigenvalue table.
                    let n_compare = top;
                    if hp_result.eigenvalues_pos.is_empty() {
                        anyhow::bail!("independent CCM discovery returned no roots");
                    }
                    let reference_first = first_root_index;
                    let reference_last = reference_first
                        .checked_add(n_compare.saturating_sub(1))
                        .ok_or_else(|| {
                        anyhow::anyhow!("requested reference-zero range overflows usize")
                    })?;
                    let all_ref_strings = xc_zeta::zeros::bundled_first_n_strings(reference_last)?;
                    let ref_strings = &all_ref_strings[reference_first - 1..reference_last];
                    let cmp_prec = hp_result.precision_bits * 2;
                    // Enough sig digits to resolve e.g. 999.4 at HP-1000.
                    let column_digits =
                        ((precision_digits as f64).log10().ceil() as usize + 2).max(5);

                    println!(
                        "\n{:>7}  {:>8}  {:>22}  {:>22}  {:>14}  {:>14}  {:>11}",
                        "zero k",
                        "CCM root",
                        "computed eigenvalue",
                        "Riemann zero t_k",
                        "abs error",
                        "matching digits",
                        "status"
                    );
                    println!("{}", "-".repeat(114));

                    // Independent discovery may contain additional finite-source
                    // roots between Riemann-zero matches. Reference values enter
                    // only here, after computation and artifact production, and
                    // are matched one-to-one in increasing algebraic-root order.
                    let mut next_root_offset = 0usize;
                    for (reference_offset, ref_str) in ref_strings.iter().enumerate() {
                        let ref_val =
                            rug::Float::with_val(cmp_prec, rug::Float::parse(ref_str).unwrap());
                        use xc_spectral::ccm::hp::EigenvalueResult;
                        let mut best: Option<(usize, &EigenvalueResult, rug::Float)> = None;
                        for (root_offset, candidate) in hp_result
                            .eigenvalues_pos
                            .iter()
                            .enumerate()
                            .skip(next_root_offset)
                        {
                            let value = match candidate {
                                EigenvalueResult::Converged(result)
                                | EigenvalueResult::Stagnated(result)
                                | EigenvalueResult::Approximate(result) => &result.value,
                                EigenvalueResult::Failed { .. } => continue,
                            };
                            let mut difference = rug::Float::with_val(cmp_prec, value);
                            difference -= &ref_val;
                            let distance = difference.abs();
                            if best
                                .as_ref()
                                .is_none_or(|(_, _, current)| distance < *current)
                            {
                                best = Some((root_offset, candidate, distance));
                            }
                        }
                        let (root_offset, eig_full, abs_err) = best.ok_or_else(|| {
                            anyhow::anyhow!(
                                "no independently discovered CCM root remains for reference zero {}",
                                reference_first + reference_offset
                            )
                        })?;
                        next_root_offset = root_offset + 1;
                        let (result, status) = match eig_full {
                            EigenvalueResult::Converged(result) => (result, "converged"),
                            EigenvalueResult::Stagnated(result) => (result, "stagnated"),
                            EigenvalueResult::Approximate(result) => (result, "approximate"),
                            EigenvalueResult::Failed { iterations, reason } => anyhow::bail!(
                                "CCM root {} failed after {} iterations: {}",
                                hp_result.first_positive_root_index + root_offset,
                                iterations,
                                reason
                            ),
                        };
                        let eig_hp = rug::Float::with_val(cmp_prec, &result.value);
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
                        let eig_str = xc_numerics::fmt::display_hp(&eig_hp, display_digits);
                        println!(
                            "{:>7}  {:>8}  {:>22}  {:>22}  {:>14}  {:>14}  {:>11}",
                            reference_first + reference_offset,
                            hp_result.first_positive_root_index + root_offset,
                            eig_str,
                            xc_numerics::fmt::display_hp(&ref_val, display_digits),
                            abs_err_str,
                            matching,
                            status
                        );
                    }

                    if let (Some(evenness), Some(sectors), Some(sector_eigenpairs)) =
                        (captured_evenness, captured_sectors, sector_eigenpairs)
                    {
                        println!("\n=== Supplemental research artifact capture ===");
                        println!(
                            "  natural-evenness evidence captured from validated parity sectors: deviation={}",
                            xc_numerics::fmt::display_hp(
                                &evenness.evenness_deviation,
                                display_digits
                            )
                        );
                        println!(
                            "  even/odd sector spectra and GapLog captured: {} eigenpairs per sector, complete spectra={}, GapLog={}",
                            sector_eigenpairs,
                            if research_capture == ResearchCapture::Maximum {
                                "yes"
                            } else {
                                "no"
                            },
                            xc_numerics::fmt::display_hp(&sectors.gap_log, display_digits)
                        );
                    }
                    println!(
                        "\n  total claim and research capture time: {:.3}s",
                        run_started.elapsed().as_secs_f64()
                    );
                }
                #[cfg(not(feature = "hp"))]
                {
                    let _ = precision_digits;
                    let _ = no_force_even;
                    let _ = (research_capture, research_sector_eigenpairs);
                    anyhow::bail!(
                        "High-precision tier requires --features hp at build time.\n\
                         Build with: cargo build --release --features hp"
                    );
                }
            }
        }

        Command::CheckEvenness {
            lambda_sq,
            n_modes,
            precision_digits,
            display_digits,
            research_capture,
            research_sector_eigenpairs,
        } => {
            #[cfg(not(feature = "hp"))]
            {
                let _ = (
                    lambda_sq,
                    n_modes,
                    precision_digits,
                    display_digits,
                    research_capture,
                    research_sector_eigenpairs,
                );
                anyhow::bail!("check-evenness requires --features hp at build time");
            }
            #[cfg(feature = "hp")]
            {
                if lambda_sq < 2 {
                    anyhow::bail!("lambda_sq must be >= 2 (got {lambda_sq})");
                }
                validate_research_capture(n_modes, research_capture, research_sector_eigenpairs)?;
                let params = CcmParams::from_lambda_sq_integer(lambda_sq, n_modes);
                let cfg = ccm::hp::HighPrecConfig::for_decimal_digits(precision_digits);
                println!(
                    "Measuring evenness: lambda^2={}, N={}, precision={} digits",
                    lambda_sq, n_modes, precision_digits
                );

                let result = ccm::hp::measure_evenness(&params, &cfg)?;

                // Pure HP display — no f64 conversion.
                use xc_numerics::fmt::{display_hp, relative_difference, sign_of, Sign};
                let prec = result.natural_eigenvalue.prec();

                println!(
                    "  ||xi - gamma(xi)|| / ||xi|| = {}",
                    display_hp(&result.evenness_deviation, display_digits)
                );
                println!(
                    "  natural smallest eigenvalue     = {}",
                    display_hp(&result.natural_eigenvalue, display_digits)
                );
                println!(
                    "  forced-even smallest eigenvalue = {}",
                    display_hp(&result.forced_eigenvalue, display_digits)
                );

                let nat_sign = sign_of(&result.natural_eigenvalue);
                let forced_sign = sign_of(&result.forced_eigenvalue);
                println!(
                    "  natural sign = {}, forced-even sign = {}",
                    nat_sign.as_str(),
                    forced_sign.as_str()
                );

                if nat_sign != forced_sign && nat_sign != Sign::Zero && forced_sign != Sign::Zero {
                    println!(
                        "  => natural and forced-even smallest eigenvalues have OPPOSITE SIGNS"
                    );
                }

                let one_eminus_ten =
                    rug::Float::with_val(prec, rug::Float::parse("1e-10").unwrap());
                let one_eminus_two = rug::Float::with_val(prec, rug::Float::parse("1e-2").unwrap());
                let dev = &result.evenness_deviation;

                if *dev < one_eminus_ten {
                    println!("  => Eigenvector is essentially even (deviation < 1e-10)");
                } else if *dev < one_eminus_two {
                    println!("  => Eigenvector is approximately even (small deviation)");
                } else {
                    println!("  => Eigenvector is NOT even (significant deviation)");
                    if let Some(rel) =
                        relative_difference(&result.natural_eigenvalue, &result.forced_eigenvalue)
                    {
                        println!(
                            "  |natural - forced| / |forced| = {}",
                            display_hp(&rel, display_digits)
                        );
                    } else {
                        println!("  forced-even eigenvalue is exactly zero; relative difference undefined");
                    }
                }

                if research_capture != ResearchCapture::Claim {
                    capture_supplemental_research_artifacts(
                        &params,
                        &cfg,
                        true,
                        false,
                        research_capture
                            .sector_eigenpairs(research_sector_eigenpairs)
                            .map(|count| count.min(n_modes)),
                        research_capture == ResearchCapture::Maximum,
                        display_digits,
                    )?;
                }
            }
        }

        Command::SectorGap {
            lambda_sq,
            n_modes,
            precision_digits,
            eigenpairs,
            display_digits,
        } => {
            #[cfg(not(feature = "hp"))]
            {
                let _ = (
                    lambda_sq,
                    n_modes,
                    precision_digits,
                    eigenpairs,
                    display_digits,
                );
                anyhow::bail!("sector-gap requires --features hp at build time");
            }
            #[cfg(feature = "hp")]
            {
                if lambda_sq < 2 {
                    anyhow::bail!("lambda_sq must be >= 2 (got {lambda_sq})");
                }
                if eigenpairs < 2 || eigenpairs > n_modes {
                    anyhow::bail!("eigenpairs must be between 2 and N");
                }
                let params = CcmParams::from_lambda_sq_integer(lambda_sq, n_modes);
                let mut cfg = ccm::hp::HighPrecConfig::for_decimal_digits(precision_digits);
                cfg.n_eigenvalues = 0;
                println!(
                    "CCM sector analysis: lambda^2={}, N={}, precision={} digits, retained={} per sector",
                    lambda_sq, n_modes, precision_digits, eigenpairs
                );
                let result = ccm::hp::analyze_sector_gap(&params, &cfg, eigenpairs)?;
                let show = |value: &rug::Float| xc_numerics::fmt::display_hp(value, display_digits);
                println!("  lambda_even             = {}", show(&result.lambda_even));
                println!("  lambda_odd              = {}", show(&result.lambda_odd));
                println!("  D_even                  = {}", show(&result.d_even));
                println!("  D_odd                   = {}", show(&result.d_odd));
                println!("  GapLog (D_even-D_odd)   = {}", show(&result.gap_log));
                println!(
                    "  lambda_odd-lambda_even  = {}",
                    show(&result.lambda_difference)
                );
                println!(
                    "  difference depth        = {}",
                    show(&result.difference_depth)
                );
                println!("  ordering                = {}", result.ordering);
                println!("  even ground state simple = {}", result.even_simple);
                println!(
                    "  even simplicity margin  = {}",
                    show(&result.even_simplicity_margin)
                );
                for sector in [&result.even, &result.odd] {
                    println!(
                        "\n  {} sector (dimension {}):",
                        sector.parity.as_str(),
                        sector.dimension
                    );
                    for pair in &sector.eigenpairs {
                        println!(
                            "    algebraic index {:>3}: eigenvalue={}, residual={}",
                            pair.algebraic_index,
                            show(&pair.eigenvalue),
                            show(&pair.residual_norm)
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_research_capture(
    n_modes: usize,
    capture: ResearchCapture,
    sector_eigenpairs: usize,
) -> Result<()> {
    let Some(sector_eigenpairs) = capture.sector_eigenpairs(sector_eigenpairs) else {
        return Ok(());
    };
    if n_modes < 2 {
        anyhow::bail!("research artifact capture requires N >= 2");
    }
    if sector_eigenpairs < 2 {
        anyhow::bail!("research_sector_eigenpairs must be at least 2");
    }
    Ok(())
}

fn research_capture_label(capture: ResearchCapture, maximum_count: usize) -> String {
    match capture {
        ResearchCapture::Claim => "claim (requested roots and native artifacts)".to_string(),
        ResearchCapture::Research => {
            "research (complete finite root window; no sector solve)".to_string()
        }
        ResearchCapture::Gap => {
            "gap (complete roots, evenness, GapLog, 2 eigenpairs per sector)".to_string()
        }
        ResearchCapture::Maximum => format!(
            "maximum (complete roots, evenness, GapLog, {maximum_count} eigenpairs per sector)"
        ),
    }
}

#[cfg(any(feature = "hp", test))]
fn full_finite_positive_window(
    params: &CcmParams,
    precision_bits: u32,
) -> Result<ccm::window::ZeroTarget> {
    use rug::{ops::Pow, Float};

    let lambda_squared = if params.lambda_sq.is_integer {
        Float::with_val(precision_bits, params.lambda_sq.value_u64)
    } else {
        Float::with_val(
            precision_bits,
            Float::parse(format!("{:.17}", params.lambda_sq.value_f64))?,
        )
    };
    let log_lambda_squared = lambda_squared.ln();
    if !log_lambda_squared.is_finite() || log_lambda_squared <= 0 {
        anyhow::bail!("lambda_squared does not define a finite positive CCM window");
    }
    // Stay strictly inside the terminal secular pole. The toolkit performs
    // the authoritative independent scan and determines the actual root count.
    let mut upper = Float::with_val(precision_bits, rug::float::Constant::Pi);
    upper *= 2u32;
    upper *= params.n_modes;
    upper /= log_lambda_squared;
    let mut interior_scale = Float::with_val(precision_bits, 1);
    interior_scale -= Float::with_val(precision_bits, 2).pow(-64i32);
    upper *= interior_scale;
    let lower = Float::with_val(precision_bits, 2).pow(-(precision_bits as i32));
    Ok(ccm::window::ZeroTarget::HeightWindow {
        lower: lower.to_string(),
        upper: upper.to_string(),
    })
}

/// Fill the artifact families not already produced by the command that called
/// this helper. Each toolkit API owns its ordinary managed-cache lifecycle, so
/// author publication settings still apply without publication logic in this
/// consumer repository.
#[cfg(feature = "hp")]
fn capture_supplemental_research_artifacts(
    params: &CcmParams,
    cfg: &ccm::hp::HighPrecConfig,
    capture_roots: bool,
    capture_evenness: bool,
    sector_eigenpairs: Option<usize>,
    complete_sector_spectrum: bool,
    display_digits: usize,
) -> Result<()> {
    let supplemental_started = std::time::Instant::now();
    println!("\n=== Supplemental research artifact capture ===");

    if capture_roots {
        let roots_started = std::time::Instant::now();
        let mut root_cfg = cfg.clone();
        root_cfg.n_eigenvalues = 0;
        let roots = ccm::hp::run_independent(
            params,
            &root_cfg,
            &full_finite_positive_window(params, root_cfg.precision_bits)?,
        )?;
        let counts = roots.eigenvalues_pos.iter().fold(
            (0usize, 0usize, 0usize, 0usize),
            |mut counts, root| {
                match root {
                    ccm::hp::EigenvalueResult::Converged(_) => counts.0 += 1,
                    ccm::hp::EigenvalueResult::Stagnated(_) => counts.1 += 1,
                    ccm::hp::EigenvalueResult::Approximate(_) => counts.2 += 1,
                    ccm::hp::EigenvalueResult::Failed { .. } => counts.3 += 1,
                }
                counts
            },
        );
        println!(
            "  complete positive finite-source root window captured: {} converged, {} stagnated, {} approximate, {} failed; elapsed={:.3}s",
            counts.0,
            counts.1,
            counts.2,
            counts.3,
            roots_started.elapsed().as_secs_f64()
        );
    } else {
        println!("  complete positive finite-source root window captured by the primary run");
    }

    if capture_evenness {
        let evenness_started = std::time::Instant::now();
        let evenness = ccm::hp::measure_evenness(params, cfg)?;
        println!(
            "  natural-evenness evidence captured: deviation={}, elapsed={:.3}s",
            xc_numerics::fmt::display_hp(&evenness.evenness_deviation, display_digits),
            evenness_started.elapsed().as_secs_f64()
        );
    } else {
        println!("  natural-evenness evidence captured by the primary run");
    }

    if let Some(sector_eigenpairs) = sector_eigenpairs {
        let sector_started = std::time::Instant::now();
        let mut sector_cfg = cfg.clone();
        sector_cfg.n_eigenvalues = 0;
        sector_cfg.force_even = true;
        let sector_options = if complete_sector_spectrum {
            ccm::hp::CcmSectorAnalysisOptions::maximum(sector_eigenpairs)
        } else {
            ccm::hp::CcmSectorAnalysisOptions::selected(sector_eigenpairs)
        };
        let sectors =
            ccm::hp::analyze_sector_gap_with_options(params, &sector_cfg, sector_options)?;
        println!(
            "  even/odd sector spectra and GapLog captured: {} eigenpairs per sector, complete spectra={}, GapLog={}, elapsed={:.3}s",
            sector_eigenpairs,
            if complete_sector_spectrum { "yes" } else { "no" },
            xc_numerics::fmt::display_hp(&sectors.gap_log, display_digits),
            sector_started.elapsed().as_secs_f64()
        );
    }
    println!(
        "  supplemental artifact capture complete: elapsed={:.3}s",
        supplemental_started.elapsed().as_secs_f64()
    );
    Ok(())
}

/// Print f64-tier results (used by --f64-only flag). Eigenvalues, abs error
/// and rel error are all f64 — accurate to ~15 digits, sufficient for
/// quick smoke-tests but not for publication-grade convergence claims.
fn print_results_f64(result: &CcmResult, top: usize) -> Result<()> {
    println!(
        "  built and solved in {:.3}s, smallest Weil eigenvalue epsilon_N = {:.6e}",
        result.elapsed_seconds, result.weil_min_eigenvalue
    );
    let zero_strings = xc_zeta::zeros::bundled_first_n_strings(top.max(50))?;
    let zeros = zero_strings
        .iter()
        .map(|zero| {
            zero.parse::<f64>().map_err(|error| {
                anyhow::anyhow!("failed to parse bundled reference zero {zero:?}: {error}")
            })
        })
        .collect::<Result<Vec<_>>>()?;
    println!(
        "\n{:>4}  {:>20}  {:>20}  {:>14}  {:>10}",
        "k", "computed eigenvalue", "Riemann zero t_k", "abs error", "rel error"
    );
    println!("{}", "-".repeat(78));
    let n_show = top.min(result.eigenvalues_pos.len()).min(zeros.len());
    for (k, (&computed, &truth)) in result
        .eigenvalues_pos
        .iter()
        .zip(&zeros)
        .take(n_show)
        .enumerate()
    {
        let abs_err = (computed - truth).abs();
        let rel_err = abs_err / truth.abs();
        println!(
            "{:>4}  {:>20.10}  {:>20.10}  {:>14.4e}  {:>10.4e}",
            k + 1,
            computed,
            truth,
            abs_err,
            rel_err
        );
    }
    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn version_flag_reports_the_package_version() {
        let error = match Cli::try_parse_from(["ccm-reproduction", "--version"]) {
            Ok(_) => panic!("--version must terminate with a display-version result"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayVersion);
        assert!(error.to_string().contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn research_capture_flags_parse_for_primary_commands() {
        assert!(Cli::try_parse_from([
            "ccm-reproduction",
            "run",
            "--research-capture",
            "maximum",
            "--research-sector-eigenpairs",
            "10",
        ])
        .is_ok());
        assert!(Cli::try_parse_from([
            "ccm-reproduction",
            "check-evenness",
            "--research-capture",
            "gap",
            "--research-sector-eigenpairs",
            "10",
        ])
        .is_ok());
    }

    #[test]
    fn research_capture_bounds_are_guarded() {
        assert!(validate_research_capture(120, ResearchCapture::Maximum, 8).is_ok());
        assert!(validate_research_capture(120, ResearchCapture::Maximum, 1).is_err());
        assert!(validate_research_capture(5, ResearchCapture::Gap, 8).is_ok());
        assert!(validate_research_capture(1, ResearchCapture::Gap, 2).is_err());
        assert!(validate_research_capture(5, ResearchCapture::Research, 1).is_ok());
    }

    #[test]
    fn finite_window_stays_inside_terminal_pole() {
        let params = CcmParams::from_lambda_sq_integer(13, 10);
        let target = full_finite_positive_window(&params, 256).unwrap();
        let ccm::window::ZeroTarget::HeightWindow { lower, upper } = target else {
            panic!("research capture must use a finite height window");
        };
        let precision_bits = 256;
        let lower = rug::Float::with_val(precision_bits, rug::Float::parse(lower).unwrap());
        let upper = rug::Float::with_val(precision_bits, rug::Float::parse(upper).unwrap());
        let mut terminal_pole = rug::Float::with_val(precision_bits, rug::float::Constant::Pi);
        terminal_pole *= 20u32;
        terminal_pole /= rug::Float::with_val(precision_bits, 13).ln();
        assert!(lower > 0);
        assert!(upper < terminal_pole);
    }
}
