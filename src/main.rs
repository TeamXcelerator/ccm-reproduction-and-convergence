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
    /// Requested roots plus their detailed research artifacts, without sector analysis.
    Research,
    /// Research capture plus natural-evenness evidence and the two lowest eigenpairs per sector.
    Gap,
    /// Maximum capture, including a configurable low spectrum from both parity sectors.
    Maximum,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum RootValidation {
    /// Do not construct a separate interval root certificate.
    Off,
    /// Certify the displayed ordinal range of the exact retained finite CCM point source.
    Certified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum RootAcquisitionMode {
    /// Refine roots from the toolkit-owned, content-bound reference-zero table.
    Seeded,
    /// Discover starting points from the finite CCM secular source itself.
    Independent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum RootReport {
    /// Compare the requested reference-zero ordinals with their CCM roots.
    Reference,
    /// Preserve independently discovered CCM root order and classify each root.
    DiscoveryOrdering,
}

impl ResearchCapture {
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
        /// How many CCM roots to request and print.
        #[arg(long, default_value_t = 20)]
        top: usize,
        /// One-based index of the first positive CCM root to acquire.
        /// The default reproduces the ordinary first-K prefix; values above
        /// one target a later ordinal window under the selected acquisition
        /// policy. Advanced signed discovery requires this to remain one.
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
        /// Root acquisition policy. Paper claims default to reference-seeded refinement.
        #[arg(long, value_enum, default_value_t = RootAcquisitionMode::Seeded)]
        root_acquisition: RootAcquisitionMode,
        /// Root-table layout. Discovery ordering preserves every independently
        /// discovered root, including unmatched finite-source roots.
        #[arg(long, value_enum, default_value_t = RootReport::Reference)]
        root_report: RootReport,
        /// Minimum decimal digits required to classify a discovered root as a
        /// reference-zero match in a discovery-ordering report.
        #[arg(long, default_value_t = 10)]
        minimum_match_digits: u32,
        /// Number of bundled reference zeros considered by discovery-ordering
        /// sequence alignment.
        #[arg(long, default_value_t = 400)]
        reference_zero_limit: usize,
        /// Advanced independent-discovery mode: scan the complete signed
        /// finite source window [-Tmax, Tmax] instead of positive roots only.
        #[arg(long, default_value_t = false)]
        include_negative_roots: bool,
        /// Advanced independent-discovery mode: permit top > N and return all
        /// roots actually found when the finite source cannot fill the target.
        #[arg(long, default_value_t = false)]
        allow_root_oversubscription: bool,
        /// Optional root-only certification. This does not interval-certify Tau or the eigenstate.
        #[arg(long, value_enum, default_value_t = RootValidation::Off)]
        root_validation: RootValidation,
        /// Optional decimal-width target for each certified root enclosure.
        /// Defaults to display_digits and is independent of HP working precision.
        #[arg(long)]
        root_enclosure_digits: Option<u32>,
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
        /// Root acquisition policy for supplemental research capture.
        #[arg(long, value_enum, default_value_t = RootAcquisitionMode::Seeded)]
        root_acquisition: RootAcquisitionMode,
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
            root_acquisition,
            root_report,
            minimum_match_digits,
            reference_zero_limit,
            include_negative_roots,
            allow_root_oversubscription,
            root_validation,
            root_enclosure_digits,
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
            if root_validation != RootValidation::Off && f64_only {
                anyhow::bail!("root certification requires the HP tier");
            }
            if root_acquisition == RootAcquisitionMode::Seeded && f64_only {
                anyhow::bail!(
                    "reference-seeded root acquisition requires the HP tier; use --root-acquisition independent with --f64-only"
                );
            }
            if root_report == RootReport::DiscoveryOrdering {
                if root_acquisition != RootAcquisitionMode::Independent {
                    anyhow::bail!(
                        "discovery-ordering reports require --root-acquisition independent"
                    );
                }
                if f64_only {
                    anyhow::bail!("discovery-ordering reports require the HP tier");
                }
                if first_root_index != 1 {
                    anyhow::bail!(
                        "discovery-ordering reports currently require --first-root-index 1"
                    );
                }
                if minimum_match_digits == 0 {
                    anyhow::bail!("minimum_match_digits must be positive");
                }
                if reference_zero_limit == 0 {
                    anyhow::bail!("reference_zero_limit must be positive");
                }
            }
            if include_negative_roots || allow_root_oversubscription {
                if root_acquisition != RootAcquisitionMode::Independent {
                    anyhow::bail!(
                        "advanced signed or oversubscribed roots require --root-acquisition independent"
                    );
                }
                if f64_only {
                    anyhow::bail!("advanced signed or oversubscribed roots require the HP tier");
                }
                if first_root_index != 1 {
                    anyhow::bail!(
                        "advanced signed or oversubscribed discovery currently requires --first-root-index 1"
                    );
                }
                if root_validation != RootValidation::Off {
                    anyhow::bail!(
                        "advanced signed or incomplete root windows cannot be root-certified"
                    );
                }
            }
            if include_negative_roots && root_report != RootReport::DiscoveryOrdering {
                anyhow::bail!("--include-negative-roots requires --root-report discovery-ordering");
            }
            if root_validation == RootValidation::Off && root_enclosure_digits.is_some() {
                anyhow::bail!("--root-enclosure-digits requires --root-validation certified");
            }
            let root_enclosure_digits = root_enclosure_digits
                .map_or_else(|| u32::try_from(display_digits), Ok)
                .map_err(|_| anyhow::anyhow!("display_digits exceeds the supported u32 range"))?;
            if root_validation == RootValidation::Certified
                && (root_enclosure_digits == 0 || root_enclosure_digits > precision_digits)
            {
                anyhow::bail!("root enclosure digits must be between 1 and precision_digits");
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
                    if top > n_modes && !allow_root_oversubscription {
                        anyhow::bail!("requested root count cannot exceed N");
                    }
                    let (target, requested_last_root_index) = explicit_ordinal_root_target(
                        first_root_index,
                        top,
                        n_modes,
                        allow_root_oversubscription,
                    )?;
                    let discovery_options = ccm::hp::IndependentRootDiscoveryOptions::advanced(
                        include_negative_roots,
                        allow_root_oversubscription,
                    );
                    println!("  precision: {} decimal digits", precision_digits);
                    let mut cfg = ccm::hp::HighPrecConfig::for_decimal_digits(precision_digits);
                    cfg.n_eigenvalues = top;
                    // Both acquisition modes use the same production HP
                    // Halley refinement and convergence policy.
                    if no_force_even {
                        cfg.force_even = false;
                        println!("  forced-even projection: DISABLED (natural eigenvector)");
                    }

                    // Capture level never changes root acquisition semantics or
                    // expands an ordinal claim into a height window. A run is
                    // wholly seeded or wholly independent for this exact range.
                    let seeded_input = if root_acquisition == RootAcquisitionMode::Seeded {
                        let (dataset, seed_first_index, seed_strings) =
                            bundled_reference_seed_window(&target)?;
                        let seeds = seed_strings
                            .iter()
                            .map(|seed| {
                                rug::Float::parse(seed)
                                    .map(|parsed| rug::Float::with_val(cfg.precision_bits, parsed))
                                    .map_err(|error| {
                                        anyhow::anyhow!(
                                            "failed to parse bundled reference seed {seed:?}: {error}"
                                        )
                                    })
                            })
                            .collect::<Result<Vec<_>>>()?;
                        println!(
                            "  root acquisition: reference-seeded refinement (indices {}..={}, dataset={}, sha256={})",
                            seed_first_index,
                            seed_first_index + seeds.len() - 1,
                            dataset.resource_id,
                            &dataset.content_sha256[..12]
                        );
                        Some((dataset, seed_first_index, seeds))
                    } else {
                        println!(
                            "  root acquisition: independent CCM discovery (indices {}..={})",
                            first_root_index, requested_last_root_index
                        );
                        if include_negative_roots {
                            println!(
                                "  root domain: signed finite window [-Tmax, Tmax] (advanced)"
                            );
                        }
                        if allow_root_oversubscription {
                            println!(
                                "  root count policy: return available finite roots on shortfall (advanced)"
                            );
                        }
                        None
                    };
                    let run_started = std::time::Instant::now();
                    println!(
                        "  research capture: {}",
                        research_capture_label(research_capture, research_sector_eigenpairs)
                    );
                    let sector_eigenpairs = research_capture
                        .sector_eigenpairs(research_sector_eigenpairs)
                        .map(|count| count.min(n_modes));
                    let sector_analysis = sector_eigenpairs.map(|sector_eigenpairs| {
                        if research_capture == ResearchCapture::Maximum {
                            ccm::hp::CcmSectorAnalysisOptions::maximum(sector_eigenpairs)
                        } else {
                            ccm::hp::CcmSectorAnalysisOptions::selected(sector_eigenpairs)
                        }
                    });
                    let root_certification = match root_validation {
                        RootValidation::Off => None,
                        RootValidation::Certified => {
                            #[cfg(not(feature = "root-certification"))]
                            anyhow::bail!(
                                "--root-validation certified requires building with --features hp,root-certification"
                            );
                            #[cfg(feature = "root-certification")]
                            {
                                let certification_target = if first_root_index == 1 {
                                    ccm::certified_roots::IndependentCcmRootTarget::Prefix {
                                        count: top,
                                    }
                                } else {
                                    ccm::certified_roots::IndependentCcmRootTarget::IndexRange {
                                        first: first_root_index,
                                        last: requested_last_root_index,
                                    }
                                };
                                println!(
                                    "  root validation: certified exact finite-source ordinals {}..={} at {} digits (separate certificate artifact)",
                                    first_root_index,
                                    requested_last_root_index,
                                    root_enclosure_digits
                                );
                                Some(ccm::hp::CcmRootCertificationOptions::for_decimal_digits(
                                    certification_target,
                                    root_enclosure_digits,
                                )?)
                            }
                        }
                    };
                    let capture_requested =
                        sector_analysis.is_some() || root_certification.is_some();
                    let capture_options = ccm::hp::CcmResearchCaptureOptions {
                        capture_evenness: sector_analysis.is_some(),
                        sector_analysis,
                        root_certification,
                    };
                    let (hp_result, captured_evenness, captured_sectors, root_certificate) =
                        match root_acquisition {
                            RootAcquisitionMode::Independent if capture_requested => {
                                let captured =
                                    ccm::hp::run_independent_with_options_and_research_capture(
                                        &params,
                                        &cfg,
                                        &target,
                                        discovery_options,
                                        capture_options,
                                    )?;
                                (
                                    captured.primary,
                                    captured.evenness,
                                    captured.sector_gap,
                                    captured.root_certificate,
                                )
                            }
                            RootAcquisitionMode::Independent => (
                                ccm::hp::run_independent_with_options(
                                    &params,
                                    &cfg,
                                    &target,
                                    discovery_options,
                                )?,
                                None,
                                None,
                                None,
                            ),
                            RootAcquisitionMode::Seeded if capture_requested => {
                                let (dataset, seed_first_index, seeds) = seeded_input
                                    .as_ref()
                                    .expect("seeded acquisition prepared reference inputs");
                                let captured = ccm::hp::run_indexed_seeded_with_research_capture(
                                    &params,
                                    &cfg,
                                    *seed_first_index,
                                    seeds,
                                    dataset,
                                    capture_options,
                                )?;
                                (
                                    captured.primary,
                                    captured.evenness,
                                    captured.sector_gap,
                                    captured.root_certificate,
                                )
                            }
                            RootAcquisitionMode::Seeded => {
                                let (dataset, seed_first_index, seeds) = seeded_input
                                    .as_ref()
                                    .expect("seeded acquisition prepared reference inputs");
                                (
                                    ccm::hp::run_indexed_seeded(
                                        &params,
                                        &cfg,
                                        *seed_first_index,
                                        seeds,
                                        dataset,
                                    )?,
                                    None,
                                    None,
                                    None,
                                )
                            }
                        };

                    if let Some(certificate) = &root_certificate {
                        println!(
                            "  certified root census: {} roots, indices {}..={}, scope=exact stored point source",
                            certificate.selected_root_count,
                            certificate.first_selected_positive_index.unwrap_or(0),
                            certificate.last_selected_positive_index.unwrap_or(0)
                        );
                    }

                    // ε_N is displayed in HP — at λ² >= 100 it routinely
                    // underflows f64 (10^-308). All downstream display stays
                    // in HP via xc_numerics::fmt helpers.
                    println!(
                        "  built and solved in {:.3}s, smallest Weil eigenvalue epsilon_N = {}",
                        hp_result.elapsed_seconds,
                        xc_numerics::fmt::display_hp(&hp_result.weil_min_eigenvalue, 6)
                    );

                    if hp_result.eigenvalues_pos.is_empty() && !allow_root_oversubscription {
                        anyhow::bail!("CCM root acquisition returned no roots");
                    }
                    if include_negative_roots || allow_root_oversubscription {
                        println!(
                            "  advanced discovery result: requested {}, returned {}, domain={}",
                            top,
                            hp_result.eigenvalues_pos.len(),
                            if include_negative_roots {
                                "signed"
                            } else {
                                "positive"
                            }
                        );
                    }
                    if root_report == RootReport::DiscoveryOrdering {
                        print_discovery_ordering_report(
                            &hp_result,
                            &DiscoveryOrderingReportOptions {
                                requested_roots: top,
                                reference_zero_limit,
                                minimum_match_digits,
                                precision_digits,
                                display_digits,
                                allow_incomplete: allow_root_oversubscription,
                                signed_domain: include_negative_roots,
                            },
                        )?;
                    } else {
                        // HP-native reference-ordinal eigenvalue table.
                        let n_compare = top;
                        let reference_first = first_root_index;
                        let reference_last = reference_first
                            .checked_add(n_compare.saturating_sub(1))
                            .ok_or_else(|| {
                                anyhow::anyhow!("requested reference-zero range overflows usize")
                            })?;
                        let all_ref_strings =
                            xc_zeta::zeros::bundled_first_n_strings(reference_last)?;
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

                        // Seeded refinement preserves its explicitly assigned
                        // ordinals. Independent discovery may contain additional
                        // finite-source roots and therefore uses monotone nearest
                        // matching only for this post-computation comparison table.
                        let mut next_root_offset = 0usize;
                        for (reference_offset, ref_str) in ref_strings.iter().enumerate() {
                            let ref_val =
                                rug::Float::with_val(cmp_prec, rug::Float::parse(ref_str).unwrap());
                            use xc_spectral::ccm::hp::EigenvalueResult;
                            let reference_index = reference_first + reference_offset;
                            let (root_offset, eig_full, abs_err) = match root_acquisition {
                                RootAcquisitionMode::Seeded => {
                                    let root_offset = reference_index
                                    .checked_sub(hp_result.first_positive_root_index)
                                    .ok_or_else(|| {
                                        anyhow::anyhow!(
                                            "seeded CCM root window begins after reference ordinal {reference_index}"
                                        )
                                    })?;
                                    let candidate = hp_result
                                    .eigenvalues_pos
                                    .get(root_offset)
                                    .ok_or_else(|| {
                                        anyhow::anyhow!(
                                            "seeded CCM root window does not contain reference ordinal {reference_index}"
                                        )
                                    })?;
                                    let value = candidate.value().ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "seeded CCM root ordinal {reference_index} failed before comparison"
                                    )
                                })?;
                                    let mut difference = rug::Float::with_val(cmp_prec, value);
                                    difference -= &ref_val;
                                    (root_offset, candidate, difference.abs())
                                }
                                RootAcquisitionMode::Independent => {
                                    let mut best: Option<(usize, &EigenvalueResult, rug::Float)> =
                                        None;
                                    for (root_offset, candidate) in hp_result
                                        .eigenvalues_pos
                                        .iter()
                                        .enumerate()
                                        .skip(next_root_offset)
                                    {
                                        let Some(value) = candidate.value() else {
                                            continue;
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
                                    let best = best.ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "no independently discovered CCM root remains for reference zero {reference_index}"
                                    )
                                })?;
                                    next_root_offset = best.0 + 1;
                                    best
                                }
                            };
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
                    let _ = (
                        research_capture,
                        research_sector_eigenpairs,
                        root_acquisition,
                        root_report,
                        minimum_match_digits,
                        reference_zero_limit,
                        root_validation,
                        root_enclosure_digits,
                    );
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
            root_acquisition,
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
                    root_acquisition,
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
                        SupplementalResearchCaptureOptions {
                            capture_roots: true,
                            capture_evenness: false,
                            sector_eigenpairs: research_capture
                                .sector_eigenpairs(research_sector_eigenpairs)
                                .map(|count| count.min(n_modes)),
                            complete_sector_spectrum: research_capture == ResearchCapture::Maximum,
                            display_digits,
                            root_acquisition,
                        },
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct AlignmentScore {
    matches: usize,
    quality: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AlignmentStep {
    Start,
    SkipRoot,
    SkipReference,
    Match,
}

fn score_is_better(candidate: AlignmentScore, current: AlignmentScore) -> bool {
    candidate.matches > current.matches
        || (candidate.matches == current.matches && candidate.quality > current.quality)
}

/// Find a monotone one-to-one alignment between discovered roots and reference
/// zeros. A candidate edge exists only when it meets the requested digit
/// threshold. The primary objective is the number of identified zeros; total
/// matching digits resolve otherwise equivalent alignments.
fn align_discovered_roots(
    matching_digits: &[Vec<Option<f64>>],
    minimum_match_digits: f64,
) -> Vec<Option<usize>> {
    let root_count = matching_digits.len();
    let reference_count = matching_digits.first().map_or(0, Vec::len);
    assert!(
        matching_digits
            .iter()
            .all(|row| row.len() == reference_count),
        "matching matrix must be rectangular"
    );

    let width = reference_count + 1;
    let mut scores = vec![AlignmentScore::default(); (root_count + 1) * width];
    let mut steps = vec![AlignmentStep::Start; (root_count + 1) * width];

    for root in 1..=root_count {
        steps[root * width] = AlignmentStep::SkipRoot;
    }
    for step in steps.iter_mut().take(width).skip(1) {
        *step = AlignmentStep::SkipReference;
    }

    for (root_offset, digit_row) in matching_digits.iter().enumerate() {
        let root = root_offset + 1;
        for (reference_offset, digits) in digit_row.iter().enumerate() {
            let reference = reference_offset + 1;
            let index = root * width + reference;
            let skip_root = scores[(root - 1) * width + reference];
            let skip_reference = scores[root * width + reference - 1];
            let mut best = skip_root;
            let mut step = AlignmentStep::SkipRoot;
            if score_is_better(skip_reference, best) {
                best = skip_reference;
                step = AlignmentStep::SkipReference;
            }

            if let Some(digits) = digits {
                if digits.is_finite() && *digits >= minimum_match_digits {
                    let previous = scores[(root - 1) * width + reference - 1];
                    let candidate = AlignmentScore {
                        matches: previous.matches + 1,
                        quality: previous.quality + *digits,
                    };
                    if score_is_better(candidate, best) {
                        best = candidate;
                        step = AlignmentStep::Match;
                    }
                }
            }
            scores[index] = best;
            steps[index] = step;
        }
    }

    let mut assignments = vec![None; root_count];
    let mut root = root_count;
    let mut reference = reference_count;
    while root > 0 || reference > 0 {
        match steps[root * width + reference] {
            AlignmentStep::Match => {
                assignments[root - 1] = Some(reference - 1);
                root -= 1;
                reference -= 1;
            }
            AlignmentStep::SkipRoot => root -= 1,
            AlignmentStep::SkipReference => reference -= 1,
            AlignmentStep::Start => break,
        }
    }
    assignments
}

#[cfg(feature = "hp")]
fn root_value_and_status(root: &ccm::hp::EigenvalueResult) -> (Option<&rug::Float>, &'static str) {
    match root {
        ccm::hp::EigenvalueResult::Converged(result) => (Some(&result.value), "converged"),
        ccm::hp::EigenvalueResult::Stagnated(result) => (Some(&result.value), "stagnated"),
        ccm::hp::EigenvalueResult::Approximate(result) => (Some(&result.value), "approximate"),
        ccm::hp::EigenvalueResult::Failed { .. } => (None, "failed"),
    }
}

#[cfg(feature = "hp")]
fn comparison_metrics(
    value: &rug::Float,
    reference: &rug::Float,
    precision_bits: u32,
) -> (rug::Float, f64) {
    let mut difference = rug::Float::with_val(precision_bits, value);
    difference -= reference;
    let absolute_error = difference.abs();
    let digits = if absolute_error.is_zero() {
        f64::from(precision_bits) * std::f64::consts::LOG10_2
    } else {
        xc_numerics::fmt::matching_digits(value, reference).to_f64()
    };
    (absolute_error, digits)
}

#[cfg(feature = "hp")]
struct DiscoveryOrderingReportOptions {
    requested_roots: usize,
    reference_zero_limit: usize,
    minimum_match_digits: u32,
    precision_digits: u32,
    display_digits: usize,
    allow_incomplete: bool,
    signed_domain: bool,
}

#[cfg(feature = "hp")]
fn print_discovery_ordering_report(
    result: &ccm::hp::HighPrecResult,
    options: &DiscoveryOrderingReportOptions,
) -> Result<()> {
    let DiscoveryOrderingReportOptions {
        requested_roots,
        reference_zero_limit,
        minimum_match_digits,
        precision_digits,
        display_digits,
        allow_incomplete,
        signed_domain,
    } = *options;
    let roots = result
        .eigenvalues_pos
        .iter()
        .take(requested_roots)
        .collect::<Vec<_>>();
    if roots.len() != requested_roots && !allow_incomplete {
        anyhow::bail!(
            "discovery-ordering report requested {requested_roots} roots but the CCM run returned {}",
            roots.len()
        );
    }
    if roots.is_empty() && !allow_incomplete {
        anyhow::bail!("discovery-ordering report received no finite-source roots");
    }
    if roots.len() < requested_roots {
        println!(
            "  finite discovery shortfall: requested {requested_roots}, reporting all {} roots returned",
            roots.len()
        );
    }
    if roots.is_empty() {
        println!(
            "\nDiscovery-ordering classification: no finite-source roots were returned for comparison"
        );
        println!("\n=== Root-ordering summary ===");
        println!("  requested finite roots: {requested_roots}; returned: 0");
        println!("  identified reference zeros: 0/0 (not applicable)");
        println!("  unmatched finite-source roots: 0/0 (not applicable)");
        println!("  correct ordinal position: 0/0; displaced matches: 0");
        println!("  first reference zero was not present in the finite CCM root window");
        return Ok(());
    }

    let reference_strings = xc_zeta::zeros::bundled_first_n_strings(reference_zero_limit)?;
    let comparison_precision = result.precision_bits.saturating_mul(2);
    let references = reference_strings
        .iter()
        .map(|text| {
            rug::Float::parse(text)
                .map(|parsed| rug::Float::with_val(comparison_precision, parsed))
                .map_err(|error| {
                    anyhow::anyhow!("failed to parse bundled reference zero {text:?}: {error}")
                })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut digit_matrix = vec![vec![None; references.len()]; roots.len()];
    for (root_offset, root) in roots.iter().enumerate() {
        let (Some(value), _) = root_value_and_status(root) else {
            continue;
        };
        let value = rug::Float::with_val(comparison_precision, value);
        for (reference_offset, reference) in references.iter().enumerate() {
            let (_, digits) = comparison_metrics(&value, reference, comparison_precision);
            digit_matrix[root_offset][reference_offset] = Some(digits);
        }
    }
    let assignments = align_discovered_roots(&digit_matrix, f64::from(minimum_match_digits));
    let column_digits = ((precision_digits as f64).log10().ceil() as usize + 2).max(5);

    println!(
        "\nDiscovery-ordering classification: monotone one-to-one alignment against the first {} reference zeros; match threshold >= {} decimal digits",
        references.len(),
        minimum_match_digits
    );
    println!(
        "{:>8}  {:>22}  {:>5}  {:>9}  {:>22}  {:>14}  {:>14}  {:>12}  {:>11}",
        "CCM root",
        "discovered value",
        "match",
        "candidate k",
        "reference zero",
        "abs error",
        "matching digits",
        "displacement",
        "status"
    );
    println!("{}", "-".repeat(142));

    let mut matched = 0usize;
    let mut correct_position = 0usize;
    let mut displacements = Vec::new();
    let mut first_zero_root = None;
    let negative_roots = if signed_domain {
        roots
            .iter()
            .filter(|root| {
                root_value_and_status(root)
                    .0
                    .is_some_and(|value| value < &0)
            })
            .count()
    } else {
        0
    };

    for (root_offset, root) in roots.iter().enumerate() {
        let root_index = if signed_domain {
            if root_offset < negative_roots {
                isize::try_from(root_offset)? - isize::try_from(negative_roots)?
            } else {
                isize::try_from(root_offset - negative_roots + 1)?
            }
        } else {
            isize::try_from(result.first_positive_root_index + root_offset)?
        };
        let (value, status) = root_value_and_status(root);
        let Some(value) = value else {
            println!(
                "{:>8}  {:>22}  {:>5}  {:>9}  {:>22}  {:>14}  {:>14}  {:>12}  {:>11}",
                root_index, "--", "NO", "--", "--", "--", "--", "--", status
            );
            continue;
        };
        let value = rug::Float::with_val(comparison_precision, value);

        let accepted_reference = assignments[root_offset];
        let displayed_reference = accepted_reference.or_else(|| {
            digit_matrix[root_offset]
                .iter()
                .enumerate()
                .filter_map(|(index, digits)| digits.map(|digits| (index, digits)))
                .max_by(|(_, left), (_, right)| {
                    left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(index, _)| index)
        });
        let Some(reference_offset) = displayed_reference else {
            anyhow::bail!("discovery-ordering report has no reference zeros");
        };
        let reference_index = reference_offset + 1;
        let reference = &references[reference_offset];
        let (absolute_error, digits) = comparison_metrics(&value, reference, comparison_precision);
        let accepted = accepted_reference.is_some();
        let displacement = if accepted {
            let displacement = root_index
                .checked_sub(isize::try_from(reference_index)?)
                .ok_or_else(|| anyhow::anyhow!("root displacement overflow"))?;
            matched += 1;
            if displacement == 0 {
                correct_position += 1;
            }
            if reference_index == 1 {
                first_zero_root = Some(root_index);
            }
            displacements.push(displacement);
            displacement.to_string()
        } else {
            "--".to_owned()
        };
        let absolute_error = if absolute_error.is_zero() {
            "0".to_owned()
        } else {
            xc_numerics::fmt::display_hp(&absolute_error, column_digits)
        };
        let matching_digits = if digits.is_finite() {
            format!("{digits:.6}")
        } else {
            format!(">={}", comparison_precision / 3)
        };
        println!(
            "{:>8}  {:>22}  {:>5}  {:>9}  {:>22}  {:>14}  {:>14}  {:>12}  {:>11}",
            root_index,
            xc_numerics::fmt::display_hp(&value, display_digits),
            if accepted { "YES" } else { "NO" },
            reference_index,
            xc_numerics::fmt::display_hp(reference, display_digits),
            absolute_error,
            matching_digits,
            displacement,
            status
        );
    }

    let returned_roots = roots.len();
    let percentage = 100.0 * matched as f64 / returned_roots as f64;
    println!("\n=== Root-ordering summary ===");
    println!("  requested finite roots: {requested_roots}; returned: {returned_roots}");
    println!("  identified reference zeros: {matched}/{returned_roots} ({percentage:.2}%)");
    println!(
        "  unmatched finite-source roots: {}/{} ({:.2}%)",
        returned_roots - matched,
        returned_roots,
        100.0 - percentage
    );
    println!(
        "  correct ordinal position: {correct_position}/{matched}; displaced matches: {}",
        matched.saturating_sub(correct_position)
    );
    if let Some(root_index) = first_zero_root {
        println!(
            "  first reference zero occurs at CCM root {root_index} (displacement {:+})",
            root_index - 1
        );
    } else {
        println!("  first reference zero was not identified in the requested CCM root window");
    }
    if let (Some(minimum), Some(maximum)) = (displacements.iter().min(), displacements.iter().max())
    {
        let max_absolute = displacements
            .iter()
            .map(|value| value.unsigned_abs())
            .max()
            .unwrap_or(0);
        println!(
            "  matched-root displacement range: {minimum:+}..{maximum:+}; maximum absolute displacement: {max_absolute}"
        );
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

#[cfg(any(feature = "hp", test))]
fn explicit_ordinal_root_target(
    first_root_index: usize,
    root_count: usize,
    n_modes: usize,
    allow_oversubscription: bool,
) -> Result<(ccm::window::ZeroTarget, usize)> {
    if first_root_index == 0 || root_count == 0 {
        anyhow::bail!("CCM ordinal root targets require positive indices and counts");
    }
    let last = first_root_index
        .checked_add(root_count - 1)
        .ok_or_else(|| anyhow::anyhow!("requested root index range overflows usize"))?;
    if last > n_modes && !allow_oversubscription {
        anyhow::bail!("requested root window exceeds the finite CCM reach N");
    }
    let target = if first_root_index == 1 {
        ccm::window::ZeroTarget::FirstK { count: root_count }
    } else {
        ccm::window::ZeroTarget::IndexRange {
            first: first_root_index,
            last,
        }
    };
    Ok((target, last))
}

fn research_capture_label(capture: ResearchCapture, maximum_count: usize) -> String {
    match capture {
        ResearchCapture::Claim => "claim (requested roots and native artifacts)".to_string(),
        ResearchCapture::Research => {
            "research (requested roots and detailed artifacts; no sector solve)".to_string()
        }
        ResearchCapture::Gap => {
            "gap (requested roots, evenness, GapLog, 2 eigenpairs per sector)".to_string()
        }
        ResearchCapture::Maximum => format!(
            "maximum (requested roots, evenness, GapLog, {maximum_count} eigenpairs per sector)"
        ),
    }
}

#[cfg(any(feature = "hp", test))]
fn bundled_reference_seed_window(
    target: &ccm::window::ZeroTarget,
) -> Result<(
    xc_zeta::zeros::ReferenceZeroDatasetIdentity,
    usize,
    Vec<String>,
)> {
    let dataset = xc_zeta::zeros::bundled_dataset_identity()?;
    let all = xc_zeta::zeros::bundled_first_n_strings(dataset.record_count)?;
    let indexed_slice = |first: usize, last: usize| -> Result<(usize, Vec<String>)> {
        if first == 0 || first > last || last > all.len() {
            anyhow::bail!(
                "reference-zero dataset {} does not cover requested indices {}..={}",
                dataset.resource_id,
                first,
                last
            );
        }
        Ok((first, all[first - 1..last].to_vec()))
    };
    let (first, seeds) = match target {
        ccm::window::ZeroTarget::FirstK { count } => indexed_slice(1, *count)?,
        ccm::window::ZeroTarget::IndexRange { first, last } => indexed_slice(*first, *last)?,
        ccm::window::ZeroTarget::HeightWindow { .. }
        | ccm::window::ZeroTarget::SymmetricHeightWindow { .. } => anyhow::bail!(
            "reference-seeded CCM refinement requires an explicit ordinal target; capture level cannot convert a height window into reference seeds"
        ),
    };
    if seeds.is_empty() {
        anyhow::bail!("seeded CCM acquisition requires a nonempty reference window");
    }
    Ok((dataset, first, seeds))
}

/// Fill the artifact families not already produced by the command that called
/// this helper. Each toolkit API owns its ordinary managed-cache lifecycle, so
/// author publication settings still apply without publication logic in this
/// consumer repository.
#[cfg(feature = "hp")]
struct SupplementalResearchCaptureOptions {
    capture_roots: bool,
    capture_evenness: bool,
    sector_eigenpairs: Option<usize>,
    complete_sector_spectrum: bool,
    display_digits: usize,
    root_acquisition: RootAcquisitionMode,
}

#[cfg(feature = "hp")]
fn capture_supplemental_research_artifacts(
    params: &CcmParams,
    cfg: &ccm::hp::HighPrecConfig,
    options: SupplementalResearchCaptureOptions,
) -> Result<()> {
    let SupplementalResearchCaptureOptions {
        capture_roots,
        capture_evenness,
        sector_eigenpairs,
        complete_sector_spectrum,
        display_digits,
        root_acquisition,
    } = options;
    let supplemental_started = std::time::Instant::now();
    println!("\n=== Supplemental research artifact capture ===");

    if capture_roots {
        let roots_started = std::time::Instant::now();
        let mut root_cfg = cfg.clone();
        let supplemental_root_count = root_cfg.n_eigenvalues.min(params.n_modes).max(1);
        root_cfg.n_eigenvalues = supplemental_root_count;
        let target = ccm::window::ZeroTarget::FirstK {
            count: supplemental_root_count,
        };
        let roots = match root_acquisition {
            RootAcquisitionMode::Independent => {
                ccm::hp::run_independent(params, &root_cfg, &target)?
            }
            RootAcquisitionMode::Seeded => {
                let (dataset, first, seed_strings) = bundled_reference_seed_window(&target)?;
                let seeds = seed_strings
                    .iter()
                    .map(|seed| {
                        rug::Float::parse(seed)
                            .map(|parsed| rug::Float::with_val(root_cfg.precision_bits, parsed))
                            .map_err(|error| {
                                anyhow::anyhow!(
                                    "failed to parse bundled reference seed {seed:?}: {error}"
                                )
                            })
                    })
                    .collect::<Result<Vec<_>>>()?;
                ccm::hp::run_indexed_seeded(params, &root_cfg, first, &seeds, &dataset)?
            }
        };
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
            "  bounded first-{} root window captured via {:?}: {} converged, {} stagnated, {} approximate, {} failed; elapsed={:.3}s",
            supplemental_root_count,
            root_acquisition,
            counts.0,
            counts.1,
            counts.2,
            counts.3,
            roots_started.elapsed().as_secs_f64()
        );
    } else {
        println!("  requested root window captured by the primary run");
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
            "--root-acquisition",
            "independent",
            "--root-validation",
            "certified",
            "--root-enclosure-digits",
            "75",
        ])
        .is_ok());
        assert!(Cli::try_parse_from([
            "ccm-reproduction",
            "check-evenness",
            "--research-capture",
            "gap",
            "--research-sector-eigenpairs",
            "10",
            "--root-acquisition",
            "seeded",
        ])
        .is_ok());
    }

    #[test]
    fn paper_root_acquisition_defaults_to_seeded() {
        let cli = Cli::try_parse_from(["ccm-reproduction", "run"]).unwrap();
        let Command::Run {
            root_acquisition, ..
        } = cli.command
        else {
            panic!("run command was not parsed");
        };
        assert_eq!(root_acquisition, RootAcquisitionMode::Seeded);

        let cli = Cli::try_parse_from([
            "ccm-reproduction",
            "run",
            "--root-acquisition",
            "independent",
        ])
        .unwrap();
        let Command::Run {
            root_acquisition, ..
        } = cli.command
        else {
            panic!("run command was not parsed");
        };
        assert_eq!(root_acquisition, RootAcquisitionMode::Independent);
    }

    #[test]
    fn discovery_ordering_flags_parse() {
        let cli = Cli::try_parse_from([
            "ccm-reproduction",
            "run",
            "--root-acquisition",
            "independent",
            "--root-report",
            "discovery-ordering",
            "--minimum-match-digits",
            "12",
            "--reference-zero-limit",
            "500",
            "--include-negative-roots",
            "--allow-root-oversubscription",
        ])
        .unwrap();
        let Command::Run {
            root_report,
            minimum_match_digits,
            reference_zero_limit,
            include_negative_roots,
            allow_root_oversubscription,
            ..
        } = cli.command
        else {
            panic!("run command was not parsed");
        };
        assert_eq!(root_report, RootReport::DiscoveryOrdering);
        assert_eq!(minimum_match_digits, 12);
        assert_eq!(reference_zero_limit, 500);
        assert!(include_negative_roots);
        assert!(allow_root_oversubscription);
    }

    #[test]
    fn discovery_alignment_preserves_root_order_and_spurious_entries() {
        let digits = vec![
            vec![Some(1.0), Some(0.5), Some(0.1)],
            vec![Some(80.0), Some(2.0), Some(1.0)],
            vec![Some(3.0), Some(75.0), Some(2.0)],
            vec![Some(1.0), Some(4.0), Some(70.0)],
        ];
        assert_eq!(
            align_discovered_roots(&digits, 10.0),
            vec![None, Some(0), Some(1), Some(2)]
        );
    }

    #[test]
    fn discovery_alignment_handles_a_missing_reference_zero() {
        let digits = vec![
            vec![Some(60.0), Some(1.0), Some(0.5)],
            vec![Some(0.5), Some(2.0), Some(55.0)],
        ];
        assert_eq!(
            align_discovered_roots(&digits, 10.0),
            vec![Some(0), Some(2)]
        );
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
    fn claim_root_targets_are_bounded_explicit_ordinals() {
        let (prefix, last) = explicit_ordinal_root_target(1, 25, 120, false).unwrap();
        assert!(matches!(
            prefix,
            ccm::window::ZeroTarget::FirstK { count: 25 }
        ));
        assert_eq!(last, 25);

        let (later, last) = explicit_ordinal_root_target(51, 25, 120, false).unwrap();
        assert!(matches!(
            later,
            ccm::window::ZeroTarget::IndexRange {
                first: 51,
                last: 75
            }
        ));
        assert_eq!(last, 75);
        assert!(explicit_ordinal_root_target(101, 25, 120, false).is_err());
        let (advanced, last) = explicit_ordinal_root_target(1, 200, 100, true).unwrap();
        assert!(matches!(
            advanced,
            ccm::window::ZeroTarget::FirstK { count: 200 }
        ));
        assert_eq!(last, 200);
    }

    #[test]
    fn seeded_windows_require_explicit_ordinals() {
        let (dataset, first, seeds) =
            bundled_reference_seed_window(&ccm::window::ZeroTarget::IndexRange {
                first: 1,
                last: 25,
            })
            .unwrap();
        assert!(dataset.validate());
        assert_eq!(first, 1);
        assert_eq!(seeds.len(), 25);
        assert!(
            bundled_reference_seed_window(&ccm::window::ZeroTarget::HeightWindow {
                lower: "0".to_owned(),
                upper: "300".to_owned(),
            })
            .is_err()
        );
    }
}
