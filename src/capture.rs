//! Durable, private run journals and the toolkit's shared capture recipe.
use clap::Args;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapturePolicy {
    Full,
    Claim1cHp1000,
    Claim8Natural,
}

#[derive(Clone, Debug, Args)]
pub struct CaptureJournalArgs {
    /// Named applicability policy; exclusions are recorded before computation.
    #[arg(long, global = true, value_enum, default_value = "full")]
    pub capture_policy: CapturePolicy,
    /// Parent directory for unique private claim journals (primary, receipt, events).
    #[arg(long, global = true, default_value = ".xcelerator-cache/claim-runs")]
    pub capture_output: PathBuf,
    /// Additional retained prefix dimensions k=N+1. Missing states remain missing.
    #[arg(long, global = true, value_delimiter = ',')]
    pub capture_prefix_checkpoints: Vec<usize>,
    /// Arithmetic precision for retained diagnostics; cannot lower source precision.
    #[arg(long, global = true)]
    pub capture_working_precision_bits: Option<u32>,
    /// Override the default distance convention grid for every claim command.
    #[arg(long, global = true)]
    pub capture_grid_resolution: Option<usize>,
    /// Override retained eigenfunction sampling for every claim command.
    #[arg(long, global = true)]
    pub capture_profile_steps: Option<usize>,
    /// Explicit cubic reduction budget for Ultra; larger sources are recorded blocked.
    #[arg(long, global = true, default_value_t = 8193)]
    pub capture_reduction_max_dimension: usize,
    /// Return failure after saving the journal if any requested capture is incomplete.
    #[arg(long, global = true)]
    pub require_complete_capture: bool,
}

#[cfg(feature = "hp")]
pub use hp::{execute, write_evenness, write_measurements};

