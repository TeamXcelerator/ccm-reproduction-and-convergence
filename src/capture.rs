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
    /// Retain publication staging for one explicit publish-staged call after a campaign.
    #[arg(long, global = true)]
    pub defer_publication: bool,
}

#[cfg(feature = "hp")]
pub use hp::{
    execute, ground_index_policy, measure_evenness, publish_staged, recover_capture_with_mode,
    replay_finite, validate_configuration, write_evenness, write_measurements,
};

#[cfg(feature = "hp")]
mod hp {
    use super::*;
    mod recovery {
        include!("capture_recovery.rs");
    }
    pub use recovery::recover_capture_with_mode;
    /// Ground-index admission of the primary state under a capture policy.
    ///
    /// The lowest stored sector eigenvalues of the frozen Claim 1c configuration
    /// are unresolved at HP-1000, so its policy
    /// retains the selected state and records the ground-index proof's outcome.
    /// Every other policy requires the proof.
    pub fn ground_index_policy(
        policy: CapturePolicy,
    ) -> xc_spectral::ccm::hp::CcmGroundIndexPolicy {
        match policy {
            CapturePolicy::Claim1cHp1000 => {
                xc_spectral::ccm::hp::CcmGroundIndexPolicy::RecordOutcome
            }
            CapturePolicy::Full | CapturePolicy::Claim8Natural => {
                xc_spectral::ccm::hp::CcmGroundIndexPolicy::Require
            }
        }
    }

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

    fn validate_deferred_publication(config: &xc_cache::ManagedArtifactCacheConfig) -> Result<()> {
        anyhow::ensure!(
            config.profile == xc_cache::ManagedRunProfile::Author
                && config.execute_remote_mutations
                && config.publication_target != xc_core::PublicationTarget::None
                && config.staging_root.is_some()
                && config.output_validation.is_none(),
            "deferred publication requires an explicit author staging root, execution, destination and no output validation"
        );
        Ok(())
    }

    /// Preserve the natural-space measurement before a campaign publishes its
    /// retained drafts. The ordinary Toolkit entry point finalizes immediately.
    pub fn measure_evenness(
        params: &CcmParams,
        cfg: &HighPrecConfig,
        args: &CaptureJournalArgs,
    ) -> Result<xc_spectral::ccm::hp::EvennessResult> {
        if !args.defer_publication {
            return xc_spectral::ccm::hp::measure_evenness(params, cfg);
        }
        let config = xc_cache::ManagedArtifactCacheConfig::from_environment()?
            .context("deferred evenness requires explicit managed cache configuration")?;
        validate_deferred_publication(&config)?;
        let managed = xc_cache::ManagedArtifactCacheSession::new_with_resources(
            config,
            xc_cache::managed_resource_policy_from_environment()?,
        )?;
        xc_spectral::ccm::hp::measure_evenness_via_cache(params, cfg, &managed.context())
    }

