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
pub use hp::{execute, replay_finite, validate_configuration, write_evenness, write_measurements};

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

    /// These exclusions belong to the frozen paper reproduction, not to the
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

    /// Measurements that `scripts/claim_summary.py` reviews numerically.
    const REVIEWED_MEASUREMENTS: [&str; 3] =
        ["distance_resolution", "prefix_ladder", "retained_reduction"];

    /// Toolkit 0.16.0 records a measurement whose value is exactly a retained
    /// artifact payload by reference. Save the reviewed values, reconstructed
    /// and digest-checked by the toolkit, beside the receipt so the summary can
    /// assess them. An unresolved reference fails the export rather than
    /// silently omitting requested evidence.
    fn save_reviewed_values(
        path: &Path,
        record: &xc_cache::CaptureArtifact,
        cache: &xc_cache::ArtifactCacheContext<'_>,
    ) -> Result<()> {
        let mut values = serde_json::Map::new();
        for name in REVIEWED_MEASUREMENTS.into_iter().chain(
            xc_spectral::ccm::capture::FINITE_DIAGNOSTICS
                .iter()
                .copied(),
        ) {
            let Some(measurement) = record.measurements.get(name) else {
                continue;
            };
            if measurement.value_reference.is_none() {
                continue;
            }
            let resolver = cache
                .resolver
                .context("capture export requires a cache resolver")?;
            let policy = cache
                .acceptance
                .context("capture export requires a cache acceptance policy")?;
            match xc_cache::measurement_value(measurement, resolver, policy) {
                Ok(value) => {
                    values.insert(name.to_owned(), value);
                }
                Err(error) => {
                    anyhow::bail!("{name}: referenced capture value not resolved: {error}")
                }
            }
        }
        save(path, &values)
    }

    pub struct CapturedRun {
        pub primary: HighPrecResult,
        pub directory: PathBuf,
    }

    /// Supplemental acquisition reads named retained artifacts only. Missing
    /// sources fail; there is no path to a new primary solve in this function.
    pub fn replay_finite(journal: &Path, output: &Path, diagnostics: &[String]) -> Result<()> {
        use xc_spectral::ccm::convergence_capture::finite_capture::RetainedFiniteSources;
        let managed = local_replay_session()?;
        anyhow::ensure!(
            !diagnostics.is_empty()
                && diagnostics
                    .iter()
                    .all(|id| xc_spectral::ccm::capture::FINITE_DIAGNOSTICS.contains(&id.as_str())),
            "unknown or empty supplemental diagnostic request"
        );
        let unique: std::collections::BTreeSet<_> = diagnostics.iter().collect();
        anyhow::ensure!(
            unique.len() == diagnostics.len(),
            "duplicate supplemental diagnostic"
        );
        let source_path = journal.join("primary-sources.json");
        anyhow::ensure!(
            fs::metadata(&source_path)?.len() <= 64 * 1024 * 1024,
            "retained manifest inventory exceeds admission budget"
        );
        let bytes = fs::read(source_path)?;
        let manifests: Vec<xc_cache::ArtifactManifest> = serde_json::from_slice(&bytes)?;
        let cache = managed.context();
        let need_matrix = diagnostics.iter().any(|id| {
            matches!(
                id.as_str(),
                "trial_vector_energy"
                    | "trial_vector_parity"
                    | "directional_error_bound"
                    | "finite_tail_bound"
                    | "spectral_cluster_bound"
            )
        });
        let sources = RetainedFiniteSources::from_manifests(&manifests, need_matrix, &cache)?;
        let research_path = journal.join("research-sources.json");
        let research_sources: Vec<xc_cache::ArtifactManifest> = if research_path.exists() {
            anyhow::ensure!(
                fs::metadata(&research_path)?.len() <= 64 * 1024 * 1024,
                "retained research manifest inventory exceeds admission budget"
            );
            serde_json::from_slice(&fs::read(research_path)?)?
        } else {
            Vec::new()
        };
        let input_path = journal.join("external-research-inputs.json");
        let input = if input_path.exists() {
            Some(
                xc_spectral::ccm::extended_research::ExternalResearchInputs::from_file(
                    &input_path,
                )?,
            )
        } else {
            None
        };
        // Never replace or mutate the primary journal or an earlier replay.
        fs::create_dir(output)?;
        let request = serde_json::json!({"schema_version":1,"phase":"supplemental_retained_analysis",
            "primary_sources_sha256":xc_cache::ContentDigest::sha256(&bytes),"sources":manifests,
            "diagnostics":diagnostics,"research_sources":research_sources,"input_sha256":input.as_ref().map(|i|serde_json::to_vec(i).map(|b|xc_cache::ContentDigest::sha256(&b))).transpose()?,
            "primary_recomputation":false,"cargo_lock":include_str!("../Cargo.lock")});
        save(&output.join("request.json"), &request)?;
        save(
            &output.join("convergence-observation.json"),
            &sources.observation()?,
        )?;
        let record = xc_cache::capture_and_persist(
            &request,
            diagnostics.to_vec(),
            |id| {
                sources
                    .capture(id, input.as_ref(), &research_sources, &cache)
                    .and_then(|r| Ok(xc_cache::CapturedDiagnostic::from_cached(r)?))
                    .map_err(CaptureFailure::failed)
            },
            &cache,
        )?;
        save(&output.join("capture.json"), &record.value)?;
        save_reviewed_values(&output.join("capture-values.json"), &record.value, &cache)?;
        let complete = record.value.receipt.is_complete();
        save(
            &output.join("status.json"),
            &serde_json::json!({"capture_complete":complete,"primary_recomputed":false,
            "numerical_coverage":record.value.numerical_coverage,"assurance":"supplemental finite analysis; original claim verdict unchanged"}),
        )?;
        managed.finalize_publication_inventory()?;
        println!("Supplemental journal: {}", output.display());
        anyhow::ensure!(
            complete,
            "supplemental acquisition incomplete; retained receipts explain each failure"
        );
        Ok(())
    }

    fn local_replay_session() -> Result<xc_cache::ManagedArtifactCacheSession> {
        let mut config = xc_cache::ManagedArtifactCacheConfig::from_environment()?
            .context("managed retained cache required")?;
        anyhow::ensure!(
            config.publication_target == xc_core::PublicationTarget::None
                && !config.execute_remote_mutations && !config.replace_existing_publication,
            "supplemental replay refuses publication; clear XC_PUBLISH_TARGET, XC_PUBLISH_EXECUTE and XC_PUBLISH_REPLACE"
        );
        // An inherited author staging queue must not be finalized by replay.
        // Supplemental writes belong only to the local analysis cache.
        config.staging_root = None;
        config.remote_cache_mode = xc_cache::ManagedRemoteCacheMode::None;
        config.output_validation = None;
        config.cache_mode = xc_cache::ArtifactExecutionCacheMode::PreferReuse;
        xc_cache::ManagedArtifactCacheSession::new(config).map_err(Into::into)
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

    fn research_input_inventory() -> serde_json::Value {
        let mut inputs = serde_json::Map::new();
        inputs.insert("runtime_target_reference_preparation".into(), serde_json::json!({
            "enabled": std::env::var("XC_RESEARCH_PREPARE_TARGET_REFERENCE").is_ok_and(|v| v == "1"),
            "scope": "runtime target samples and finite Fourier jets; run-derived weighted atoms, arithmetic tail model, polynomial band and interval block bounds; finite retained-source scope"
        }));
        for name in [
            "XC_RESEARCH_REFERENCE_FILE",
            "XC_RESEARCH_INPUTS_FILE",
            "XC_TARGET_SPEC_FILE",
        ] {
            let value = match std::env::var_os(name) {
                None => serde_json::json!({"status":"not_supplied"}),
                Some(path) if fs::metadata(&path).is_ok_and(|m| m.len() > 64 * 1024 * 1024) => {
                    serde_json::json!({"status":"exceeds_64_mib_input_limit"})
                }
                Some(path) => match fs::read(Path::new(&path)) {
                    Ok(bytes) => {
                        serde_json::json!({"status":"supplied", "bytes":bytes.len(), "sha256":xc_cache::ContentDigest::sha256(&bytes)})
                    }
                    Err(_) => serde_json::json!({"status":"unreadable"}),
                },
            };
            inputs.insert(name.into(), value);
        }
        inputs.into()
    }

    // An explicitly configured private adapter runs only after primary sources
    // exist. The executable and recipe stay outside the public paper repository.
    // Its output is data and must pass the Toolkit's authenticated source join.
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct InputPreparer {
        argv: Vec<String>,
        timeout_seconds: u64,
    }

    #[derive(Debug)]
    struct PreparedInputs {
        input: xc_spectral::ccm::extended_research::ExternalResearchInputs,
        digest: xc_cache::ContentDigest,
    }

    struct PreparerProcess(std::process::Child);
    impl Drop for PreparerProcess {
        fn drop(&mut self) {
            // Every descendant that retains the child's process group is
            // stopped on success, failure and timeout, before reading output.
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("/bin/kill")
                    .args(["-KILL", "--", &format!("-{}", self.0.id())])
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
            }
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn invoke_preparer(
        directory: &Path,
        request: &serde_json::Value,
        configuration: &str,
    ) -> Result<PreparedInputs> {
        use std::process::{Command, Stdio};
        anyhow::ensure!(
            cfg!(unix),
            "private input preparation requires Unix process-group supervision"
        );
        #[cfg(unix)]
        anyhow::ensure!(
            Path::new("/bin/kill").is_file(),
            "process-group cleanup executable unavailable"
        );
        let preparer: InputPreparer = serde_json::from_str(configuration)?;
        anyhow::ensure!(
            !preparer.argv.is_empty()
                && preparer.argv.len() <= 16
                && preparer
                    .argv
                    .iter()
                    .all(|a| !a.is_empty() && a.len() <= 32768)
                && (1..=7200).contains(&preparer.timeout_seconds),
            "invalid private input preparation command or time limit"
        );
        let directory = directory.canonicalize()?;
        let request_path = directory.join("input-preparation-request.json");
        let output_path = directory.join("prepared-research-inputs.json");
        save(&request_path, request)?;
        let stdout = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("input-preparation.stdout.log"))?;
        let stderr = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("input-preparation.stderr.log"))?;
        let mut command = Command::new(&preparer.argv[0]);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child = PreparerProcess(
            command
                .args(&preparer.argv[1..])
                .arg("--request")
                .arg(&request_path)
                .arg("--output")
                .arg(&output_path)
                .stdin(Stdio::null())
                .stdout(stdout)
                .stderr(stderr)
                .spawn()
                .context("starting private input preparer")?,
        );
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.0.try_wait()? {
                break status;
            }
            let excessive = [
                "input-preparation.stdout.log",
                "input-preparation.stderr.log",
                "prepared-research-inputs.json",
            ]
            .iter()
            .any(|name| {
                fs::metadata(directory.join(name)).is_ok_and(|m| {
                    m.len() > xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES
                })
            });
            if excessive {
                anyhow::bail!("private input preparer exceeded its output budget");
            }
            if started.elapsed().as_secs() >= preparer.timeout_seconds {
                anyhow::bail!("private input preparation exceeded its time limit");
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        drop(child);
        anyhow::ensure!(
            status.success(),
            "private input preparer exited with {status}"
        );
        anyhow::ensure!(
            [
                "input-preparation.stdout.log",
                "input-preparation.stderr.log"
            ]
            .iter()
            .all(|name| fs::metadata(directory.join(name))
                .is_ok_and(|m| m.len() <= 64 * 1024 * 1024)),
            "private input preparer log exceeds 64 MiB"
        );
        anyhow::ensure!(
            fs::metadata(&output_path)?.len()
                <= xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES,
            "private prepared input exceeds the research input byte limit"
        );
        // Read once: validation, loading and the recorded digest use identical
        // admitted bytes. A changed retained file will fail summary verification.
        use std::io::Read;
        let mut bytes = Vec::new();
        fs::File::open(&output_path)?
            .take(xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES + 1)
            .read_to_end(&mut bytes)?;
        let input = xc_spectral::ccm::extended_research::ExternalResearchInputs::from_bytes(
            &output_path,
            &bytes,
        )?;
        Ok(PreparedInputs {
            input,
            digest: xc_cache::ContentDigest::sha256(&bytes),
        })
    }

    fn prepare_after_acquisition(
        run: &mut RetainedCcmRun,
        directory: &Path,
        resolved: &serde_json::Value,
        configuration: &str,
        cache: &ArtifactCacheContext<'_>,
    ) -> Result<serde_json::Value> {
        let base = run.prepare_extended_research_inputs()?;
        let request = serde_json::json!({"schema_version":1,
            "configuration":resolved,"primary_sources":run.primary_sources(),
            "base_inputs":base,"target_spec_file":std::env::var("XC_TARGET_SPEC_FILE").ok(),
            "output_visibility":"private_only","primary_recomputation":false});
        let prepared = invoke_preparer(directory, &request, configuration)?;
        run.set_extended_research_inputs(prepared.input)?;
        let component_status = match run.prepare_retained_trial_components(
            320,
            xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES,
            cache,
        ) {
            Ok(true) => "retained_independent_components_prepared",
            Ok(false) => "resource_blocked",
            Err(error) => {
                save(
                    &directory.join("component-preparation-status.json"),
                    &serde_json::json!({"status":"unavailable","reason":format!("{error:#}")}),
                )?;
                "unavailable"
            }
        };
        Ok(
            serde_json::json!({"status":"prepared","phase":"after_primary_acquisition",
            "input_sha256":prepared.digest,
            "command_configuration_sha256":xc_cache::ContentDigest::sha256(configuration.as_bytes()),
            "source_join_validated":true,"primary_recomputed":false,
            "component_preparation":component_status,
            "scope":"private prepared inputs; scientific qualification remains in each diagnostic"}),
        )
    }

    /// Validate the same recipe and fixed applicability contract without a journal.
    pub fn validate_configuration(
        params: &CcmParams,
        cfg: &HighPrecConfig,
        level: ResearchCapture,
        count: usize,
        args: &CaptureJournalArgs,
    ) -> Result<()> {
        let plan = recipe(level, count, params.n_modes + 1, args, cfg.precision_bits)?;
        let excluded = exclusions(args.capture_policy, level, params, cfg, args)?;
        requested_diagnostics(&plan, level == ResearchCapture::Ultra, &excluded)?;
        Ok(())
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
        let resolved = serde_json::json!({"schema_version":1,"paper_version":env!("CARGO_PKG_VERSION"),"toolkit_release":"0.16.0","arb_enabled":cfg!(feature="root-certification"),"research_inputs":research_input_inventory(),"capture":plan,"applicability":applicability,"retained_reduction":reduction,"lambda_squared":params.lambda_sq_int(),"n_modes":params.n_modes,"precision_bits":cfg.precision_bits,"root_precision_policy":cfg.root_precision_policy,"root_maximum_extra_precision_bits":cfg.root_maximum_extra_precision_bits,"root_verification_precision_bits":cfg.root_verification_precision_bits,"parity_policy":cfg.effective_parity_policy().as_str(),"distance":options.distance_capture.as_ref().map(|d|serde_json::json!({"alpha":d.alpha,"rules":d.rules.iter().map(|r|serde_json::json!({"family":r.family(),"rule":r.rule(),"variable":r.variable().as_str(),"resolution":r.resolution()})).collect::<Vec<_>>(),"profile_steps":d.profile_steps})),"source_policy":"resolve compatible identities; preserve historical artifacts","assurance":"capture completeness is separate from numerical acceptance"});
        let mut resolved = resolved;
        resolved["assembly_policy"] = serde_json::json!({
            "recipe":"paper_default_mode_length_precision_quadrature",
            "quadrature_base_points":cfg.quad_points,
            "effective_orders":"mode, cutoff and precision dependent; base points are a floor"});
        resolved["convergence_dependency_policy"] = serde_json::json!({
            "semantics":"retained_then_cohort_v1","single_run":"derive after primary acquisition",
            "comparisons":"after compatible independently computed configurations exist",
            "analytic_bounds":"open obligations remain explicit; never inferred from refinement differences"});
        resolved["private_input_preparation"] = serde_json::json!({
            "semantics":"retained_private_input_preparation_v1",
            "configured":std::env::var_os("XC_RESEARCH_PREPARER").is_some(),
            "phase":"after_primary_acquisition","visibility":"private_only"});
        save(&directory.join("request.json"), &resolved)?;
        // Preserve the exact dependency lockfile with each run, including the
        // qualified Git commit after a release tag has been amended.
        save(
            &directory.join("build.json"),
            &serde_json::json!({"cargo_lock":include_str!("../Cargo.lock"),
                "toolkit_working_source_digest":option_env!("XC_TOOLKIT_SOURCE_DIGEST"),
                "local_qualification":option_env!("XC_TOOLKIT_SOURCE_DIGEST").is_some()}),
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
        // Only requested diagnostics may be computed ahead of their turn.
        run.set_lookahead_requests(requested.iter().cloned());
        save(
            &directory.join("primary.json"),
            &PortableHighPrecResult::from_runtime(run.primary())?,
        )?;
        save(
            &directory.join("primary-sources.json"),
            &run.primary_sources(),
        )?;
        // Source-bound observations are outputs of primary acquisition. They
        // never require a second dimension or a future run to start this one.
        let observation = xc_spectral::ccm::convergence_capture::finite_capture::RetainedFiniteSources::from_manifests(
            &run.primary_sources(), false, &cache,
        ).and_then(|s| s.observation());
        let observation_error = match observation {
            Ok(value) => {
                save(&directory.join("convergence-observation.json"), &value)?;
                None
            }
            Err(error) => Some(format!("retained convergence observation: {error:#}")),
        };
        event(
            &mut events,
            &serde_json::json!({"phase":"primary","status":"saved","elapsed_seconds":started.elapsed().as_secs_f64()}),
        )?;
        let preparation_error = if level == ResearchCapture::Ultra {
            match std::env::var("XC_RESEARCH_PREPARER") {
                Ok(configuration) => {
                    let prepared = prepare_after_acquisition(
                        &mut run,
                        &directory,
                        &resolved,
                        &configuration,
                        &cache,
                    );
                    let status = match &prepared {
                        Ok(value) => value.clone(),
                        Err(error) => {
                            serde_json::json!({"status":"failed","phase":"after_primary_acquisition",
                            "reason":format!("{error:#}"),"primary_preserved":true})
                        }
                    };
                    save(&directory.join("input-preparation-status.json"), &status)?;
                    event(&mut events, &status)?;
                    prepared.err().map(|e| format!("{e:#}"))
                }
                Err(std::env::VarError::NotPresent) => None,
                Err(error) => Some(error.to_string()),
            }
        } else {
            None
        };
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
                run.capture_diagnostic_outcome(id, options, &cache)
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
        save_reviewed_values(
            &directory.join("capture-values.json"),
            &record.value,
            &cache,
        )?;
        let mut complete = record.value.receipt.is_complete()
            && observation_error.is_none()
            && preparation_error.is_none();
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
            save_reviewed_values(
                &directory.join("capture-explicit-values.json"),
                &extra_record.value,
                &cache,
            )?;
        }
        // Preserve the final source-bound inputs, including jets and actions
        // prepared during capture, so supplemental analysis can reuse them.
        if level == ResearchCapture::Ultra {
            match run.prepare_extended_research_inputs() {
                Ok(Some(input)) => save(&directory.join("external-research-inputs.json"), &input)?,
                Ok(None) => {}
                Err(error) => save(
                    &directory.join("external-research-inputs-status.json"),
                    &serde_json::json!({"status":"failed","reason":format!("{error:#}")}),
                )?,
            }
        }
        if level == ResearchCapture::Ultra {
            save(
                &directory.join("research-sources.json"),
                &run.extended_research_sources(),
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
            &serde_json::json!({"status":if complete{"capture_complete"}else{"capture_incomplete"},"primary_saved":true,"capture_complete":complete,"applicability":applicability,"convergence_observation_error":observation_error,"input_preparation_error":preparation_error,"numerical_acceptance":"inspect primary and measurement payloads; completion is not validity","elapsed_seconds":started.elapsed().as_secs_f64()}),
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

        #[cfg(unix)]
        #[test]
        fn private_preparer_checks_exit_timeout_size_and_schema() {
            let root = std::env::temp_dir().join(format!(
                "paper-preparer-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&root).unwrap();
            let valid = serde_json::json!({"schema_version":1,"source_eigenpair":"b".repeat(64),
                "lambda_squared":"5","n_modes":2,"precision_bits":128,
                "convention_id":"synthetic transport","definition_digest":"a".repeat(64),"approximation_scope":"synthetic transport"});
            save(&root.join("valid.json"), &valid).unwrap();
            for (index, script, passes, reason) in [
                (0, "cp ../valid.json \"$4\"", true, ""),
                (1, "exit 7", false, "exited"),
                (2, "exec sleep 2", false, "time limit"),
                (3, "truncate -s 67108865 \"$4\"", false, "64 MiB"),
                (4, "printf invalid > \"$4\"", false, ""),
                (5, "printf '{}' > \"$4\"", false, ""),
                (6, "(sleep 2; touch late-write) & wait", false, "time limit"),
                (
                    7,
                    "cp ../valid.json \"$4\"; (sleep 2; touch late-write) & exit 0",
                    true,
                    "",
                ),
            ] {
                let directory = root.join(index.to_string());
                fs::create_dir(&directory).unwrap();
                // Use an explicit cwd inside the shell test only. The real
                // adapter receives absolute request/output paths and no shell.
                let script = format!("cd -- \"$(dirname -- \"$4\")\"; {script}");
                let configuration=serde_json::json!({"argv":["/bin/sh","-c",script,"fixture"],"timeout_seconds":1}).to_string();
                let result = invoke_preparer(
                    &directory,
                    &serde_json::json!({"primary_saved":true}),
                    &configuration,
                );
                assert_eq!(result.is_ok(), passes, "case {index}: {result:?}");
                if !passes && !reason.is_empty() {
                    assert!(format!("{:#}", result.as_ref().unwrap_err()).contains(reason));
                }
                assert!(directory.join("input-preparation-request.json").exists());
                if let Ok(prepared) = result {
                    let bytes = fs::read(directory.join("prepared-research-inputs.json")).unwrap();
                    assert_eq!(prepared.digest, xc_cache::ContentDigest::sha256(&bytes));
                    fs::write(
                        directory.join("prepared-research-inputs.json"),
                        b"changed after read",
                    )
                    .unwrap();
                    assert_eq!(prepared.input.source_eigenpair.0, "b".repeat(64));
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(2200));
            assert!(!root.join("6/late-write").exists());
            assert!(!root.join("7/late-write").exists());
            fs::remove_dir_all(root).unwrap();
        }

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
        fn ultra_exports_every_referenced_finite_measurement_and_rejects_lost_values() {
            use xc_cache::*;
            use xc_spectral::ccm::capture::FINITE_DIAGNOSTICS;
            let store = EphemeralCacheStore::new(8 << 20).unwrap();
            let plan = CcmCapturePlan::ultra(8, 121).unwrap();
            let requested = requested_diagnostics(&plan, true, &Default::default()).unwrap();
            assert_eq!(requested.len(), 52);
            assert!(requested.contains(&"trial_vector_energy".into()));
            let mut expected = serde_json::Map::new();
            let mut artifacts = std::collections::BTreeMap::new();
            for &id in FINITE_DIAGNOSTICS {
                assert!(requested.contains(&id.to_string()));
                // Distinct exact values expose missing or cross-wired exports.
                // These are transport fixtures, not physical trial vectors.
                let value = serde_json::json!({"kind":"ccm_finite_diagnostic_analysis",
                    "data":{"diagnostic":id,"outcome":"computed","rows":[{
                        "label":id,"outcome":"point_measurement"}],
                        "result":{"exact_fixture":"1/3","identity":id}}});
                let draft = ArtifactDraft {
                    schema_version: 1,
                    key: ArtifactKey::new("ccm_finite_diagnostic_analysis", id, id.as_bytes())
                        .unwrap(),
                    producer_toolkit_version: ToolkitVersion::parse("0.16.0").unwrap(),
                    minimum_reader_version: ToolkitVersion::parse("0.16.0").unwrap(),
                    maximum_reader_version: None,
                    quality: CacheQuality::Validated,
                    visibility: CacheVisibility::Local,
                    immutable: true,
                    dependencies: vec![],
                    tags: Default::default(),
                    provenance_digest: None,
                };
                assert!(!artifact_kind_admitted_to_destination(
                    &draft.key.kind,
                    PublicationDestination::Public
                ));
                assert!(artifact_kind_admitted_to_destination(
                    &draft.key.kind,
                    PublicationDestination::Private
                ));
                artifacts.insert(
                    id,
                    store
                        .put(&draft, &serde_json::to_vec(&value).unwrap())
                        .unwrap(),
                );
                expected.insert(id.into(), value);
            }
            let mut record = collect_capture(
                &plan,
                FINITE_DIAGNOSTICS.iter().map(|s| s.to_string()).collect(),
                |id| {
                    CapturedDiagnostic::by_reference(
                        &expected[id],
                        vec![artifacts[id].clone()],
                        false,
                        vec![],
                    )
                    .map_err(CaptureFailure::failed)
                },
            )
            .unwrap();
            assert!(record.measurements.values().all(|m| m.value.is_null()));
            let resolver = CacheResolver::new(vec![CacheLayer {
                precedence: 0,
                store: Box::new(store),
            }]);
            let policy = CachePolicy {
                current_toolkit_version: ToolkitVersion::parse("0.16.0").unwrap(),
                minimum_quality: CacheQuality::Validated,
                accepted_schema_versions: vec![1],
                allow_deprecated: false,
                allow_quarantined: false,
                allowed_visibilities: vec![CacheVisibility::Local],
            };
            let mut cache = ArtifactCacheContext {
                resolver: Some(&resolver),
                reference_resolver: None,
                acceptance: Some(&policy),
                ordered_overlays: vec![],
                mode: ArtifactExecutionCacheMode::RequireReuse,
                write_on_miss: false,
                write_visibility: CacheVisibility::Local,
                requested_assurance: xc_core::AssuranceLevel::Computed,
                certification_failure_policy: CertificationFailurePolicy::RetainComputedFailRun,
                production_sink: None,
            };
            let path = std::env::temp_dir().join(format!(
                "paper-capture-export-{}-{}.json",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            save_reviewed_values(&path, &record, &cache).unwrap();
            let saved: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            assert_eq!(saved, serde_json::Value::Object(expected));
            fs::remove_file(&path).unwrap();
            cache.resolver = None;
            assert!(save_reviewed_values(&path, &record, &cache).is_err());
            assert!(!path.exists());
            cache.resolver = Some(&resolver);
            record
                .measurements
                .get_mut("trial_vector_energy")
                .unwrap()
                .value_reference
                .as_mut()
                .unwrap()
                .value_digest = "0".repeat(64);
            assert!(save_reviewed_values(&path, &record, &cache).is_err());
            assert!(!path.exists());
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
            let full = requested_diagnostics(&plan, true, &Default::default()).unwrap();
            // Historical v7 groups plus the current finite diagnostics; the
            // same eight predeclared exclusions remain unchanged.
            assert_eq!(excluded.len(), 8);
            assert_eq!(
                requested.len(),
                33 + xc_spectral::ccm::capture::FINITE_DIAGNOSTICS.len()
            );
            assert_eq!(
                requested,
                full.into_iter()
                    .filter(|id| !excluded.contains_key(id))
                    .collect::<Vec<_>>()
            );
            assert!(requested.contains(&"transform_enclosure".into()));
            assert!(requested.contains(&"operator_energy".into()));
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
                assert_eq!(record.receipt.outcomes().len(), requested.len());
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
                41 + xc_spectral::ccm::capture::FINITE_DIAGNOSTICS.len()
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
                assert_eq!(
                    requested.len(),
                    38 + xc_spectral::ccm::capture::FINITE_DIAGNOSTICS.len()
                );
                for added in ["assembly_error", "checkpoint_spectra", "target_comparison"] {
                    assert!(requested.contains(&added.into()), "{added}");
                }
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
                    assert_eq!(record.receipt.outcomes().len(), requested.len());
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