#[cfg(feature = "hp")]
mod hp {
    use super::*;
    use crate::ResearchCapture;
    use anyhow::{Context, Result};
    use serde::Serialize;
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        path::Path,
        time::{Instant, SystemTime, UNIX_EPOCH},
    };
    use xc_cache::{ArtifactCacheContext, CaptureFailure};
    use xc_spectral::ccm::{
        capture::{CcmCaptureLevel, CcmCapturePlan, RetainedReductionRequest},
        hp::{
            capture_run::RetainedCcmRun, CcmResearchCaptureOptions, HighPrecConfig, HighPrecResult,
            PortableHighPrecResult,
        },
        CcmParams,
    };

    /// These exclusions belong to the frozen Paper 1 reproduction, not to the
    /// toolkit's general Ultra recipe. Never infer them from a failed attempt.
    fn exclusions(
        policy: CapturePolicy,
        level: ResearchCapture,
        params: &CcmParams,
        cfg: &HighPrecConfig,
        args: &CaptureJournalArgs,
    ) -> Result<std::collections::BTreeMap<String, String>> {
        let mut result = std::collections::BTreeMap::new();
        if policy == CapturePolicy::Full {
            return Ok(result);
        }
        if policy == CapturePolicy::Claim8Natural {
            anyhow::ensure!(
                level == ResearchCapture::Ultra
                    && matches!((params.lambda_sq_int(), params.n_modes), (13, 120) | (100, 500))
                    && cfg.precision_bits == 3386
                    && cfg.effective_parity_policy() == xc_spectral::ccm::hp::CcmParityPolicy::Natural,
                "claim8-natural capture policy requires Ultra, (lambda-squared,N)=(13,120) or (100,500), HP-1000 (3386 bits), and natural parity"
            );
            anyhow::ensure!(
                args.capture_prefix_checkpoints.is_empty()
                    && args.capture_working_precision_bits.is_none(),
                "claim8-natural uses the frozen prefix recipe; use a separate full-policy research run for checkpoint or precision overrides"
            );
            for id in ["prime_power_response", "u_flow_response"] {
                result.insert(id.into(), "this response requires an isolated even-sector primary eigenstate; Claim 8 preserves the unrestricted natural primary and captures this response on the paired even route".into());
            }
            result.insert(format!("prefix_checkpoint_{}", params.n_modes + 1), "the eigenstate checkpoint requires an exact even-sector source; the natural route retains the prefix ladder and innovation export without substituting or projecting its primary state".into());
            return Ok(result);
        }
        anyhow::ensure!(
            level == ResearchCapture::Ultra
                && params.lambda_sq_int() == 1000
                && params.n_modes == 800
                && cfg.precision_bits == 3386
                && cfg.effective_parity_policy() == xc_spectral::ccm::hp::CcmParityPolicy::EvenSector,
            "claim1c-hp1000 capture policy requires Ultra, lambda-squared=1000, N=800, HP-1000 (3386 bits), and even-sector parity"
        );
        anyhow::ensure!(
            args.capture_prefix_checkpoints.is_empty()
                && args.capture_working_precision_bits.is_none(),
            "claim1c-hp1000 excludes prefix diagnostics; use a separate full-policy research run for prefix or precision overrides"
        );
        for id in ["prime_power_response", "u_flow_response"] {
            result.insert(id.into(), "HP-1000 cannot isolate the lowest sector eigenvalues for this configuration; response derivatives require an isolated eigenstate".into());
        }
        for id in ["prefix_ladder", "prefix_checkpoint_801"] {
            result.insert(id.into(), "the HP-1000 retained prefix ladder reaches a nonpositive computed pivot at dimension 593; the dimension-801 export is unresolved, not a definiteness result".into());
        }
        for id in [
            "target_distance",
            "distance_resolution",
            "target_residual_analysis",
            "deviation_decomposition",
        ] {
            result.insert(id.into(), "the HP-1000 ground state is under-resolved and uniform-u target-distance refinement failed its tolerance through Q=16000; target comparisons are excluded, while the raw retained profile is preserved".into());
        }
        Ok(result)
    }

    fn requested_diagnostics(
        plan: &CcmCapturePlan,
        reduction: bool,
        excluded: &std::collections::BTreeMap<String, String>,
    ) -> Result<Vec<String>> {
        let mut requested = plan
            .receipt()?
            .outcomes()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        if reduction {
            requested.push("retained_reduction".into());
        }
        anyhow::ensure!(
            excluded.keys().all(|id| requested.contains(id)),
            "capture policy excludes an unknown diagnostic"
        );
        requested.retain(|id| !excluded.contains_key(id));
        requested.sort();
        Ok(requested)
    }

    fn save(path: &Path, value: &impl Serialize) -> Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        serde_json::to_writer_pretty(&mut file, value)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        Ok(())
    }

    pub struct CapturedRun {
        pub primary: HighPrecResult,
        pub directory: PathBuf,
    }

    pub fn write_measurements(directory: &Path, value: &impl Serialize) -> Result<()> {
        let path = directory.join("claim-measurements.json");
        save(&path, value)?;
        println!("Claim measurements: {}", path.display());
        Ok(())
    }

    pub fn write_evenness(args: &CaptureJournalArgs, value: &impl Serialize) -> Result<()> {
        fs::create_dir_all(&args.capture_output)?;
        let directory = args.capture_output.join(format!(
            "evenness-{}-{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
            std::process::id()
        ));
        fs::create_dir(&directory)?;
        write_measurements(&directory, value)
    }

    fn event(file: &mut std::fs::File, value: &impl Serialize) -> Result<()> {
        serde_json::to_writer(&mut *file, value)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        Ok(())
    }

    fn recipe(
        level: ResearchCapture,
        count: usize,
        dimension: usize,
        args: &CaptureJournalArgs,
        source_bits: u32,
    ) -> Result<CcmCapturePlan> {
        let level = match level {
            ResearchCapture::Claim => CcmCaptureLevel::Claim,
            ResearchCapture::Research => CcmCaptureLevel::Research,
            ResearchCapture::Gap => CcmCaptureLevel::Gap,
            ResearchCapture::Maximum => CcmCaptureLevel::Maximum,
            ResearchCapture::Ultra => CcmCaptureLevel::Ultra,
        };
        let mut plan =
            CcmCapturePlan::resolve(level, count.min(dimension.saturating_sub(1)), dimension)?;
        if !args.capture_prefix_checkpoints.is_empty() {
            let mut checkpoints = args.capture_prefix_checkpoints.clone();
            checkpoints.push(dimension);
            checkpoints.sort_unstable();
            checkpoints.dedup();
            plan = plan.with_prefix_checkpoints(checkpoints)?;
        }
        if let Some(bits) = args.capture_working_precision_bits {
            plan = plan.with_prefix_working_precision(bits)?;
        }
        plan.validate_for_source_precision(source_bits)?;
        Ok(plan)
    }

    /// Primary failure remains fatal. Supplemental failures are persisted and
    /// never turn an unavailable measurement into a positive scientific result.
    #[allow(clippy::too_many_arguments)]
    pub fn execute<F>(
        params: &CcmParams,
        cfg: &HighPrecConfig,
        level: ResearchCapture,
        count: usize,
        args: &CaptureJournalArgs,
        options: &CcmResearchCaptureOptions,
        acquire: F,
    ) -> Result<CapturedRun>
    where
        F: FnOnce(&ArtifactCacheContext<'_>) -> Result<RetainedCcmRun>,
    {
        let mut options = options.clone();
        if let Some(distance) = &mut options.distance_capture {
            if args.capture_grid_resolution.is_some() || args.capture_profile_steps.is_some() {
                let grid = args
                    .capture_grid_resolution
                    .unwrap_or_else(|| distance.rules[0].resolution());
                let replacement =
                    xc_spectral::ccm::hp::CcmDistanceCaptureOptions::default_convention(
                        grid,
                        args.capture_profile_steps.unwrap_or(distance.profile_steps),
                    );
                distance.rules = replacement.rules;
                distance.profile_steps = replacement.profile_steps;
            }
        }
        let options = &options;
        let plan = recipe(level, count, params.n_modes + 1, args, cfg.precision_bits)?;
        let excluded = exclusions(args.capture_policy, level, params, cfg, args)?;
        let started = Instant::now();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let directory = args.capture_output.join(format!(
            "c{}-n{}-p{}-{timestamp}-{}",
            params.lambda_sq_int(),
            params.n_modes,
            cfg.precision_bits,
            std::process::id()
        ));
        fs::create_dir_all(&args.capture_output)?;
        fs::create_dir(&directory)?;
        println!("  private claim journal: {}", directory.display());
        let mut events = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("events.jsonl"))?;
        let reduction = (level == ResearchCapture::Ultra).then(|| RetainedReductionRequest {
            working_precision_bits: args
                .capture_working_precision_bits
                .unwrap_or(cfg.precision_bits),
            maximum_dimension: args.capture_reduction_max_dimension,
            relative_tolerance: format!(
                "1e-{}",
                (cfg.precision_bits.saturating_sub(48) as usize * 3 / 10).max(1)
            ),
        });
        let requested = requested_diagnostics(&plan, reduction.is_some(), &excluded)?;
        let applicability = serde_json::json!({"schema_version":1,"policy":args.capture_policy,"requested_diagnostics":requested,"excluded_diagnostics":excluded});
        println!(
            "  capture applicability: {} ({} requested, {} excluded)",
            serde_json::to_value(args.capture_policy)?
                .as_str()
                .unwrap_or("unknown"),
            requested.len(),
            excluded.len()
        );
        for (id, reason) in &excluded {
            println!("  [EXCLUDED] {id}: {reason}");
        }
        let resolved = serde_json::json!({"schema_version":1,"paper_version":env!("CARGO_PKG_VERSION"),"toolkit_release":"0.15.0","capture":plan,"applicability":applicability,"retained_reduction":reduction,"lambda_squared":params.lambda_sq_int(),"n_modes":params.n_modes,"precision_bits":cfg.precision_bits,"parity_policy":cfg.effective_parity_policy().as_str(),"distance":options.distance_capture.as_ref().map(|d|serde_json::json!({"alpha":d.alpha,"rules":d.rules.iter().map(|r|serde_json::json!({"family":r.family(),"rule":r.rule(),"variable":r.variable().as_str(),"resolution":r.resolution()})).collect::<Vec<_>>(),"profile_steps":d.profile_steps})),"source_policy":"resolve compatible identities; preserve historical artifacts","assurance":"capture completeness is separate from numerical acceptance"});
        save(&directory.join("request.json"), &resolved)?;
        // Preserve the exact dependency lockfile with each run, including the
        // qualified Git commit after a release tag has been amended.
        save(
            &directory.join("build.json"),
            &serde_json::json!({"cargo_lock":include_str!("../Cargo.lock")}),
        )?;
        let managed = xc_cache::ManagedArtifactCacheSession::from_environment()?
            .context("managed cache required")?;
        let cache = managed.context();
        event(
            &mut events,
            &serde_json::json!({"phase":"primary","status":"started"}),
        )?;
        let mut run = match acquire(&cache) {
            Ok(run) => run,
            Err(error) => {
                save(
                    &directory.join("status.json"),
                    &serde_json::json!({"status":"primary_failed","failure":CaptureFailure::failed(&error),"elapsed_seconds":started.elapsed().as_secs_f64()}),
                )?;
                return Err(error);
            }
        };
        save(
            &directory.join("primary.json"),
            &PortableHighPrecResult::from_runtime(run.primary())?,
        )?;
        save(
            &directory.join("primary-sources.json"),
            &run.primary_sources(),
        )?;
        event(
            &mut events,
            &serde_json::json!({"phase":"primary","status":"saved","elapsed_seconds":started.elapsed().as_secs_f64()}),
        )?;
        let retained = if plan.capture_prefix_analysis {
            match run.retained_even_sources(&cache) {
                Ok(sources) => Some(sources),
                Err(error) => {
                    event(
                        &mut events,
                        &serde_json::json!({"phase":"retained_sources","failure":CaptureFailure::failed(&error)}),
                    )?;
                    None
                }
            }
        } else {
            None
        };
        let target = xc_spectral::target::TargetProfileSpec::from_environment();
        let target_missing = target
            .as_ref()
            .err()
            .map(|e| format!("runtime target unavailable: {e}"));
        let mut execute = |id: &str| {
            let timer = Instant::now();
            println!("  capture {id}: started");
            event(
                &mut events,
                &serde_json::json!({"diagnostic":id,"status":"started"}),
            )
            .map_err(CaptureFailure::failed)?;
            let result = if matches!(id, "prime_power_response" | "u_flow_response")
                && cfg.effective_parity_policy()
                    != xc_spectral::ccm::hp::CcmParityPolicy::EvenSector
            {
                Err(CaptureFailure::Blocked {
                    reason: "requires an isolated even-sector state; primary parity preserved"
                        .into(),
                })
            } else if matches!(
                id,
                "target_distance"
                    | "distance_resolution"
                    | "target_residual_analysis"
                    | "deviation_decomposition"
            ) && target_missing.is_some()
            {
                Err(CaptureFailure::Missing {
                    reason: target_missing.clone().expect("missing target"),
                })
            } else {
                run.capture_diagnostic(id, options, &cache)
                    .map_err(CaptureFailure::failed)
            };
            let status = match &result {
                Ok(_) => "completed",
                Err(CaptureFailure::Missing { .. }) => "missing",
                Err(CaptureFailure::Blocked { .. }) => "blocked",
                Err(_) => "failed",
            };
            println!(
                "  capture {id}: {status} ({:.3}s)",
                timer.elapsed().as_secs_f64()
            );
            if let Err(failure) = &result {
                println!("  capture {id} details: {}", serde_json::json!(failure));
            }
            event(&mut events, &serde_json::json!({"diagnostic":id,"status":status,"failure":result.as_ref().err(),"elapsed_seconds":timer.elapsed().as_secs_f64()})).map_err(CaptureFailure::failed)?;
            result
        };
        let record = if excluded.is_empty() {
            plan.execute_with_receipt_and_reduction(
                retained.as_ref().map(|(m, e)| (m, e.as_slice())),
                reduction.as_ref(),
                &cache,
                &mut execute,
            )
        } else {
            // The persisted receipt binds both the effective request and every
            // exclusion reason. Excluded work never enters the callback.
            xc_cache::capture_and_persist(
                &resolved,
                requested.clone(),
                |id| {
                    if id == "prefix_ladder" {
                        let (matrix, eigenpairs) =
                            retained.as_ref().ok_or_else(|| CaptureFailure::Missing {
                                reason: "retained even matrix unavailable".into(),
                            })?;
                        let result = plan
                            .capture_retained_diagnostics(matrix, eigenpairs, &cache)
                            .map_err(CaptureFailure::failed)?
                            .ok_or_else(|| CaptureFailure::failed("prefix capture is disabled"))?;
                        let sources = result
                            .produced_manifest
                            .as_ref()
                            .or(result.reused_manifest.as_ref())
                            .map(|m| vec![m.clone()])
                            .unwrap_or_else(|| vec![matrix.manifest().clone()]);
                        return xc_cache::CapturedDiagnostic::new(&result.value, sources)
                            .map_err(CaptureFailure::failed);
                    }
                    if id != "retained_reduction" {
                        return execute(id);
                    }
                    let (matrix, _) = retained.as_ref().ok_or_else(|| CaptureFailure::Missing {
                        reason: "retained even matrix unavailable".into(),
                    })?;
                    let budget = reduction
                        .as_ref()
                        .expect("requested reduction has a budget");
                    if matrix.dimension() > budget.maximum_dimension
                        || matrix.source_precision_bits() > budget.working_precision_bits
                    {
                        return Err(CaptureFailure::Blocked {
                            reason:
                                "retained reduction exceeds dimension or source precision budget"
                                    .into(),
                        });
                    }
                    let result = xc_spectral::ccm::prefix::check_retained_reduction_via_cache(
                        matrix,
                        budget.working_precision_bits,
                        budget.maximum_dimension,
                        &budget.relative_tolerance,
                        &cache,
                    )
                    .map_err(CaptureFailure::failed)?;
                    let sources = result
                        .produced_manifest
                        .as_ref()
                        .or(result.reused_manifest.as_ref())
                        .map(|m| vec![m.clone()])
                        .unwrap_or_else(|| vec![matrix.manifest().clone()]);
                    xc_cache::CapturedDiagnostic::new(&result.value, sources)
                        .map_err(CaptureFailure::failed)
                },
                &cache,
            )
            .map_err(Into::into)
        };
        let record = match record {
            Ok(record) => record,
            Err(error) => {
                save(
                    &directory.join("status.json"),
                    &serde_json::json!({"status":"capture_persistence_failed","primary_saved":true,"failure":CaptureFailure::failed(&error)}),
                )?;
                return Err(error);
            }
        };
        save(&directory.join("capture.json"), &record.value)?;
        let mut complete = record.value.receipt.is_complete();
        // Explicit requests below their named capture level and certificates
        // receive their own source-bound receipt, with no hidden policy change.
        let mut extra = Vec::new();
        let present = record.value.receipt.outcomes();
        for (id, wanted) in [
            ("prime_power_response", options.capture_prime_power_response),
            ("u_flow_response", options.capture_u_flow_response),
            ("root_certificate", options.root_certification.is_some()),
            (
                "sector_gap_certificate",
                options.sector_gap_certification.is_some(),
            ),
            ("distance_profile", options.distance_capture.is_some()),
            ("target_distance", options.distance_capture.is_some()),
            (
                "distance_resolution",
                options
                    .distance_capture
                    .as_ref()
                    .is_some_and(|d| d.capture_resolution_evidence),
            ),
            (
                "target_residual_analysis",
                options
                    .distance_capture
                    .as_ref()
                    .is_some_and(|d| d.capture_residual_analysis),
            ),
            (
                "deviation_decomposition",
                options
                    .distance_capture
                    .as_ref()
                    .is_some_and(|d| d.capture_deviation_decomposition),
            ),
        ] {
            if wanted && !present.contains_key(id) && !excluded.contains_key(id) {
                extra.push(id.to_string());
            }
        }
        if !extra.is_empty() {
            let extra_record = xc_cache::capture_and_persist(
                &serde_json::json!({"paper_capture":resolved,"explicit_diagnostics":extra}),
                extra,
                &mut execute,
                &cache,
            )?;
            complete &= extra_record.value.receipt.is_complete();
            save(
                &directory.join("capture-explicit.json"),
                &extra_record.value,
            )?;
        }
        for (id, outcome) in record.value.receipt.outcomes() {
            println!(
                "  receipt {id}: {}",
                match outcome {
                    xc_core::DiagnosticOutcome::Completed { .. } => "completed",
                    xc_core::DiagnosticOutcome::Missing { .. } => "missing",
                    xc_core::DiagnosticOutcome::Blocked { .. } => "blocked",
                    xc_core::DiagnosticOutcome::Failed { .. } => "failed",
                    _ => "pending",
                }
            );
        }
        managed.finalize_publication_inventory()?;
        save(
            &directory.join("status.json"),
            &serde_json::json!({"status":if complete{"capture_complete"}else{"capture_incomplete"},"primary_saved":true,"capture_complete":complete,"applicability":applicability,"numerical_acceptance":"inspect primary and measurement payloads; completion is not validity","elapsed_seconds":started.elapsed().as_secs_f64()}),
        )?;
        println!(
            "  capture journal saved: {} ({})",
            directory.display(),
            if complete {
                "complete"
            } else {
                "incomplete; see receipt reasons"
            }
        );
        if args.require_complete_capture && !complete {
            anyhow::bail!(
                "capture incomplete; primary and receipts preserved in {}",
                directory.display()
            );
        }
        Ok(CapturedRun {
            primary: run.into_primary(),
            directory,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use clap::Parser;

        fn args() -> CaptureJournalArgs {
            crate::Cli::try_parse_from([
                "ccm-reproduction",
                "run",
                "--capture-policy",
                "claim1c-hp1000",
            ])
            .unwrap()
            .capture_journal
        }

        #[test]
        fn claim1c_policy_is_confined_to_the_frozen_configuration() {
            let mut args = args();
            for (c, n, digits) in [(13, 120, 1000), (1000, 890, 1000), (1000, 800, 2000)] {
                assert!(exclusions(
                    args.capture_policy,
                    ResearchCapture::Ultra,
                    &CcmParams::from_lambda_sq_integer(c, n),
                    &HighPrecConfig::for_decimal_digits(digits),
                    &args
                )
                .is_err());
            }
            let params = CcmParams::from_lambda_sq_integer(1000, 800);
            let mut cfg = HighPrecConfig::for_decimal_digits(1000);
            assert!(exclusions(
                args.capture_policy,
                ResearchCapture::Maximum,
                &params,
                &cfg,
                &args
            )
            .is_err());
            cfg.parity_policy = xc_spectral::ccm::hp::CcmParityPolicy::Natural;
            assert!(exclusions(
                args.capture_policy,
                ResearchCapture::Ultra,
                &params,
                &cfg,
                &args
            )
            .is_err());
            cfg = HighPrecConfig::for_decimal_digits(1000);
            args.capture_prefix_checkpoints.push(593);
            assert!(exclusions(
                args.capture_policy,
                ResearchCapture::Ultra,
                &params,
                &cfg,
                &args
            )
            .is_err());
        }

        #[test]
        fn excluded_diagnostics_are_not_executed_or_reported_as_successful() {
            let args = args();
            let plan = CcmCapturePlan::ultra(8, 801).unwrap();
            let excluded = exclusions(
                args.capture_policy,
                ResearchCapture::Ultra,
                &CcmParams::from_lambda_sq_integer(1000, 800),
                &HighPrecConfig::for_decimal_digits(1000),
                &args,
            )
            .unwrap();
            let requested = requested_diagnostics(&plan, true, &excluded).unwrap();
            assert_eq!(
                requested,
                [
                    "distance_profile",
                    "evenness",
                    "retained_reduction",
                    "root_conditioning",
                    "sector_analysis"
                ]
            );
            assert_eq!(excluded.len(), 8);
            let resolved = serde_json::json!({"excluded_diagnostics":excluded, "requested_diagnostics":requested});
            for fail in [false, true] {
                let mut executed = Vec::new();
                let record = xc_cache::collect_capture(&resolved, requested.clone(), |id| {
                    assert!(!excluded.contains_key(id));
                    executed.push(id.to_string());
                    if fail && id == "root_conditioning" {
                        return Err(CaptureFailure::failed("unexpected source failure"));
                    }
                    xc_cache::CapturedDiagnostic::new(&serde_json::json!({"test":true}), vec![])
                        .map_err(CaptureFailure::failed)
                })
                .unwrap();
                assert_eq!(executed, requested);
                assert_eq!(record.receipt.is_complete(), !fail);
                assert_eq!(record.receipt.outcomes().len(), 5);
            }
        }

        #[test]
        fn full_ultra_keeps_every_request_and_its_existing_recipe() {
            let args = args();
            let excluded = exclusions(
                CapturePolicy::Full,
                ResearchCapture::Ultra,
                &CcmParams::from_lambda_sq_integer(1000, 800),
                &HighPrecConfig::for_decimal_digits(1000),
                &args,
            )
            .unwrap();
            assert!(excluded.is_empty());
            assert_eq!(
                requested_diagnostics(&CcmCapturePlan::ultra(8, 801).unwrap(), true, &excluded)
                    .unwrap()
                    .len(),
                13
            );
        }

        #[test]
        fn claim8_policy_keeps_innovations_and_rejects_unrelated_runs() {
            let mut args = args();
            args.capture_policy = CapturePolicy::Claim8Natural;
            let mut cfg = HighPrecConfig::for_decimal_digits(1000);
            cfg.parity_policy = xc_spectral::ccm::hp::CcmParityPolicy::Natural;
            for (c, n) in [(13, 120), (100, 500)] {
                let params = CcmParams::from_lambda_sq_integer(c, n);
                let excluded = exclusions(
                    args.capture_policy,
                    ResearchCapture::Ultra,
                    &params,
                    &cfg,
                    &args,
                )
                .unwrap();
                let plan = CcmCapturePlan::ultra(8, n + 1).unwrap();
                let requested = requested_diagnostics(&plan, true, &excluded).unwrap();
                assert_eq!(excluded.len(), 3);
                assert!(excluded.contains_key(&format!("prefix_checkpoint_{}", n + 1)));
                assert_eq!(requested.len(), 10);
                assert!(requested.contains(&"prefix_ladder".into()));
                assert!(requested.contains(&"retained_reduction".into()));
                for fail in [false, true] {
                    let record = xc_cache::collect_capture(
                        &serde_json::json!({"applicability":excluded}),
                        requested.clone(),
                        |id| {
                            assert!(!excluded.contains_key(id));
                            if fail && id == "prefix_ladder" {
                                return Err(CaptureFailure::failed("unexpected prefix failure"));
                            }
                            xc_cache::CapturedDiagnostic::new(
                                &serde_json::json!({"test":true}),
                                vec![],
                            )
                            .map_err(CaptureFailure::failed)
                        },
                    )
                    .unwrap();
                    assert_eq!(record.receipt.is_complete(), !fail);
                    assert_eq!(record.receipt.outcomes().len(), 10);
                }
            }
            for (c, n, digits, level, natural, checkpoint, bits) in [
                (13, 121, 1000, ResearchCapture::Ultra, true, false, false),
                (100, 120, 1000, ResearchCapture::Ultra, true, false, false),
                (13, 120, 2000, ResearchCapture::Ultra, true, false, false),
                (13, 120, 1000, ResearchCapture::Maximum, true, false, false),
                (13, 120, 1000, ResearchCapture::Ultra, false, false, false),
                (13, 120, 1000, ResearchCapture::Ultra, true, true, false),
                (13, 120, 1000, ResearchCapture::Ultra, true, false, true),
            ] {
                let mut cfg = HighPrecConfig::for_decimal_digits(digits);
                if natural {
                    cfg.parity_policy = xc_spectral::ccm::hp::CcmParityPolicy::Natural;
                }
                args.capture_prefix_checkpoints = if checkpoint { vec![100] } else { vec![] };
                args.capture_working_precision_bits = bits.then_some(4000);
                assert!(exclusions(
                    args.capture_policy,
                    level,
                    &CcmParams::from_lambda_sq_integer(c, n),
                    &cfg,
                    &args
                )
                .is_err());
            }
        }
    }
}