    /// Publish only authenticated retained drafts; no numerical callback exists.
    pub fn publish_staged(staging_root: &Path) -> Result<()> {
        let config = xc_cache::ManagedArtifactCacheConfig::from_environment()?
            .context("publish-staged requires explicit managed cache configuration")?;
        validate_deferred_publication(&config)?;
        anyhow::ensure!(
            fs::symlink_metadata(staging_root)?.file_type().is_dir()
                && config
                    .staging_root
                    .as_ref()
                    .context("staging root missing")?
                    .canonicalize()?
                    == staging_root.canonicalize()?,
            "publish-staged requires the exact configured ordinary staging directory"
        );
        let managed = xc_cache::ManagedArtifactCacheSession::new_with_resources(
            config,
            xc_cache::managed_resource_policy_from_environment()?,
        )?;
        let inventory = managed
            .retry_retained_publication()?
            .context("publish-staged did not finalize a retained inventory")?;
        println!("Staged publication finalized: {}", inventory.display());
        Ok(())
    }

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
        for id in [
            "sector_analysis",
            "evenness",
            "sector_gap_certificate",
            "finite_root_budget",
        ] {
            result.insert(id.into(), "at HP-1000, 35 even-sector eigenvalues lie within the working-precision floor of the selected value, so no sector eigenvector can be identified; sector analysis, natural evenness, the sector-gap certificate and the root budget that replays it cannot be formed".into());
        }
        Ok(result)
    }

    /// Exclusions that name an explicitly requested certificate rather than a
    /// planned diagnostic.
    const EXPLICIT_EXCLUSIONS: [&str; 1] = ["sector_gap_certificate"];

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
            excluded
                .keys()
                .all(|id| requested.contains(id) || EXPLICIT_EXCLUSIONS.contains(&id.as_str())),
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
        let _timing = xc_core::performance_stage_with("paper.capture.export", || {
            xc_core::PerformanceStageMetadata::default()
        });
        // Only references enter this export. Sort names exactly as serde_json's
        // map did, while retaining at most one decoded measurement at a time.
        let mut values = std::collections::BTreeMap::new();
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
            values.insert(name, measurement);
        }
        // A failed integrity check must leave no partial final export. A hard
        // link installs the completed file without replacing an existing path.
        let temporary = path.with_extension("json.export-partial");
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let pending_file = options.open(&temporary)?;
        struct RemoveTemporary(PathBuf);
        impl Drop for RemoveTemporary {
            fn drop(&mut self) {
                let _ = fs::remove_file(&self.0);
            }
        }
        let _cleanup = RemoveTemporary(temporary.clone());
        let mut file = std::io::BufWriter::new(pending_file);
        struct Indented<'a>(&'a mut std::io::BufWriter<std::fs::File>);
        impl Write for Indented<'_> {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                for part in bytes.split_inclusive(|byte| *byte == b'\n') {
                    self.0.write_all(part)?;
                    if part.last() == Some(&b'\n') {
                        self.0.write_all(b"  ")?;
                    }
                }
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                self.0.flush()
            }
        }
        if values.is_empty() {
            file.write_all(b"{}\n")?;
        } else {
            file.write_all(b"{\n")?;
            for (index, (name, measurement)) in values.into_iter().enumerate() {
                if index != 0 {
                    file.write_all(b",\n")?;
                }
                file.write_all(b"  ")?;
                serde_json::to_writer(&mut file, name)?;
                file.write_all(b": ")?;
                let resolver = cache
                    .resolver
                    .context("capture export requires a cache resolver")?;
                let policy = cache
                    .acceptance
                    .context("capture export requires a cache acceptance policy")?;
                let mut writer = Indented(&mut file);
                let result =
                    xc_cache::measurement_value(measurement, resolver, policy).and_then(|value| {
                        serde_json::to_writer_pretty(&mut writer, &value)?;
                        Ok(())
                    });
                result.with_context(|| format!("{name}: referenced capture value not resolved"))?;
            }
            file.write_all(b"\n}\n")?;
        }
        file.flush()?;
        file.get_ref().sync_all()?;
        drop(file);
        fs::hard_link(&temporary, path)?;
        Ok(())
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
        xc_cache::ManagedArtifactCacheSession::new_with_resources(
            config,
            xc_cache::managed_resource_policy_from_environment()?,
        )
        .map_err(Into::into)
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

    // Operational telemetry deliberately stays out of prepared inputs, source
    // identities and capture plans. Never echo adapter stdout/stderr or argv.
    struct PreparationProgress {
        started: Instant,
        last_heartbeat: Instant,
        file: fs::File,
        directory: PathBuf,
    }

    impl PreparationProgress {
        fn new(directory: &Path) -> Result<Self> {
            Ok(Self {
                started: Instant::now(),
                last_heartbeat: Instant::now(),
                file: OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(directory.join("input-preparation-progress.jsonl"))?,
                directory: directory.to_path_buf(),
            })
        }

        fn emit(&mut self, stage: &'static str) -> Result<()> {
            self.emit_timed(stage, None)
        }

        fn emit_timed(
            &mut self,
            stage: &'static str,
            validation_seconds: Option<f64>,
        ) -> Result<()> {
            let bytes = |name| {
                fs::metadata(self.directory.join(name))
                    .map(|m| m.len())
                    .unwrap_or(0)
            };
            let event = serde_json::json!({"schema_version":1,"stage":stage,
                "elapsed_seconds":self.started.elapsed().as_secs_f64(),
                "validation_seconds":validation_seconds,
                "stdout_bytes":bytes("input-preparation.stdout.log"),
                "stderr_bytes":bytes("input-preparation.stderr.log"),
                "output_bytes":bytes("prepared-research-inputs.json"),
                "numerical_progress_verified":false});
            serde_json::to_writer(&mut self.file, &event)?;
            self.file.write_all(b"\n")?;
            self.file.sync_data()?;
            self.last_heartbeat = Instant::now();
            xc_core::progress_message!("private preparation: stage={stage}, elapsed={:.1}s; retained logs: input-preparation.stdout.log, input-preparation.stderr.log (private preparation directory); byte counters are operational only", self.started.elapsed().as_secs_f64());
            Ok(())
        }
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
        let mut progress = PreparationProgress::new(&directory)?;
        progress.emit("request_admitted")?;
        // The location is driver-owned and screened independently from adapter
        // output; secret-bearing paths are never echoed to the terminal.
        let location = directory.display().to_string();
        if xc_core::validate_secret_free(&location, "preparation log location").is_ok() {
            xc_core::progress_message!("private preparation log directory: {location}");
        }
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
        progress.emit("process_started")?;
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
                progress.emit("output_budget_exceeded")?;
                anyhow::bail!("private input preparer exceeded its output budget");
            }
            if started.elapsed().as_secs() >= preparer.timeout_seconds {
                progress.emit("timeout")?;
                anyhow::bail!("private input preparation exceeded its time limit");
            }
            if progress.last_heartbeat.elapsed() >= std::time::Duration::from_secs(30) {
                progress.emit("process_running")?;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        drop(child);
        progress.emit(if status.success() {
            "process_exited_successfully"
        } else {
            "process_exited_unsuccessfully"
        })?;
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
        let validation_started = Instant::now();
        progress.emit("validating_prepared_input")?;
        fs::File::open(&output_path)?
            .take(xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES + 1)
            .read_to_end(&mut bytes)?;
        let input = xc_spectral::ccm::extended_research::ExternalResearchInputs::from_bytes(
            &output_path,
            &bytes,
        );
        progress.emit_timed(
            if input.is_ok() {
                "prepared_input_validated"
            } else {
                "prepared_input_validation_failed"
            },
            Some(validation_started.elapsed().as_secs_f64()),
        )?;
        let input = input?;
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
        even_state_isolation: bool,
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
        let excluded = exclusions(args.capture_policy, level, params, cfg, args)?;
        // An excluded sector analysis is not formed for other diagnostics
        // either; they record its absence as an unavailable input.
        if excluded.contains_key("sector_analysis") {
            options.sector_analysis = None;
            options.capture_evenness = false;
        }
        if excluded.contains_key("sector_gap_certificate") {
            options.sector_gap_certification = None;
        }
        let options = &options;
        let plan = recipe(level, count, params.n_modes + 1, args, cfg.precision_bits)?;
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
        let resolved = serde_json::json!({"schema_version":1,"paper_version":env!("CARGO_PKG_VERSION"),"toolkit_release":"0.17.0","arb_enabled":cfg!(feature="root-certification"),"research_inputs":research_input_inventory(),"capture":plan,"applicability":applicability,"retained_reduction":reduction,"lambda_squared":params.lambda_sq_int(),"n_modes":params.n_modes,"precision_bits":cfg.precision_bits,"root_precision_policy":cfg.root_precision_policy,"root_maximum_extra_precision_bits":cfg.root_maximum_extra_precision_bits,"root_verification_precision_bits":cfg.root_verification_precision_bits,"parity_policy":cfg.effective_parity_policy().as_str(),"distance":options.distance_capture.as_ref().map(|d|serde_json::json!({"alpha":d.alpha,"rules":d.rules.iter().map(|r|serde_json::json!({"family":r.family(),"rule":r.rule(),"variable":r.variable().as_str(),"resolution":r.resolution()})).collect::<Vec<_>>(),"profile_steps":d.profile_steps})),"source_policy":"resolve compatible identities; preserve historical artifacts","assurance":"capture completeness is separate from numerical acceptance"});
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
        let managed_config = xc_cache::ManagedArtifactCacheConfig::from_environment()?
            .context("managed cache required")?;
        if args.defer_publication {
            validate_deferred_publication(&managed_config)?;
        }
        let managed = xc_cache::ManagedArtifactCacheSession::new_with_resources(
            managed_config,
            xc_cache::managed_resource_policy_from_environment()?,
        )?;
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
        // Evenness claims also retain the even-state spectral isolation that
        // the paper's separations table reports. It is claim data, so a
        // failure fails the claim once the journal is saved.
        let isolation_error = if even_state_isolation {
            match run.even_state_spectral_isolation(&cache) {
                Ok(isolation) => {
                    save(&directory.join("spectral-isolation.json"), &isolation)?;
                    None
                }
                Err(error) => {
                    let reason = format!("{error:#}");
                    save(
                        &directory.join("spectral-isolation-status.json"),
                        &serde_json::json!({"status":"failed","reason":reason}),
                    )?;
                    Some(reason)
                }
            }
        } else {
            None
        };
        complete &= isolation_error.is_none();
        if args.defer_publication {
            println!("  publication deferred: retained staging awaits publish-staged");
        } else {
            managed.finalize_publication_inventory()?;
        }
        save(
            &directory.join("status.json"),
            &serde_json::json!({"status":if complete{"capture_complete"}else{"capture_incomplete"},"primary_saved":true,"capture_complete":complete,"publication_status":if args.defer_publication{"deferred"}else{"not_deferred"},"applicability":applicability,"convergence_observation_error":observation_error,"input_preparation_error":preparation_error,"numerical_acceptance":"inspect primary and measurement payloads; completion is not validity","elapsed_seconds":started.elapsed().as_secs_f64()}),
        )?;
        println!(
            "  capture journal saved: {} ({})",
            directory.display(),
            if args.defer_publication {
                if complete {
                    "capture complete; publication pending"
                } else {
                    "capture incomplete; publication pending; see receipt reasons"
                }
            } else if complete {
                "complete"
            } else {
                "incomplete; see receipt reasons"
            }
        );
        if let Some(reason) = isolation_error {
            anyhow::bail!(
                "even-state spectral isolation failed; primary and receipts preserved in {}: {reason}",
                directory.display()
            );
        }
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
                (
                    3,
                    "truncate -s 4294967297 \"$4\"",
                    false,
                    "research input byte limit",
                ),
                (4, "printf invalid > \"$4\"", false, ""),
                (5, "printf '{}' > \"$4\"", false, ""),
                (6, "(sleep 2; touch late-write) & wait", false, "time limit"),
                (
                    7,
                    "cp ../valid.json \"$4\"; (sleep 2; touch late-write) & exit 0",
                    true,
                    "",
                ),
                (
                    8,
                    "printf 'arbitrary-private-output' >&2; cp ../valid.json \"$4\"",
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
                    let error = format!("{:#}", result.as_ref().unwrap_err());
                    // The size guard can run before child exit or during the
                    // final output check. Both paths must refuse the payload.
                    let expected = error.contains(reason)
                        || (index == 3
                            && error.contains("private input preparer exceeded its output budget"));
                    assert!(expected, "case {index}: {error}");
                }
                assert!(directory.join("input-preparation-request.json").exists());
                let progress =
                    fs::read_to_string(directory.join("input-preparation-progress.jsonl")).unwrap();
                assert!(!progress.contains("arbitrary-private-output"));
                let stages: Vec<serde_json::Value> = progress
                    .lines()
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect();
                assert_eq!(stages[0]["stage"], "request_admitted");
                assert!(stages
                    .iter()
                    .all(|stage| stage["numerical_progress_verified"] == false));
                if let Ok(prepared) = result {
                    assert_eq!(stages.last().unwrap()["stage"], "prepared_input_validated");
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

        #[test]
        fn deferred_publication_preserves_capture_recipe_and_applicability_identity() {
            let ordinary = args();
            let mut deferred = ordinary.clone();
            deferred.defer_publication = true;
            let params = CcmParams::from_lambda_sq_integer(1000, 800);
            let cfg = HighPrecConfig::for_decimal_digits(1000);
            let identity = |args: &CaptureJournalArgs| {
                serde_json::json!({
                "capture":recipe(ResearchCapture::Ultra, 8, 801, args, cfg.precision_bits).unwrap(),
                "applicability":exclusions(args.capture_policy, ResearchCapture::Ultra, &params, &cfg, args).unwrap(),
                "requested":requested_diagnostics(&recipe(ResearchCapture::Ultra, 8, 801, args, cfg.precision_bits).unwrap(), true,
                    &exclusions(args.capture_policy, ResearchCapture::Ultra, &params, &cfg, args).unwrap()).unwrap(),
                "precision_bits":cfg.precision_bits,"lambda_squared":params.lambda_sq_int(),"n_modes":params.n_modes})
            };
            assert_eq!(identity(&ordinary), identity(&deferred));
            let cli = crate::Cli::try_parse_from([
                "ccm-reproduction",
                "publish-staged",
                "--staging-root",
                "retained",
            ])
            .unwrap();
            assert!(matches!(cli.command, crate::Command::PublishStaged { .. }));
            assert!(!cli.capture_journal.defer_publication);
        }

        #[test]
        fn deferred_publication_rejects_ignored_or_read_only_command_paths() {
            for arguments in [
                vec![
                    "ccm-reproduction",
                    "--defer-publication",
                    "run",
                    "--f64-only",
                ],
                vec!["ccm-reproduction", "--defer-publication", "sector-gap"],
                vec![
                    "ccm-reproduction",
                    "--defer-publication",
                    "publish-staged",
                    "--staging-root",
                    "retained",
                ],
                vec![
                    "ccm-reproduction",
                    "--defer-publication",
                    "--preflight-only",
                    "run",
                ],
            ] {
                let cli = crate::Cli::try_parse_from(arguments).unwrap();
                assert!(crate::validate_publication_mode(&cli).is_err());
            }
            for command in ["run", "check-evenness"] {
                let cli = crate::Cli::try_parse_from([
                    "ccm-reproduction",
                    "--defer-publication",
                    command,
                ])
                .unwrap();
                crate::validate_publication_mode(&cli).unwrap();
            }
        }

        #[test]
        fn deferred_publication_requires_explicit_mutating_author_route() {
            use xc_cache::*;
            let valid = ManagedArtifactCacheConfig {
                profile: ManagedRunProfile::Author,
                requested_assurance: xc_core::AssuranceLevel::Computed,
                certification_failure_policy: CertificationFailurePolicy::RetainComputedFailRun,
                cache_root: PathBuf::from("cache"),
                staging_root: Some(PathBuf::from("staging")),
                publication_target: xc_core::PublicationTarget::Private,
                repository_owner: "TeamXcelerator".into(),
                remote_cache_mode: ManagedRemoteCacheMode::None,
                cache_mode: ArtifactExecutionCacheMode::PreferReuse,
                replace_existing_publication: false,
                execute_remote_mutations: true,
                output_validation: None,
            };
            validate_deferred_publication(&valid).unwrap();
            for index in 0..4 {
                let mut rejected = valid.clone();
                match index {
                    0 => rejected.profile = ManagedRunProfile::Normal,
                    1 => rejected.execute_remote_mutations = false,
                    2 => rejected.publication_target = xc_core::PublicationTarget::None,
                    _ => rejected.staging_root = None,
                }
                assert!(validate_deferred_publication(&rejected).is_err());
            }
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
            let fallback_bytes = fs::read(&path).unwrap();
            let saved: serde_json::Value = serde_json::from_slice(&fallback_bytes).unwrap();
            assert_eq!(saved, serde_json::Value::Object(expected.clone()));
            let mut legacy_bytes = serde_json::to_vec_pretty(&expected).unwrap();
            legacy_bytes.push(b'\n');
            assert_eq!(fallback_bytes, legacy_bytes);
            // The complete output is installed once and cannot overwrite evidence.
            assert!(save_reviewed_values(&path, &record, &cache).is_err());
            assert_eq!(fs::read(&path).unwrap(), legacy_bytes);
            fs::remove_file(&path).unwrap();
            cache.resolver = None;
            assert!(save_reviewed_values(&path, &record, &cache).is_err());
            assert!(!path.exists());
            assert!(!path.with_extension("json.export-partial").exists());
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
        fn only_claim1c_records_an_unproven_ground_index() {
            use xc_spectral::ccm::hp::CcmGroundIndexPolicy;
            assert_eq!(
                super::ground_index_policy(CapturePolicy::Claim1cHp1000),
                CcmGroundIndexPolicy::RecordOutcome
            );
            for policy in [CapturePolicy::Full, CapturePolicy::Claim8Natural] {
                assert_eq!(
                    super::ground_index_policy(policy),
                    CcmGroundIndexPolicy::Require
                );
            }
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
            // Historical v7 groups plus the current finite diagnostics, less
            // eleven planned exclusions; the twelfth is the explicit
            // sector-gap certificate request.
            assert_eq!(excluded.len(), 12);
            assert!(excluded.contains_key("sector_gap_certificate"));
            assert!(!full.contains(&"sector_gap_certificate".into()));
            assert_eq!(
                requested.len(),
                41 + xc_spectral::ccm::capture::FINITE_DIAGNOSTICS.len() - 11
            );
            assert_eq!(
                requested,
                full.into_iter()
                    .filter(|id| !excluded.contains_key(id))
                    .collect::<Vec<_>>()
            );
            assert!(requested.contains(&"transform_enclosure".into()));
            assert!(requested.contains(&"operator_energy".into()));
            for id in ["sector_analysis", "evenness", "finite_root_budget"] {
                assert!(!requested.contains(&id.into()));
            }
            // Toolkit shape-based admission supersedes the old dimension-512
            // cap. Preserve these requests even when their finite premises
            // may remain unresolved at the source precision.
            for id in [
                "directional_error_bound",
                "finite_tail_bound",
                "spectral_cluster_bound",
            ] {
                assert!(!excluded.contains_key(id));
                assert!(requested.contains(&id.into()));
            }
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
