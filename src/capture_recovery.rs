// Copyright (c) 2026 Ronnie Andrews, Jr. (Team Xcelerator Inc.(R))
// All rights reserved. See LICENSE file for terms.
//
// This source code is provided for verification and study purposes only.
// Modification, redistribution, and commercial use are prohibited
// without explicit written permission.
//
// Supplemental recovery deliberately has no RetainedCcmRun or acquisition callback.
use super::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use xc_spectral::ccm::extended_research::ExternalResearchInputs;

const INVENTORY_LIMIT: u64 = 64 * 1024 * 1024;
const COPY_BUFFER_BYTES: usize = 64 * 1024;
const ORIGINAL_REQUIRED_FILES: &[&str] = &[
    "request.json",
    "input-preparation-request.json",
    "primary-sources.json",
    "capture.json",
];
const ORIGINAL_OPTIONAL_FILES: &[&str] = &[
    "capture-explicit.json",
    "primary.json",
    "status.json",
    "research-sources.json",
    "claim-measurements.json",
    "input-preparation-status.json",
];

fn optional_original_inputs(
    journal: &Path,
) -> Result<(
    serde_json::Map<String, serde_json::Value>,
    serde_json::Map<String, serde_json::Value>,
)> {
    let mut digests = serde_json::Map::new();
    let mut inputs = serde_json::Map::new();
    for &name in ORIGINAL_OPTIONAL_FILES {
        match fs::symlink_metadata(journal.join(name)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
        let (value, digest) = frozen_json(&journal.join(name))?;
        digests.insert(name.into(), serde_json::to_value(digest)?);
        if ["research-sources.json", "input-preparation-status.json"].contains(&name) {
            inputs.insert(name.into(), value);
        }
    }
    Ok((digests, inputs))
}

fn verify_original_inventory(journal: &Path, inventory: &serde_json::Value) -> Result<()> {
    let inventory = inventory.as_object().context("invalid frozen inventory")?;
    for &name in ORIGINAL_REQUIRED_FILES
        .iter()
        .chain(ORIGINAL_OPTIONAL_FILES)
    {
        let observed = match fs::symlink_metadata(journal.join(name)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
            Ok(_) => Some(serde_json::to_value(frozen_json(&journal.join(name))?.1)?),
        };
        anyhow::ensure!(
            observed.as_ref() == inventory.get(name),
            "original journal presence or bytes changed during recovery: {name}"
        );
    }
    Ok(())
}

fn stream_copy_digest(
    reader: &mut impl std::io::Read,
    writer: &mut impl Write,
    maximum_bytes: u64,
) -> Result<(xc_cache::ContentDigest, u64)> {
    let mut buffer = [0u8; COPY_BUFFER_BYTES];
    let mut digest = Sha256::new();
    let mut total = 0u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read as u64)
            .context("recovery copy size overflow")?;
        anyhow::ensure!(
            total <= maximum_bytes,
            "recovery file exceeds admission budget"
        );
        digest.update(&buffer[..read]);
        writer.write_all(&buffer[..read])?;
    }
    Ok((
        xc_cache::ContentDigest(format!("{:x}", digest.finalize())),
        total,
    ))
}

fn checkpoint_file(path: &Path, source: &Path, expected: &xc_cache::ContentDigest) -> Result<()> {
    let limit = xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES;
    if path.exists() {
        anyhow::ensure!(
            bounded_file_digest(path)? == *expected,
            "retained recovery file differs: {}",
            path.display()
        );
        return Ok(());
    }
    anyhow::ensure!(
        fs::symlink_metadata(source)?.file_type().is_file(),
        "recovery copy source must be regular"
    );
    let mut source = fs::File::open(source)?;
    let expected_size = source.metadata()?.len();
    anyhow::ensure!(
        expected_size <= limit,
        "recovery copy source exceeds admission budget"
    );
    let temporary = path.with_extension(format!(
        "copy-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    struct RemoveTemporary(PathBuf);
    impl Drop for RemoveTemporary {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let _cleanup = RemoveTemporary(temporary.clone());
    let mut file = options.open(&temporary)?;
    let (actual, copied) = stream_copy_digest(&mut source, &mut file, limit)?;
    anyhow::ensure!(
        actual == *expected && copied == expected_size,
        "recovery copy source changed or its digest differs"
    );
    file.sync_all()?;
    drop(file);
    fs::hard_link(&temporary, path)?;
    #[cfg(unix)]
    fs::File::open(path.parent().context("checkpoint parent missing")?)?.sync_all()?;
    Ok(())
}

fn recovery_replace_file(
    path: &Path,
    source: &Path,
    expected: &xc_cache::ContentDigest,
) -> Result<()> {
    let temporary = path.with_extension(format!(
        "update-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    checkpoint_file(&temporary, source, expected)?;
    let result = (|| -> Result<()> {
        if path.exists() {
            anyhow::ensure!(
                fs::symlink_metadata(path)?.file_type().is_file(),
                "invalid recovery export location"
            );
        }
        fs::rename(&temporary, path)?;
        #[cfg(unix)]
        fs::File::open(path.parent().context("recovery export parent missing")?)?.sync_all()?;
        Ok(())
    })();
    let _ = fs::remove_file(temporary);
    result
}

fn checkpoint_capture_export<F>(
    aggregate: &Path,
    capture_digest: &xc_cache::ContentDigest,
    produce: F,
) -> Result<(PathBuf, xc_cache::ContentDigest)>
where
    F: FnOnce(&Path) -> Result<()>,
{
    let export = aggregate.join(format!("{}.values.json", capture_digest.0));
    let checkpoint = aggregate.join(format!("{}.values-checkpoint.json", capture_digest.0));
    if checkpoint.exists() {
        let retained = frozen_json(&checkpoint)?.0;
        let expected: xc_cache::ContentDigest =
            serde_json::from_value(retained["export_sha256"].clone())?;
        let expected_size = retained["export_bytes"]
            .as_u64()
            .context("retained export size missing")?;
        anyhow::ensure!(
            retained["schema_version"] == 1
                && expected.validate()
                && retained["capture_sha256"] == serde_json::to_value(capture_digest)?
                && fs::metadata(&export)?.len() == expected_size
                && bounded_file_digest(&export)? == expected,
            "retained aggregate export identity or bytes differ"
        );
        return Ok((export, expected));
    }
    // An export without this checkpoint is provisional. Regenerate through
    // Toolkit's reference validation and require exact old bytes if present.
    let unique = aggregate.join(format!(
        "{}.{}.values.json",
        capture_digest.0,
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let result = (|| -> Result<xc_cache::ContentDigest> {
        produce(&unique)?;
        let digest = bounded_file_digest(&unique)?;
        let size = fs::metadata(&unique)?.len();
        checkpoint_file(&export, &unique, &digest)?;
        checkpoint_save(
            &checkpoint,
            &serde_json::json!({"schema_version":1,
            "capture_sha256":capture_digest,"export_sha256":digest,"export_bytes":size}),
        )?;
        Ok(digest)
    })();
    let _ = fs::remove_file(unique);
    Ok((export, result?))
}

fn staged_draft_inventory(
    drafts: &[xc_cache::CanonicalProductionDraft],
) -> Result<serde_json::Value> {
    let mut identities = drafts
        .iter()
        .map(xc_cache::ContentDigest::sha256_json)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    identities.sort();
    Ok(
        serde_json::json!({"schema_version":1,"draft_count":identities.len(),
        "descriptor_digests":identities,"sha256":xc_cache::ContentDigest::sha256_json(&identities)?}),
    )
}

fn verify_staged_draft_inventory(
    expected: &serde_json::Value,
    drafts: &[xc_cache::CanonicalProductionDraft],
) -> Result<()> {
    anyhow::ensure!(
        *expected == staged_draft_inventory(drafts)?,
        "retained publication staging inventory differs from the completed supplemental capture"
    );
    Ok(())
}

fn verify_completed_capture(output: &Path, completion: &serde_json::Value) -> Result<()> {
    let files = completion["files"]
        .as_object()
        .context("invalid completed capture inventory")?;
    anyhow::ensure!(
        completion["schema_version"] == 1
            && [
                "capture.json",
                "capture-values.json",
                "capture-request.json",
                "external-research-inputs.json",
                "preparation-evidence.json",
                "input-preparation-status.json"
            ]
            .iter()
            .all(|name| files.contains_key(*name)),
        "incomplete retained capture inventory"
    );
    for (name, digest) in files {
        anyhow::ensure!(
            Path::new(name).components().count() == 1
                && serde_json::to_value(bounded_file_digest(&output.join(name))?)? == *digest,
            "retained supplemental capture file differs"
        );
    }
    let record: xc_cache::CaptureArtifact =
        serde_json::from_value(frozen_json(&output.join("capture.json"))?.0)?;
    record.validate()?;
    anyhow::ensure!(
        record.resolved_plan == frozen_json(&output.join("capture-request.json"))?.0,
        "retained capture request differs"
    );
    Ok(())
}

// Authenticated recovery state is append-only. A temporary file is synced
// before a non-replacing hard-link installs it; an interrupted write can never
// masquerade as a completed checkpoint. Operational status is separate.
fn checkpoint_save(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    checkpoint_bytes(path, &bytes)
}

fn checkpoint_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.exists() {
        anyhow::ensure!(
            fs::symlink_metadata(path)?.file_type().is_file()
                && fs::metadata(path)?.len() == bytes.len() as u64
                && fs::read(path)? == bytes,
            "retained recovery file differs: {}",
            path.display()
        );
        return Ok(());
    }
    let temporary = path.with_extension(format!(
        "partial-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    let installed = fs::hard_link(&temporary, path);
    let _ = fs::remove_file(&temporary);
    installed?;
    #[cfg(unix)]
    fs::File::open(path.parent().context("checkpoint parent missing")?)?.sync_all()?;
    Ok(())
}

fn recovery_replace(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    recovery_replace_bytes(path, &bytes)
}

fn recovery_replace_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let temporary = path.with_extension(format!(
        "update-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    checkpoint_bytes(&temporary, bytes)?;
    if path.exists() {
        anyhow::ensure!(
            fs::symlink_metadata(path)?.file_type().is_file(),
            "invalid recovery status location"
        );
    }
    fs::rename(temporary, path)?;
    #[cfg(unix)]
    fs::File::open(path.parent().context("recovery status parent missing")?)?.sync_all()?;
    Ok(())
}

fn verify_diagnostic_sources(
    record: &xc_cache::CaptureArtifact,
    cache: &ArtifactCacheContext<'_>,
) -> Result<()> {
    record.validate()?;
    let resolver = cache
        .resolver
        .context("retained diagnostic requires a cache resolver")?;
    let policy = cache
        .acceptance
        .context("retained diagnostic requires cache policy")?;
    resolver.prefetch_exact_dependencies(&record.source_dependencies, policy)?;
    for dependency in &record.source_dependencies {
        let source = resolver
            .resolve_exact_encoded(
                &dependency.key,
                &dependency.content_digest,
                dependency.required_quality,
                policy,
            )?
            .context("completed diagnostic source is unavailable; no computation attempted")?;
        resolver.validate_dependency_closure(&source.manifest, policy)?;
    }
    for measurement in record.measurements.values() {
        // Exact reference digests and their coverage retain the normal Toolkit
        // validation; a missing/corrupt source never authorizes a fresh solve.
        if measurement.value_reference.is_some() {
            xc_cache::measurement_value(measurement, resolver, policy)?;
        }
    }
    Ok(())
}

fn checkpoint_diagnostic<F>(
    directory: &Path,
    index: usize,
    id: &str,
    plan: &serde_json::Value,
    cache: &ArtifactCacheContext<'_>,
    execute: F,
) -> Result<xc_cache::CaptureArtifact>
where
    F: FnOnce() -> std::result::Result<xc_cache::CapturedDiagnostic, xc_cache::CaptureFailure>,
{
    let directory = directory
        .join("diagnostic-checkpoints")
        .join(format!("{index:03}"));
    fs::create_dir_all(&directory)?;
    let mut next = None;
    for attempt in 0..16 {
        let path = directory.join(format!("{attempt:02}.json"));
        if !path.exists() {
            next = Some(path);
            break;
        }
        let record: xc_cache::CaptureArtifact = serde_json::from_value(frozen_json(&path)?.0)?;
        record.validate()?;
        anyhow::ensure!(
            record.resolved_plan == *plan && record.requested_diagnostics == vec![id.to_owned()],
            "retained diagnostic identity differs"
        );
        if record.receipt.is_complete() {
            verify_diagnostic_sources(&record, cache)?;
            xc_core::progress_message!(
                "supplemental recovery: diagnostic {id}: reusing completed checkpoint"
            );
            return Ok(record);
        }
    }
    let path = next.context("diagnostic retry limit (16 retained attempts) reached")?;
    // Record the approved diagnostic name before invoking any expensive work.
    let mut events = OpenOptions::new().create(true).append(true).open(
        directory
            .parent()
            .and_then(Path::parent)
            .context("diagnostic journal missing")?
            .join("recovery-progress.jsonl"),
    )?;
    event(
        &mut events,
        &serde_json::json!({"diagnostic":id,"status":"started"}),
    )?;
    events.sync_data()?;
    xc_core::progress_message!("supplemental recovery: diagnostic {id}: started");
    let mut result = Some(execute);
    let record =
        xc_cache::collect_capture(plan, vec![id.to_owned()], |_| result.take().unwrap()())?;
    checkpoint_save(&path, &record)?;
    event(
        &mut events,
        &serde_json::json!({"diagnostic":id,"status":"checkpoint_saved",
        "acquisition_complete":record.receipt.is_complete()}),
    )?;
    events.sync_data()?;
    Ok(record)
}

fn bounded_file_digest(path: &Path) -> Result<xc_cache::ContentDigest> {
    anyhow::ensure!(
        fs::symlink_metadata(path)?.file_type().is_file(),
        "retained identity file must be regular"
    );
    let mut file = fs::File::open(path)?;
    let limit = xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES;
    anyhow::ensure!(
        file.metadata()?.is_file() && file.metadata()?.len() <= limit,
        "producer identity file exceeds admission budget"
    );
    let expected_size = file.metadata()?.len();
    let (digest, observed_size) = stream_copy_digest(&mut file, &mut std::io::sink(), limit)?;
    anyhow::ensure!(
        observed_size == expected_size,
        "retained identity file changed while hashing"
    );
    Ok(digest)
}

fn producer_identity(configuration: &str) -> Result<serde_json::Value> {
    let preparer: InputPreparer = serde_json::from_str(configuration)?;
    anyhow::ensure!(!preparer.argv.is_empty(), "empty preparer command");
    let executable = PathBuf::from(&preparer.argv[0]);
    let executable = if executable.components().count() == 1 {
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .map(|p| p.join(&executable))
            .find(|p| p.is_file())
            .context("preparer executable not found")?
    } else {
        executable
    };
    let mut files = std::collections::BTreeMap::new();
    for path in std::iter::once(executable).chain(
        preparer
            .argv
            .iter()
            .skip(1)
            .map(PathBuf::from)
            .filter(|p| p.is_file()),
    ) {
        let path = path.canonicalize()?;
        files.insert(path.display().to_string(), bounded_file_digest(&path)?);
        if let Some(parent) = path.parent() {
            let manifest_path = parent.join("preparer-snapshot.json");
            if manifest_path.exists() {
                let (manifest, digest) = frozen_json(&manifest_path)?;
                files.insert(manifest_path.display().to_string(), digest);
                for (name, expected) in manifest.as_object().context("invalid preparer snapshot")? {
                    anyhow::ensure!(
                        Path::new(name).components().count() == 1 && !name.starts_with('.'),
                        "invalid preparer snapshot entry"
                    );
                    let source = parent.join(name);
                    anyhow::ensure!(
                        !fs::symlink_metadata(&source)?.file_type().is_symlink(),
                        "preparer snapshot source is a symbolic link"
                    );
                    let digest = bounded_file_digest(&source)?;
                    anyhow::ensure!(
                        serde_json::to_value(&digest)? == *expected,
                        "preparer snapshot source identity differs"
                    );
                    files.insert(source.display().to_string(), digest);
                }
            }
        }
    }
    // Operational ceilings may increase on retry. The command, implementation,
    // source admission option and compiled numerical features stay identical.
    let runtime = std::env::var_os("XC_RECOVERY_PRODUCER_IDENTITY_FILE")
        .map(|path| -> Result<serde_json::Value> {
            let path = PathBuf::from(path).canonicalize()?;
            let (_, digest) = frozen_json(&path)?;
            Ok(serde_json::json!({"path":path,"sha256":digest}))
        })
        .transpose()?;
    let snapshot_runtime_required = files.keys().any(|path| {
        Path::new(path)
            .file_name()
            .is_some_and(|name| name == "preparer-snapshot.json")
    });
    Ok(
        serde_json::json!({"argv_sha256":xc_cache::ContentDigest::sha256_json(&preparer.argv)?,"files":files,
        "runtime_identity":runtime,"snapshot_runtime_required":snapshot_runtime_required,
        "source_coefficient_limit":std::env::var("XC_SMOOTH_SOURCE_MAXIMUM_COEFFICIENTS").ok(),
        "cargo_lock_sha256":xc_cache::ContentDigest::sha256(include_bytes!("../Cargo.lock")),
        "driver_sha256":xc_cache::ContentDigest::sha256_chunks([
            include_bytes!("capture.rs").as_slice(), include_bytes!("capture_recovery.rs").as_slice()]),
        "root_certification":cfg!(feature="root-certification"),
        "experimental_gl_root_parallel":cfg!(feature="experimental-gl-root-parallel")}),
    )
}

fn prepared_attempt(
    directory: &Path,
    request: &serde_json::Value,
    preparer: &str,
) -> Result<PreparedInputs> {
    let attempts = directory.join("preparation-attempts");
    fs::create_dir_all(&attempts)?;
    for index in 0..16 {
        let attempt = attempts.join(format!("{index:02}"));
        if attempt.exists() {
            if attempt.join("completed.json").exists() {
                let completed = frozen_json(&attempt.join("completed.json"))?.0;
                let files = completed["files"]
                    .as_object()
                    .context("invalid preparation inventory")?;
                anyhow::ensure!(
                    completed["schema_version"] == 1
                        && files.contains_key("prepared-research-inputs.json")
                        && files.contains_key("input-preparation-request.json")
                        && frozen_json(&attempt.join("input-preparation-request.json"))?.0
                            == *request,
                    "retained preparation request differs"
                );
                let mut prepared_bytes = None;
                for (name, digest) in files {
                    anyhow::ensure!(
                        [
                            "input-preparation-request.json",
                            "prepared-research-inputs.json",
                            "smooth-qualification.json",
                            "prepared-research-assessment.json"
                        ]
                        .contains(&name.as_str()),
                        "invalid retained preparation file name"
                    );
                    anyhow::ensure!(
                        fs::symlink_metadata(attempt.join(name))?
                            .file_type()
                            .is_file()
                            && fs::metadata(attempt.join(name))?.len()
                                <= xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES,
                        "invalid retained preparation file size or type"
                    );
                    let bytes = fs::read(attempt.join(name))?;
                    anyhow::ensure!(
                        bytes.len() as u64
                            <= xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES
                            && serde_json::to_value(xc_cache::ContentDigest::sha256(&bytes))?
                                == *digest,
                        "retained preparation file differs: {name}"
                    );
                    checkpoint_bytes(&directory.join(name), &bytes)?;
                    if name == "prepared-research-inputs.json" {
                        prepared_bytes = Some(bytes);
                    }
                }
                let bytes = prepared_bytes.context("retained prepared input missing")?;
                let input = ExternalResearchInputs::from_bytes(
                    &attempt.join("prepared-research-inputs.json"),
                    &bytes,
                )?;
                xc_core::progress_message!(
                    "supplemental recovery: reusing authenticated completed preparation"
                );
                return Ok(PreparedInputs {
                    input,
                    digest: xc_cache::ContentDigest::sha256(&bytes),
                });
            }
            continue;
        }
        let mut retained_bytes = 0u64;
        for retained in fs::read_dir(&attempts)? {
            let retained = retained?;
            anyhow::ensure!(
                retained.file_type()?.is_dir(),
                "unexpected preparation attempt entry"
            );
            for file in fs::read_dir(retained.path())? {
                retained_bytes = retained_bytes
                    .checked_add(file?.metadata()?.len())
                    .context("retained preparation byte count overflow")?;
            }
        }
        anyhow::ensure!(retained_bytes <= 4 * xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES,
            "retained preparation attempts exceed the retry disk budget; inspect them before starting a fresh recovery");
        fs::create_dir(&attempt)?;
        let prepared = invoke_preparer(&attempt, request, preparer)?;
        let mut files = serde_json::Map::new();
        for name in [
            "input-preparation-request.json",
            "prepared-research-inputs.json",
            "smooth-qualification.json",
            "prepared-research-assessment.json",
        ] {
            if attempt.join(name).exists() {
                files.insert(
                    name.into(),
                    serde_json::to_value(bounded_file_digest(&attempt.join(name))?)?,
                );
            }
        }
        checkpoint_save(
            &attempt.join("completed.json"),
            &serde_json::json!({"schema_version":1,"files":files}),
        )?;
        // Loading through the same path verifies every retained byte before use.
        drop(prepared);
        return prepared_attempt(directory, request, preparer);
    }
    anyhow::bail!("preparation retry limit (16 retained attempts) reached; inspect retained attempts before starting a fresh recovery")
}

fn merge_diagnostic_records(
    plan: &serde_json::Value,
    diagnostics: &[String],
    records: &[xc_cache::CaptureArtifact],
) -> Result<xc_cache::CaptureArtifact> {
    let mut receipt = xc_core::CaptureReceipt::new(plan, diagnostics.to_vec())?;
    let mut measurements = std::collections::BTreeMap::new();
    let mut dependencies = std::collections::BTreeMap::new();
    anyhow::ensure!(
        records.len() == diagnostics.len(),
        "recovery diagnostic inventory differs"
    );
    for (id, record) in diagnostics.iter().zip(records) {
        record.validate()?;
        anyhow::ensure!(
            record.resolved_plan == *plan && record.requested_diagnostics == [id.clone()],
            "retained diagnostic plan differs"
        );
        receipt.record(
            id,
            record
                .receipt
                .outcomes()
                .get(id)
                .context("diagnostic outcome missing")?
                .clone(),
        )?;
        measurements.extend(record.measurements.clone());
        for dependency in &record.source_dependencies {
            let key = (
                dependency.key.kind.clone(),
                dependency.key.logical_key.clone(),
                dependency.key.parameters_digest.clone(),
                dependency.content_digest.clone(),
            );
            if let Some(old) = dependencies.insert(key, dependency.clone()) {
                anyhow::ensure!(
                    old == *dependency,
                    "conflicting retained diagnostic dependencies"
                );
            }
        }
    }
    let mut record = xc_cache::CaptureArtifact {
        schema_version: 1,
        semantics: xc_cache::CAPTURE_RECORD_SEMANTICS.into(),
        resolved_plan: plan.clone(),
        requested_diagnostics: diagnostics.to_vec(),
        receipt,
        measurements,
        source_dependencies: dependencies.into_values().collect(),
        numerical_coverage: Default::default(),
    };
    record.numerical_coverage = record.coverage();
    record.validate()?;
    Ok(record)
}

fn frozen_json(path: &Path) -> Result<(serde_json::Value, xc_cache::ContentDigest)> {
    frozen_json_with_limit(path, INVENTORY_LIMIT)
}

fn frozen_json_with_limit(
    path: &Path,
    maximum_bytes: u64,
) -> Result<(serde_json::Value, xc_cache::ContentDigest)> {
    use std::io::Read;
    anyhow::ensure!(
        fs::symlink_metadata(path)?.file_type().is_file(),
        "retained journal document must be a regular file"
    );
    anyhow::ensure!(
        fs::metadata(path)?.len() <= maximum_bytes,
        "retained journal document exceeds admission budget"
    );
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(maximum_bytes + 1)
        .read_to_end(&mut bytes)?;
    anyhow::ensure!(
        bytes.len() as u64 <= maximum_bytes,
        "retained journal document exceeds admission budget"
    );
    Ok((
        serde_json::from_slice(&bytes)?,
        xc_cache::ContentDigest::sha256(&bytes),
    ))
}

fn recovery_diagnostics(
    record: &xc_cache::CaptureArtifact,
    selected: &[String],
) -> Result<Vec<String>> {
    let requested: BTreeSet<_> = record.requested_diagnostics.iter().cloned().collect();
    anyhow::ensure!(
        requested.len() == record.requested_diagnostics.len()
            && requested == record.receipt.outcomes().keys().cloned().collect(),
        "original receipt/request inventory disagrees"
    );
    for (id, outcome) in record.receipt.outcomes() {
        if matches!(outcome, xc_core::DiagnosticOutcome::Completed { .. }) {
            anyhow::ensure!(
                record.measurements.contains_key(id),
                "completed original diagnostic has no measurement: {id}"
            );
        }
    }
    let result = if selected.is_empty() {
        record
            .receipt
            .outcomes()
            .iter()
            .filter_map(|(id, outcome)| {
                (!matches!(outcome, xc_core::DiagnosticOutcome::Completed { .. }))
                    .then_some(id.clone())
            })
            .collect::<Vec<_>>()
    } else {
        selected.to_vec()
    };
    let unique: BTreeSet<_> = result.iter().collect();
    anyhow::ensure!(unique.len() == result.len(), "duplicate recovery request");
    for id in &result {
        anyhow::ensure!(
            requested.contains(id),
            "recovery diagnostic was not requested by the original journal: {id}"
        );
        let normalized = id.strip_suffix("_full").unwrap_or(id);
        anyhow::ensure!(
            xc_spectral::ccm::capture::FINITE_DIAGNOSTICS.contains(&id.as_str())
                || xc_spectral::ccm::extended_research::DIAGNOSTICS.contains(&normalized)
                || xc_spectral::ccm::convergence_capture::DIAGNOSTICS.contains(&normalized)
                || xc_spectral::ccm::research_completion::DIAGNOSTICS.contains(&normalized),
            "diagnostic has no retained-source recovery route: {id}"
        );
    }
    Ok(result)
}

fn prepared_diagnostics(
    original: &xc_cache::CaptureArtifact,
    initial: &[String],
    explicit: bool,
    preparation_failed: bool,
    input: &serde_json::Value,
    input_digest: &xc_cache::ContentDigest,
) -> (
    Vec<String>,
    std::collections::BTreeMap<String, serde_json::Value>,
) {
    let mut diagnostics: BTreeSet<_> = initial.iter().cloned().collect();
    let mut affected = std::collections::BTreeMap::new();
    if preparation_failed {
        let complex = input["finite_diagnostics"]["complex_trials"].is_object();
        let continuous = input["finite_diagnostics"]["continuous_energy"].is_object();
        let projection = input["finite_diagnostics"]["projection_energy"].is_object();
        let energy_fields: Vec<_> = [
            ("complex_trials", complex),
            ("continuous_energy", continuous),
        ]
        .into_iter()
        .filter_map(|(field, supplied)| supplied.then_some(field))
        .collect();
        for (id, supplied, fields) in [
            ("trial_vector_energy", complex || continuous, energy_fields),
            ("trial_vector_parity", complex, vec!["complex_trials"]),
            (
                "continuous_l1_bound",
                complex,
                vec!["complex_trials trial-stage profiles"],
            ),
            (
                "normalization_error_bound",
                complex,
                vec!["complex_trials trial-stage profiles"],
            ),
            ("finite_tail_bound", projection, vec!["projection_energy"]),
        ] {
            if supplied && original.requested_diagnostics.iter().any(|name| name == id) {
                affected.insert(id.into(), serde_json::json!({
                    "reason":"successful private preparation supplies source-dependent data after the original preparation failed",
                    "finite_diagnostics_fields":fields,"source_eigenpair":input["source_eigenpair"],
                    "external_input_digest":input_digest}));
                if !explicit {
                    diagnostics.insert(id.into());
                }
            }
        }
    }
    (diagnostics.into_iter().collect(), affected)
}

fn recovery_request(
    original: &serde_json::Value,
    configuration: &serde_json::Value,
    manifests: &[xc_cache::ArtifactManifest],
    target_path: &Path,
    target_digest: &xc_cache::ContentDigest,
    base: &ExternalResearchInputs,
) -> Result<serde_json::Value> {
    validate_recovery_request(original, configuration, manifests, target_digest)?;
    let mut request = original.clone();
    request["target_spec_file"] = serde_json::to_value(target_path)?;
    request["base_inputs"] = serde_json::to_value(base)?;
    Ok(request)
}

/// Retain arithmetic prerequisites independently from external preparation.
/// Reuse is authorized only by the committed request digest and the complete
/// recovery identity; validation still authenticates source joins on every run.
fn checkpoint_preparation_request<P, V>(
    directory: &Path,
    identity: &serde_json::Value,
    prepare: P,
    validate: V,
) -> Result<serde_json::Value>
where
    P: FnOnce() -> Result<serde_json::Value>,
    V: Fn(&serde_json::Value) -> Result<()>,
{
    let checkpoint_path = directory.join("recovery-preparation-checkpoint.json");
    let request_path = directory.join("recovery-preparation-request.json");
    let identity_digest = xc_cache::ContentDigest::sha256_json(identity)?;
    if checkpoint_path.exists() {
        let checkpoint = frozen_json(&checkpoint_path)?.0;
        anyhow::ensure!(
            checkpoint["schema_version"] == 1
                && checkpoint["recovery_identity_sha256"]
                    == serde_json::to_value(&identity_digest)?,
            "retained arithmetic preparation recovery identity differs"
        );
        let (request, digest) = frozen_json_with_limit(
            &request_path,
            xc_spectral::ccm::capture_runtime::RESEARCH_INPUT_MAXIMUM_BYTES,
        )?;
        anyhow::ensure!(
            checkpoint["request_sha256"] == serde_json::to_value(&digest)?,
            "retained arithmetic preparation request bytes differ"
        );
        validate(&request)?;
        xc_core::progress_message!(
            "supplemental recovery: reusing authenticated arithmetic preparation request"
        );
        return Ok(request);
    }
    // A request without its commit checkpoint is provisional. Recompute the
    // arithmetic prerequisites and require identical bytes before committing.
    // Never infer success from an interrupted preparer's request file alone.
    let request = prepare()?;
    validate(&request)?;
    checkpoint_save(&request_path, &request)?;
    let digest = bounded_file_digest(&request_path)?;
    checkpoint_save(
        &checkpoint_path,
        &serde_json::json!({"schema_version":1,
        "recovery_identity_sha256":identity_digest,"request_sha256":digest}),
    )?;
    Ok(request)
}

fn validate_recovery_request(
    original: &serde_json::Value,
    configuration: &serde_json::Value,
    manifests: &[xc_cache::ArtifactManifest],
    target_digest: &xc_cache::ContentDigest,
) -> Result<()> {
    anyhow::ensure!(
        original["schema_version"] == 1
            && original["primary_recomputation"] == false
            && original["output_visibility"] == "private_only",
        "unsupported retained preparation request"
    );
    anyhow::ensure!(
        &original["configuration"] == configuration,
        "original preparation configuration differs from original run"
    );
    anyhow::ensure!(
        original["primary_sources"] == serde_json::to_value(manifests)?,
        "original preparation source inventory differs"
    );
    anyhow::ensure!(
        original["configuration"]["research_inputs"]["XC_TARGET_SPEC_FILE"]["status"] == "supplied"
            && original["configuration"]["research_inputs"]["XC_TARGET_SPEC_FILE"]["sha256"]
                == serde_json::to_value(target_digest)?,
        "recovery target differs from the target authenticated before original acquisition"
    );
    Ok(())
}

/// Add new source-bound children in a new journal. Primary/root source misses
/// fail closed in RetainedFiniteSources; this function has no acquisition path.
pub fn recover_capture_with_mode(
    journal: &Path,
    output: &Path,
    target: Option<&Path>,
    selected: &[String],
    resume: bool,
    publish_only: bool,
) -> Result<()> {
    let preparer = std::env::var("XC_RESEARCH_PREPARER")
        .context("recovery requires an explicitly configured private preparer")?;
    let config = xc_cache::ManagedArtifactCacheConfig::from_environment()?
        .context("managed retained cache required")?;
    recover_capture_configured_mode(
        journal,
        output,
        target,
        selected,
        &preparer,
        config,
        resume,
        publish_only,
    )
}

#[cfg(test)]
fn recover_capture_configured(
    journal: &Path,
    output: &Path,
    target: Option<&Path>,
    selected: &[String],
    preparer: &str,
    managed_config: xc_cache::ManagedArtifactCacheConfig,
) -> Result<()> {
    recover_capture_configured_mode(
        journal,
        output,
        target,
        selected,
        preparer,
        managed_config,
        false,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn recover_capture_configured_mode(
    journal: &Path,
    output: &Path,
    target: Option<&Path>,
    selected: &[String],
    preparer: &str,
    mut managed_config: xc_cache::ManagedArtifactCacheConfig,
    resume: bool,
    publish_only: bool,
) -> Result<()> {
    use xc_spectral::ccm::convergence_capture::finite_capture::RetainedFiniteSources;
    let resources = xc_cache::managed_resource_policy_from_environment()?;
    let journal = journal.canonicalize()?;
    let parent = output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()?;
    let output = parent.join(
        output
            .file_name()
            .context("recovery output needs a directory name")?,
    );
    anyhow::ensure!(
        !output.starts_with(&journal),
        "recovery output must be outside the original journal"
    );
    let (configuration, configuration_digest) = frozen_json(&journal.join("request.json"))?;
    let (original_request, preparation_digest) =
        frozen_json(&journal.join("input-preparation-request.json"))?;
    let (primary_value, primary_digest) = frozen_json(&journal.join("primary-sources.json"))?;
    let manifests: Vec<xc_cache::ArtifactManifest> = serde_json::from_value(primary_value)?;
    let (original_capture, capture_digest) = frozen_json(&journal.join("capture.json"))?;
    let original: xc_cache::CaptureArtifact = serde_json::from_value(original_capture)?;
    original.validate()?;
    if let Some(requested) = configuration["applicability"]["requested_diagnostics"].as_array() {
        let declared: BTreeSet<String> = requested
            .iter()
            .map(|id| {
                id.as_str()
                    .map(str::to_owned)
                    .context("invalid original applicability diagnostic")
            })
            .collect::<Result<_>>()?;
        anyhow::ensure!(
            declared == original.requested_diagnostics.iter().cloned().collect(),
            "original applicability and capture receipt disagree"
        );
    }
    let initial_diagnostics = recovery_diagnostics(&original, selected)?;
    let (optional_digests, optional_inputs) = optional_original_inputs(&journal)?;
    let preparation_failed = optional_inputs
        .get("input-preparation-status.json")
        .is_some_and(|status| status["status"] == "failed");
    anyhow::ensure!(
        !initial_diagnostics.is_empty() || (selected.is_empty() && preparation_failed),
        "empty recovery request and no failed preparation to refresh"
    );
    let target = target
        .map(Path::to_path_buf)
        .or_else(|| {
            original_request["target_spec_file"]
                .as_str()
                .map(PathBuf::from)
        })
        .context("original preparation request has no target file")?
        .canonicalize()?;
    let (_, target_digest) = frozen_json(&target)?;
    validate_recovery_request(
        &original_request,
        &configuration,
        &manifests,
        &target_digest,
    )?;
    anyhow::ensure!(
        matches!(
            managed_config.publication_target,
            xc_core::PublicationTarget::None | xc_core::PublicationTarget::Private
        ) && !managed_config.replace_existing_publication,
        "recovery permits additive private publication only; replacement is forbidden"
    );
    managed_config.cache_mode = xc_cache::ArtifactExecutionCacheMode::PreferReuse;
    anyhow::ensure!(
        !(resume && publish_only),
        "resume and publish-only are mutually exclusive"
    );
    // Existing staging is accepted only through authenticated explicit recovery.
    if managed_config.publication_target != xc_core::PublicationTarget::None {
        let staging = managed_config
            .staging_root
            .as_ref()
            .context("private recovery requires a fresh explicit staging root")?;
        anyhow::ensure!(
            resume || publish_only || !staging.exists(),
            "recovery staging root already exists; select a fresh directory"
        );
    } else {
        managed_config.staging_root = None;
    }
    managed_config.output_validation = None;
    if resume || publish_only {
        anyhow::ensure!(fs::symlink_metadata(&output)?.file_type().is_dir()
            && output.join("recovery-checkpoint.json").is_file(),
            "resume requires a recovery-checkpoint.json from this driver; legacy supplemental journals cannot be resumed; use a fresh output (existing immutable artifact hits still apply)");
    } else {
        fs::create_dir(&output)?;
    }
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(output.join("recovery.lock"))?;
    lock.try_lock()
        .context("another process owns this supplemental recovery")?;
    let mut frozen = serde_json::json!({"request.json":configuration_digest,"input-preparation-request.json":preparation_digest,
        "primary-sources.json":primary_digest,"capture.json":capture_digest});
    for (name, digest) in optional_digests {
        frozen[name] = digest;
    }
    let identity = serde_json::json!({"schema_version":1,"original_journal":journal,
        "original_files":frozen,"target_sha256":target_digest,
        "selected_diagnostics":selected,"initial_diagnostics":initial_diagnostics,
        "producer":producer_identity(preparer)?,
        "cache_root":managed_config.cache_root,"staging_root":managed_config.staging_root,
        "publication_target":format!("{:?}", managed_config.publication_target),
        "profile":format!("{:?}", managed_config.profile),
        "assurance":format!("{:?}", managed_config.requested_assurance),
        "certification_failure_policy":format!("{:?}", managed_config.certification_failure_policy),
        "repository_owner":managed_config.repository_owner,
        "remote_cache_mode":format!("{:?}", managed_config.remote_cache_mode)});
    let checkpoint_path = output.join("recovery-checkpoint.json");
    if resume || publish_only {
        anyhow::ensure!(frozen_json(&checkpoint_path)?.0 == identity,
            "recovery identity differs: original journal, target, selection, producer, code or publication route changed");
        anyhow::ensure!(identity["producer"]["snapshot_runtime_required"] != true
            || !identity["producer"]["runtime_identity"].is_null(),
            "this recovery did not freeze its preparation runtime; start a fresh recovery with XC_RECOVERY_PRODUCER_IDENTITY_FILE to enable authenticated resume");
    } else {
        checkpoint_save(&checkpoint_path, &identity)?;
    }
    let completion = if (resume || publish_only) && output.join("capture-completed.json").exists() {
        Some(frozen_json(&output.join("capture-completed.json"))?.0)
    } else {
        anyhow::ensure!(!publish_only,
            "publication-only requires a retained final supplemental capture; use --resume to finish acquisition");
        None
    };
    let managed =
        xc_cache::ManagedArtifactCacheSession::new_with_resources(managed_config, resources)?;
    // Both retry modes authenticate the previous committed capture and queue
    // before any preparation, capture or staging can append legitimate work.
    if let Some(completion) = &completion {
        verify_completed_capture(&output, completion)?;
        verify_original_inventory(&journal, &frozen)?;
        verify_staged_draft_inventory(&completion["staged_drafts"], &managed.staged_drafts()?)?;
    }
    if publish_only {
        recovery_replace(
            &output.join("publication-status.json"),
            &serde_json::json!({"status":"retry_started","capture_recomputed":false}),
        )?;
        managed.retry_retained_publication()?;
        recovery_replace(
            &output.join("publication-status.json"),
            &serde_json::json!({"status":"finalized","capture_recomputed":false}),
        )?;
        println!(
            "Supplemental publication retry journal: {}",
            output.display()
        );
        return Ok(());
    }
    let cache = managed.context();
    checkpoint_save(
        &output.join("request.json"),
        &serde_json::json!({"schema_version":1,"phase":"retained_capture_recovery",
        "original_journal":journal,"original_files":frozen,"initial_diagnostics":initial_diagnostics,
        "selection_policy":if selected.is_empty(){"unfinished plus affected finite diagnostics after successful preparation"}else{"explicit selection"},
        "original_preparation_failed":preparation_failed,"primary_recomputation":false,
        "root_recomputation":false,"original_claim_receipt_unchanged":true,"cargo_lock":include_str!("../Cargo.lock")}),
    )?;
    let result = (|| -> Result<()> {
        let sources = RetainedFiniteSources::from_manifests(&manifests, true, &cache)?;
        let request = checkpoint_preparation_request(
            &output,
            &identity,
            || {
                let old_base: Option<ExternalResearchInputs> =
                    if original_request["base_inputs"].is_null() {
                        None
                    } else {
                        Some(serde_json::from_value(
                            original_request["base_inputs"].clone(),
                        )?)
                    };
                let base = sources.prepare_recovery_inputs(old_base.as_ref())?;
                recovery_request(
                    &original_request,
                    &configuration,
                    &manifests,
                    &target,
                    &target_digest,
                    &base,
                )
            },
            |request| {
                validate_recovery_request(request, &configuration, &manifests, &target_digest)?;
                anyhow::ensure!(
                    request["target_spec_file"] == serde_json::to_value(&target)?,
                    "retained arithmetic preparation target location differs"
                );
                let base: ExternalResearchInputs =
                    serde_json::from_value(request["base_inputs"].clone())?;
                sources.validate_recovery_input(&base)
            },
        )?;
        checkpoint_save(&output.join("primary-sources.json"), &manifests)?;
        // Use only the exact optional bytes admitted into the original
        // inventory. An initially absent file never becomes a new input.
        verify_original_inventory(&journal, &frozen)?;
        let mut parents: Vec<xc_cache::ArtifactManifest> = optional_inputs
            .get("research-sources.json")
            .map(|value| serde_json::from_value(value.clone()))
            .transpose()?
            .unwrap_or_default();
        for manifest in &manifests {
            if !parents
                .iter()
                .any(|m| m.key == manifest.key && m.content_digest == manifest.content_digest)
            {
                parents.push(manifest.clone());
            }
        }
        checkpoint_save(&output.join("research-sources.json"), &parents)?;
        anyhow::ensure!(
            producer_identity(preparer)? == identity["producer"],
            "preparer source changed before preparation"
        );
        let prepared = prepared_attempt(&output, &request, preparer)?;
        anyhow::ensure!(
            producer_identity(preparer)? == identity["producer"],
            "preparer source changed during preparation"
        );
        sources.validate_recovery_input(&prepared.input)?;
        let mut input = prepared.input;
        // An explicitly excluded old sector solve supplies provenance for an
        // assessment of unavailable vectors. It is not evidence that a new
        // sector solve ran, nor a substitute set of cluster eigenvectors.
        if initial_diagnostics
            .iter()
            .any(|id| id == "spectral_cluster_full" || id == "spectral_cluster")
            && input.cluster.is_empty()
            && input.cluster_preparation_assessment.is_none()
        {
            if let Some(reason) =
                configuration["applicability"]["excluded_diagnostics"]["sector_analysis"].as_str()
            {
                let state = manifests
                    .iter()
                    .find(|m| m.key.kind == "ccm_weil_eigenpair")
                    .context("retained eigenpair required")?;
                let matrix = manifests
                    .iter()
                    .find(|m| m.key.kind == "ccm_tau_matrix")
                    .context("retained Tau required")?;
                input.cluster_preparation_assessment = Some(
                    xc_spectral::ccm::extended_research::ClusterPreparationAssessment {
                        source_eigenpair: state.content_digest.clone(),
                        source_matrix: matrix.content_digest.clone(),
                        source_precision_bits: u32::try_from(
                            configuration["precision_bits"]
                                .as_u64()
                                .context("original precision missing")?,
                        )?,
                        requested_eigenpairs_per_sector: usize::try_from(
                            configuration["capture"]["sector_eigenpairs"]
                                .as_u64()
                                .context("original sector request missing")?,
                        )?,
                        eigenvalue_route: "complete_qr".into(),
                        origin: "declared_prior_capture_policy".into(),
                        evidence_digest: configuration_digest.clone(),
                        reason: reason.into(),
                    },
                );
            }
        }
        sources.validate_recovery_input(&input)?;
        checkpoint_save(&output.join("external-research-inputs.json"), &input)?;
        let mut preparation_evidence = serde_json::Map::new();
        for name in [
            "smooth-qualification.json",
            "prepared-research-assessment.json",
        ] {
            if output.join(name).exists() {
                let (value, digest) = frozen_json(&output.join(name))?;
                preparation_evidence.insert(
                    name.into(),
                    serde_json::json!({"sha256":digest,"value":value}),
                );
            }
        }
        let input_value = serde_json::to_value(&input)?;
        let final_input_digest = xc_cache::ContentDigest::sha256(&serde_json::to_vec(&input)?);
        let (diagnostics, affected) = prepared_diagnostics(
            &original,
            &initial_diagnostics,
            !selected.is_empty(),
            preparation_failed,
            &input_value,
            &final_input_digest,
        );
        anyhow::ensure!(
            !diagnostics.is_empty(),
            "prepared recovery has no requested diagnostics"
        );
        if let Some(expected) = input_value["finite_diagnostics"]["complex_trials"]["provenance"]
            ["qualification_sha256"]
            .as_str()
        {
            anyhow::ensure!(
                preparation_evidence
                    .get("smooth-qualification.json")
                    .and_then(|e| e["sha256"].as_str())
                    == Some(expected),
                "prepared trial qualification is absent or its exact digest differs"
            );
        }
        if let Some(report) = preparation_evidence.get("smooth-qualification.json") {
            anyhow::ensure!(
                report["value"]["source_file_sha256"] == serde_json::to_value(&target_digest)?,
                "smooth qualification belongs to a different target source"
            );
        }
        checkpoint_save(
            &output.join("input-preparation-status.json"),
            &serde_json::json!({"status":"prepared","input_sha256":prepared.digest,
                "preparer_output_file":"prepared-research-inputs.json",
                "final_input_digest":final_input_digest,
                "post_preparation_assessments":"explicit prior sector-policy provenance, when applicable",
            "source_join_validated":true,"primary_recomputed":false,"root_recomputed":false}),
        )?;
        let (status_value, status_digest) =
            frozen_json(&output.join("input-preparation-status.json"))?;
        preparation_evidence.insert(
            "input-preparation-status.json".into(),
            serde_json::json!({"sha256":status_digest,"value":status_value}),
        );
        // Verified evidence and preparation status survive managed publication
        // together with the exact final input and original journal identities.
        checkpoint_save(
            &output.join("preparation-evidence.json"),
            &preparation_evidence,
        )?;
        let capture_request = serde_json::json!({"recovery":frozen,"diagnostics":diagnostics,
            "external_input_digest":final_input_digest,"affected_diagnostics":affected,
            "automatic_refresh_applied":selected.is_empty(),"preparation_evidence":preparation_evidence,
            "original_claim_verdict_unchanged":true});
        checkpoint_save(&output.join("capture-request.json"), &capture_request)?;
        let mut records = Vec::new();
        for (index, id) in diagnostics.iter().enumerate() {
            records.push(checkpoint_diagnostic(
                &output,
                index,
                id,
                &capture_request,
                &cache,
                || sources.capture_recovery_outcome(id, &input, &parents, &cache),
            )?);
        }
        let combined = merge_diagnostic_records(&capture_request, &diagnostics, &records)?;
        let record = xc_cache::persist_capture(&combined, &cache)?;
        let aggregate = output.join("capture-attempts");
        fs::create_dir_all(&aggregate)?;
        let digest = xc_cache::ContentDigest::sha256_json(&record.value)?;
        checkpoint_save(&aggregate.join(format!("{}.json", digest.0)), &record.value)?;
        recovery_replace(&output.join("capture.json"), &record.value)?;
        let (export, export_digest) = checkpoint_capture_export(&aggregate, &digest, |path| {
            save_reviewed_values(path, &record.value, &cache)
        })?;
        recovery_replace_file(&output.join("capture-values.json"), &export, &export_digest)?;
        verify_original_inventory(&journal, &frozen)?;
        let accounted = original.receipt.outcomes().iter().all(|(id, outcome)| {
            matches!(outcome, xc_core::DiagnosticOutcome::Completed { .. })
                || matches!(
                    record.value.receipt.outcomes().get(id),
                    Some(xc_core::DiagnosticOutcome::Completed { .. })
                )
        });
        let refreshed = affected.keys().all(|id| {
            matches!(
                record.value.receipt.outcomes().get(id),
                Some(xc_core::DiagnosticOutcome::Completed { .. })
            )
        });
        recovery_replace(
            &output.join("status.json"),
            &serde_json::json!({"schema_version":1,"capture_complete":record.value.receipt.is_complete(),
            "all_original_requests_accounted":accounted,"primary_recomputed":false,"root_recomputed":false,
            "affected_refresh_complete":refreshed,"affected_diagnostics":affected,
            "original_claim_receipt_unchanged":true,"numerical_coverage":record.value.numerical_coverage,
            "scope":"supplemental retained-source acquisition; inspect qualified numerical outcomes; original claim verdict unchanged"}),
        )?;
        let mut completed_files = serde_json::Map::new();
        for name in [
            "capture.json",
            "capture-values.json",
            "capture-request.json",
            "external-research-inputs.json",
            "preparation-evidence.json",
            "input-preparation-status.json",
        ] {
            completed_files.insert(
                name.into(),
                serde_json::to_value(bounded_file_digest(&output.join(name))?)?,
            );
        }
        recovery_replace(
            &output.join("capture-completed.json"),
            &serde_json::json!({"schema_version":1,"files":completed_files,
                "staged_drafts":staged_draft_inventory(&managed.staged_drafts()?)?}),
        )?;
        // The acquisition receipt is durable before publication. An interrupted
        // or failed publication can now be retried without captures/preparation.
        recovery_replace(
            &output.join("publication-status.json"),
            &serde_json::json!({"status":"started","capture_recomputed":false}),
        )?;
        managed.finalize_publication_inventory()?;
        recovery_replace(
            &output.join("publication-status.json"),
            &serde_json::json!({"status":"finalized","capture_recomputed":false}),
        )?;
        anyhow::ensure!(
            record.value.receipt.is_complete(),
            "supplemental recovery incomplete; inspect the retained receipt"
        );
        Ok(())
    })();
    if let Err(error) = &result {
        recovery_replace(
            &output.join("last-error.json"),
            &serde_json::json!({"failure":CaptureFailure::failed(error)}),
        )?;
        if !output.join("status.json").exists() {
            recovery_replace(
                &output.join("status.json"),
                &serde_json::json!({"schema_version":1,"capture_complete":false,"primary_recomputed":false,
                "root_recomputed":false,"original_claim_receipt_unchanged":true,"error":format!("{error:#}")}),
            )?;
        }
    }
    println!("Supplemental recovery journal: {}", output.display());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_committed_generation_transition_refuses_stale_completion() {
        let root = std::env::temp_dir().join(format!(
            "paper-capture-generation-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let plan = serde_json::json!({"fixture":"committed incomplete supplement"});
        let incomplete = xc_cache::collect_capture(&plan, vec!["energy_allowance".into()], |_| {
            Err(CaptureFailure::Blocked {
                reason: "fixture resource limit".into(),
            })
        })
        .unwrap();
        assert!(!incomplete.receipt.is_complete());
        checkpoint_save(&root.join("capture.json"), &incomplete).unwrap();
        checkpoint_save(&root.join("capture-request.json"), &plan).unwrap();
        let mut files = serde_json::Map::new();
        for name in [
            "capture-values.json",
            "external-research-inputs.json",
            "preparation-evidence.json",
            "input-preparation-status.json",
        ] {
            checkpoint_save(
                &root.join(name),
                &serde_json::json!({"fixture":"retained generation bytes"}),
            )
            .unwrap();
        }
        for name in [
            "capture.json",
            "capture-request.json",
            "capture-values.json",
            "external-research-inputs.json",
            "preparation-evidence.json",
            "input-preparation-status.json",
        ] {
            files.insert(
                name.into(),
                serde_json::to_value(bounded_file_digest(&root.join(name)).unwrap()).unwrap(),
            );
        }
        let completion = serde_json::json!({"schema_version":1,"files":files,
            "staged_drafts":staged_draft_inventory(&[]).unwrap()});
        checkpoint_save(&root.join("capture-completed.json"), &completion).unwrap();
        let before = fs::read(root.join("capture-completed.json")).unwrap();
        verify_completed_capture(&root, &completion).unwrap();
        // A new valid measurement can become durable before the new aggregate
        // completion is committed. Its old completion cannot authorize it.
        let completed = xc_cache::collect_capture(&plan, vec!["energy_allowance".into()], |_| {
            xc_cache::CapturedDiagnostic::new(
                &serde_json::json!({"value":"1","outcome":"point_measurement"}),
                vec![],
            )
            .map_err(CaptureFailure::failed)
        })
        .unwrap();
        assert!(completed.receipt.is_complete());
        recovery_replace(&root.join("capture.json"), &completed).unwrap();
        assert!(verify_completed_capture(&root, &completion).is_err());
        assert_eq!(
            fs::read(root.join("capture-completed.json")).unwrap(),
            before
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn original_optional_inputs_bind_absence_presence_and_admitted_bytes() {
        let root = std::env::temp_dir().join(format!(
            "paper-original-inventory-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let mut inventory = serde_json::json!({});
        for &name in ORIGINAL_REQUIRED_FILES {
            checkpoint_save(&root.join(name), &serde_json::json!({"fixture":name})).unwrap();
            inventory[name] =
                serde_json::to_value(frozen_json(&root.join(name)).unwrap().1).unwrap();
        }
        let (optional, inputs) = optional_original_inputs(&root).unwrap();
        assert!(optional.is_empty() && inputs.is_empty());
        assert!(verify_original_inventory(&root, &inventory).is_ok());
        for &name in ORIGINAL_OPTIONAL_FILES {
            checkpoint_save(&root.join(name), &serde_json::json!({"late":name})).unwrap();
            assert!(
                verify_original_inventory(&root, &inventory).is_err(),
                "unbound optional file: {name}"
            );
            assert!(!inputs.contains_key(name));
            fs::remove_file(root.join(name)).unwrap();
        }
        let status = root.join("input-preparation-status.json");
        checkpoint_save(&status, &serde_json::json!({"status":"failed"})).unwrap();
        let (optional, inputs) = optional_original_inputs(&root).unwrap();
        inventory["input-preparation-status.json"] =
            optional["input-preparation-status.json"].clone();
        fs::write(&status, b"{\"status\":\"prepared\"}\n").unwrap();
        assert_eq!(inputs["input-preparation-status.json"]["status"], "failed");
        assert!(verify_original_inventory(&root, &inventory).is_err());
        fs::remove_file(status).unwrap();
        assert!(verify_original_inventory(&root, &inventory).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn aggregate_export_checkpoint_rejects_corruption_and_regenerates_only_provisional_exports() {
        use std::cell::Cell;
        let root = std::env::temp_dir().join(format!(
            "paper-export-checkpoint-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let capture = xc_cache::ContentDigest::sha256(b"exact capture");
        let calls = Cell::new(0usize);
        let produce = |path: &Path| {
            calls.set(calls.get() + 1);
            checkpoint_bytes(path, b"{\"validated\":true}\n")
        };
        let (export, digest) = checkpoint_capture_export(&root, &capture, produce).unwrap();
        checkpoint_capture_export(&root, &capture, |_| panic!("committed export regenerated"))
            .unwrap();
        assert_eq!(calls.get(), 1);
        let bytes = fs::read(&export).unwrap();
        fs::write(&export, b"{\"validated\":false}\n").unwrap();
        assert!(checkpoint_capture_export(&root, &capture, |_| panic!(
            "corrupt export regenerated"
        ))
        .is_err());
        fs::write(&export, &bytes).unwrap();
        assert_eq!(bounded_file_digest(&export).unwrap(), digest);
        let checkpoint = root.join(format!("{}.values-checkpoint.json", capture.0));
        fs::remove_file(&checkpoint).unwrap();
        checkpoint_capture_export(&root, &capture, produce).unwrap();
        assert_eq!(calls.get(), 2);
        fs::remove_file(&checkpoint).unwrap();
        fs::write(&export, b"corrupted provisional export").unwrap();
        assert!(checkpoint_capture_export(&root, &capture, produce).is_err());
        assert_eq!(calls.get(), 3);
        assert!(!checkpoint.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn arithmetic_preparation_checkpoint_skips_compute_and_rejects_changed_identity_or_bytes() {
        use std::cell::Cell;
        let root = std::env::temp_dir().join(format!(
            "paper-recovery-base-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let identity = serde_json::json!({"fixture":"exact recovery identity"});
        let request = serde_json::json!({"fixture":"exact arithmetic prerequisites"});
        let calls = Cell::new(0usize);
        let validations = Cell::new(0usize);
        let prepare = || {
            calls.set(calls.get() + 1);
            Ok(request.clone())
        };
        let validate = |value: &serde_json::Value| {
            validations.set(validations.get() + 1);
            anyhow::ensure!(*value == request, "source join differs");
            Ok(())
        };
        assert_eq!(
            checkpoint_preparation_request(&root, &identity, prepare, validate).unwrap(),
            request
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(
            checkpoint_preparation_request(
                &root,
                &identity,
                || panic!("completed arithmetic preparation recomputed"),
                validate
            )
            .unwrap(),
            request
        );
        assert_eq!(validations.get(), 2);
        assert!(checkpoint_preparation_request(
            &root,
            &serde_json::json!({"changed":true}),
            || panic!("changed identity recomputed"),
            validate
        )
        .is_err());
        let request_path = root.join("recovery-preparation-request.json");
        let request_bytes = fs::read(&request_path).unwrap();
        fs::write(&request_path, b"{\"changed\":true}\n").unwrap();
        assert!(checkpoint_preparation_request(
            &root,
            &identity,
            || panic!("changed request recomputed"),
            validate
        )
        .is_err());
        fs::write(&request_path, request_bytes).unwrap();
        assert!(checkpoint_preparation_request(
            &root,
            &identity,
            || panic!("invalid source join recomputed"),
            |_| anyhow::bail!("source join differs")
        )
        .is_err());
        // A crash after the provisional request but before its checkpoint
        // cannot authorize reuse. Compute again, then require exact old bytes.
        let orphan = root.join("orphan-request");
        fs::create_dir(&orphan).unwrap();
        checkpoint_save(&orphan.join("recovery-preparation-request.json"), &request).unwrap();
        assert_eq!(
            checkpoint_preparation_request(&orphan, &identity, prepare, validate).unwrap(),
            request
        );
        assert_eq!(calls.get(), 2);
        assert!(orphan
            .join("recovery-preparation-checkpoint.json")
            .is_file());
        fs::write(root.join("recovery-preparation-checkpoint.json"), b"{").unwrap();
        assert!(checkpoint_preparation_request(
            &root,
            &identity,
            || panic!("malformed checkpoint recomputed"),
            validate
        )
        .is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recovery_streamed_hash_and_copy_remain_bounded_and_reject_corruption() {
        struct SyntheticReader {
            remaining: usize,
            largest_request: usize,
        }
        impl std::io::Read for SyntheticReader {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                self.largest_request = self.largest_request.max(buffer.len());
                let size = self.remaining.min(buffer.len());
                buffer[..size].fill(b'x');
                self.remaining -= size;
                Ok(size)
            }
        }
        struct Counter(usize);
        impl Write for Counter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0 += bytes.len();
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let size = 3 * COPY_BUFFER_BYTES + 17;
        let mut source = SyntheticReader {
            remaining: size,
            largest_request: 0,
        };
        let mut sink = Counter(0);
        let (digest, copied) = stream_copy_digest(&mut source, &mut sink, size as u64).unwrap();
        assert_eq!(digest, xc_cache::ContentDigest::sha256(&vec![b'x'; size]));
        assert_eq!(copied, size as u64);
        assert_eq!(sink.0, size);
        assert!(source.largest_request <= COPY_BUFFER_BYTES);
        let mut limited = SyntheticReader {
            remaining: size,
            largest_request: 0,
        };
        let mut limited_sink = Counter(0);
        assert!(
            stream_copy_digest(&mut limited, &mut limited_sink, COPY_BUFFER_BYTES as u64).is_err()
        );
        assert!(limited_sink.0 <= COPY_BUFFER_BYTES);

        let root = std::env::temp_dir().join(format!(
            "paper-recovery-stream-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let source = root.join("source");
        let target = root.join("target");
        let bytes = vec![b'x'; size];
        fs::write(&source, &bytes).unwrap();
        let digest = bounded_file_digest(&source).unwrap();
        checkpoint_file(&target, &source, &digest).unwrap();
        assert_eq!(fs::read(&target).unwrap(), bytes);
        fs::write(&source, vec![b'y'; size]).unwrap();
        assert!(checkpoint_file(&root.join("bad"), &source, &digest).is_err());
        assert!(!root.join("bad").exists());
        assert!(recovery_replace_file(&target, &source, &digest).is_err());
        assert_eq!(fs::read(&target).unwrap(), bytes);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recovery_staging_inventory_binds_appended_and_changed_drafts() {
        use xc_cache::*;
        let root = std::env::temp_dir().join(format!(
            "paper-recovery-staging-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let semantic = SemanticKeyEnvelope {
            schema_version: 1,
            artifact_kind: "ccm_tau_matrix".into(),
            mathematical_semantics_version: "retained-recovery-fixture-v1".into(),
            resolved_mathematical_parameters: serde_json::json!({"precision_bits":128}),
            normalization: None,
            target: None,
            subspace: None,
            source_data_identities: Default::default(),
            algorithm_semantics: None,
        };
        let payload = b"{\"fixture\":1}".to_vec();
        let key = ArtifactKey {
            kind: semantic.artifact_kind.clone(),
            logical_key: "fixture".into(),
            parameters_digest: semantic.digest().unwrap(),
        };
        let store = ZipJsonFilesystemCacheStore::new(
            "fixture",
            root.join("cache"),
            true,
            CacheVisibility::Local,
        );
        let manifest = store
            .put(
                &ArtifactDraft {
                    schema_version: 1,
                    key,
                    producer_toolkit_version: current_toolkit_version().unwrap(),
                    minimum_reader_version: ToolkitVersion::parse("0.16.0").unwrap(),
                    maximum_reader_version: None,
                    quality: CacheQuality::Validated,
                    visibility: CacheVisibility::Local,
                    immutable: true,
                    dependencies: vec![],
                    tags: Default::default(),
                    provenance_digest: None,
                },
                &payload,
            )
            .unwrap();
        let draft = stage_produced_artifact(
            &ProducedArtifactRecord {
                operation: "fixture".into(),
                semantic_key: semantic,
                logical_key: "fixture".into(),
                manifest,
                payload,
                achieved_assurance: ArtifactAssuranceState::Computed,
                assurance_evidence_digests: vec![],
            },
            &root.join("staging"),
            &TransportPolicy::default(),
            &xc_core::ResourcePolicy::default(),
            &xc_core::CancellationToken::new(),
        )
        .unwrap();
        let expected = staged_draft_inventory(std::slice::from_ref(&draft)).unwrap();
        verify_staged_draft_inventory(&expected, std::slice::from_ref(&draft)).unwrap();
        assert!(verify_staged_draft_inventory(&expected, &[draft.clone(), draft.clone()]).is_err());
        let mut changed = draft.clone();
        changed.source_operation = "different producer operation".into();
        assert!(verify_staged_draft_inventory(&expected, &[changed.clone()]).is_err());
        let pair = staged_draft_inventory(&[draft.clone(), changed.clone()]).unwrap();
        verify_staged_draft_inventory(&pair, &[changed, draft]).unwrap();
        assert!(verify_staged_draft_inventory(&serde_json::Value::Null, &[]).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn merged_checkpoints_preserve_capture_identity_and_fail_closed() {
        let plan = serde_json::json!({"fixture":"per-diagnostic durability"});
        let diagnostics = vec!["a".to_owned(), "b".to_owned()];
        let capture = |id: &str| {
            xc_cache::CapturedDiagnostic::new(
                &serde_json::json!({"value":id,"outcome":"point_measurement"}),
                vec![],
            )
            .map_err(CaptureFailure::failed)
        };
        let expected = xc_cache::collect_capture(&plan, diagnostics.clone(), capture).unwrap();
        let records: Vec<_> = diagnostics
            .iter()
            .map(|id| xc_cache::collect_capture(&plan, vec![id.clone()], capture).unwrap())
            .collect();
        let merged = merge_diagnostic_records(&plan, &diagnostics, &records).unwrap();
        assert_eq!(
            serde_json::to_vec(&merged).unwrap(),
            serde_json::to_vec(&expected).unwrap()
        );
        assert!(merge_diagnostic_records(
            &serde_json::json!({"changed":true}),
            &diagnostics,
            &records
        )
        .is_err());
        let mut corrupt = records.clone();
        corrupt[0].measurements.get_mut("a").unwrap().value = serde_json::json!({"changed":true});
        assert!(merge_diagnostic_records(&plan, &diagnostics, &corrupt).is_err());
    }

    fn original_capture() -> xc_cache::CaptureArtifact {
        xc_cache::collect_capture(
            &serde_json::json!({"fixture":"frozen prior capture"}),
            vec!["compactness".into(), "energy_allowance".into()],
            |id| {
                if id == "energy_allowance" {
                    Err(xc_cache::CaptureFailure::Missing {
                        reason: "no declared block bounds".into(),
                    })
                } else {
                    xc_cache::CapturedDiagnostic::new(
                        &serde_json::json!({"value":"1","outcome":"point_measurement"}),
                        vec![],
                    )
                    .map_err(xc_cache::CaptureFailure::failed)
                }
            },
        )
        .unwrap()
    }

    #[test]
    fn recovery_defaults_to_unfinished_and_does_not_mutate_old_receipt() {
        let record = original_capture();
        let before = serde_json::to_vec(&record).unwrap();
        assert_eq!(
            recovery_diagnostics(&record, &[]).unwrap(),
            ["energy_allowance"]
        );
        assert_eq!(serde_json::to_vec(&record).unwrap(), before);
        assert!(recovery_diagnostics(
            &record,
            &["energy_allowance".into(), "energy_allowance".into()]
        )
        .is_err());
        assert!(recovery_diagnostics(&record, &["root_certificate".into()]).is_err());
        assert_eq!(
            recovery_diagnostics(&record, &["compactness".into()]).unwrap(),
            ["compactness"]
        );
    }

    #[test]
    fn recovery_refuses_inconsistent_completed_inventory() {
        let mut record = original_capture();
        record.measurements.remove("compactness");
        assert!(recovery_diagnostics(&record, &[]).is_err());
    }

    #[test]
    fn successful_preparation_refreshes_exactly_the_requested_affected_finite_groups() {
        let affected = [
            "trial_vector_energy",
            "trial_vector_parity",
            "continuous_l1_bound",
            "normalization_error_bound",
            "finite_tail_bound",
        ];
        let make_record = |omit: Option<&str>| {
            let ids = std::iter::once("energy_allowance")
                .chain(affected)
                .chain(["compactness"])
                .filter(|id| Some(*id) != omit)
                .map(str::to_owned)
                .collect::<Vec<_>>();
            xc_cache::collect_capture(
                &serde_json::json!({"fixture":"old failed preparation"}),
                ids,
                |id| {
                    if id == "energy_allowance" {
                        Err(CaptureFailure::Missing {
                            reason: "declared bounds absent".into(),
                        })
                    } else {
                        xc_cache::CapturedDiagnostic::new(
                            &serde_json::json!({"fixture":id}),
                            vec![],
                        )
                        .map_err(CaptureFailure::failed)
                    }
                },
            )
            .unwrap()
        };
        let record = make_record(None);
        let before = serde_json::to_vec(&record).unwrap();
        let initial = recovery_diagnostics(&record, &[]).unwrap();
        let input = serde_json::json!({"source_eigenpair":"b".repeat(64),"finite_diagnostics":{
            "complex_trials":{},"continuous_energy":{},"projection_energy":{}}});
        let digest = xc_cache::ContentDigest::sha256(b"new exact input fixture");
        let (selected, reasons) =
            prepared_diagnostics(&record, &initial, false, true, &input, &digest);
        assert_eq!(selected.len(), 6);
        for id in affected {
            assert!(selected.iter().any(|name| name == id));
        }
        assert!(!selected.iter().any(|name| name == "compactness"));
        assert_eq!(reasons.len(), 5);
        assert_eq!(
            reasons["trial_vector_energy"]["external_input_digest"],
            serde_json::to_value(&digest).unwrap()
        );
        assert_eq!(serde_json::to_vec(&record).unwrap(), before);
        assert_eq!(
            prepared_diagnostics(&record, &initial, true, true, &input, &digest).0,
            initial
        );
        assert_eq!(
            prepared_diagnostics(&record, &initial, false, false, &input, &digest).0,
            initial
        );
        assert_eq!(
            prepared_diagnostics(
                &record,
                &initial,
                false,
                true,
                &serde_json::json!({}),
                &digest
            )
            .0,
            initial
        );
        let limited = make_record(Some("continuous_l1_bound"));
        let (selected, reasons) =
            prepared_diagnostics(&limited, &initial, false, true, &input, &digest);
        assert!(!selected.iter().any(|name| name == "continuous_l1_bound"));
        assert!(!reasons.contains_key("continuous_l1_bound"));
    }

    #[test]
    fn recovery_target_and_original_source_join_fail_closed() {
        let digest = xc_cache::ContentDigest::sha256(b"exact original target");
        let configuration = serde_json::json!({"research_inputs":{"XC_TARGET_SPEC_FILE":{"status":"supplied","sha256":digest}}});
        let mut request = serde_json::json!({"schema_version":1,"configuration":configuration,"primary_sources":[],
            "primary_recomputation":false,"output_visibility":"private_only"});
        assert!(validate_recovery_request(&request, &configuration, &[], &digest).is_ok());
        assert!(validate_recovery_request(
            &request,
            &configuration,
            &[],
            &xc_cache::ContentDigest::sha256(b"changed target")
        )
        .is_err());
        request["primary_sources"] = serde_json::json!([{"untrusted":"substitution"}]);
        assert!(validate_recovery_request(&request, &configuration, &[], &digest).is_err());
        request["primary_sources"] = serde_json::json!([]);
        request["primary_recomputation"] = serde_json::json!(true);
        assert!(validate_recovery_request(&request, &configuration, &[], &digest).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn tiny_retained_recovery_is_additive_and_source_misses_never_solve() {
        use xc_cache::*;
        let root = std::env::temp_dir().join(format!(
            "paper-retained-recovery-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let original = root.join("original");
        fs::create_dir(&original).unwrap();
        let cache_root = root.join("cache");
        let store =
            ZipJsonFilesystemCacheStore::new("fixture", &cache_root, true, CacheVisibility::Local);
        let put = |kind: &str, value: serde_json::Value, dependencies: Vec<DependencyRef>| {
            store
                .put(
                    &ArtifactDraft {
                        schema_version: 1,
                        key: ArtifactKey::new(
                            kind,
                            "tiny-source-only-recovery-fixture",
                            kind.as_bytes(),
                        )
                        .unwrap(),
                        producer_toolkit_version: ToolkitVersion::parse("0.17.0").unwrap(),
                        minimum_reader_version: ToolkitVersion::parse("0.16.0").unwrap(),
                        maximum_reader_version: None,
                        quality: CacheQuality::Validated,
                        visibility: CacheVisibility::Local,
                        immutable: true,
                        dependencies,
                        tags: Default::default(),
                        provenance_digest: None,
                    },
                    &serde_json::to_vec(&value).unwrap(),
                )
                .unwrap()
        };
        // Hand-authored exact dyadic fixtures: no primary or root solve occurs.
        let matrix = put(
            "ccm_tau_matrix",
            serde_json::json!({"schema_version":2,"lambda_squared":"5","n_modes":1,
            "precision_bits":128,"entries":["1","0","0","0","2","0","0","0","3"]}),
            vec![],
        );
        let state = put(
            "ccm_weil_eigenpair",
            serde_json::json!({"schema_version":3,"lambda_squared":"5","n_modes":1,
            "precision_bits":128,"force_even":true,"eigenvalue":"2","eigenvector":["0","1","0"]}),
            vec![DependencyRef {
                key: matrix.key.clone(),
                content_digest: matrix.content_digest.clone(),
                required_quality: CacheQuality::Validated,
            }],
        );
        let manifests = vec![state, matrix];
        let target = root.join("target.json");
        save(
            &target,
            &serde_json::json!({"fixture":"exact retained target bytes"}),
        )
        .unwrap();
        let target_digest = frozen_json(&target).unwrap().1;
        let configuration = serde_json::json!({"schema_version":1,"precision_bits":128,"capture":{"sector_eigenpairs":2},
            "research_inputs":{"XC_TARGET_SPEC_FILE":{"status":"supplied","sha256":target_digest}}});
        save(&original.join("request.json"), &configuration).unwrap();
        save(&original.join("input-preparation-request.json"),&serde_json::json!({"schema_version":1,"configuration":configuration,
            "primary_sources":manifests,"base_inputs":null,"target_spec_file":target,"output_visibility":"private_only","primary_recomputation":false})).unwrap();
        save(&original.join("primary-sources.json"), &manifests).unwrap();
        let capture = xc_cache::collect_capture(
            &serde_json::json!({"fixture":"prior missing input"}),
            vec!["energy_allowance".into()],
            |_| {
                Err(CaptureFailure::Missing {
                    reason: "private preparation failed".into(),
                })
            },
        )
        .unwrap();
        save(&original.join("capture.json"), &capture).unwrap();
        let old_bytes: Vec<_> = [
            "request.json",
            "input-preparation-request.json",
            "primary-sources.json",
            "capture.json",
        ]
        .iter()
        .map(|name| (name.to_string(), fs::read(original.join(name)).unwrap()))
        .collect();
        let config = ManagedArtifactCacheConfig {
            profile: ManagedRunProfile::Normal,
            requested_assurance: xc_core::AssuranceLevel::Computed,
            certification_failure_policy: CertificationFailurePolicy::RetainComputedFailRun,
            cache_root: cache_root.clone(),
            staging_root: None,
            publication_target: xc_core::PublicationTarget::None,
            repository_owner: "TeamXcelerator".into(),
            remote_cache_mode: ManagedRemoteCacheMode::None,
            cache_mode: ArtifactExecutionCacheMode::PreferReuse,
            replace_existing_publication: false,
            execute_remote_mutations: false,
            output_validation: None,
        };
        let preparer = serde_json::json!({"argv":["python3","-c",
            "import json,pathlib,sys; marker=pathlib.Path(sys.argv[2]).parents[2]/'preparer-ran'; assert not marker.exists(), 'preparation reran'; marker.touch(); r=json.loads(pathlib.Path(sys.argv[2]).read_text()); pathlib.Path(sys.argv[4]).write_text(json.dumps(r['base_inputs']))"],"timeout_seconds":10}).to_string();
        let output = root.join("supplement");
        recover_capture_configured(&original, &output, None, &[], &preparer, config.clone())
            .unwrap();
        let status = frozen_json(&output.join("status.json")).unwrap().0;
        assert_eq!(status["capture_complete"], true);
        assert_eq!(status["primary_recomputed"], false);
        assert_eq!(status["root_recomputed"], false);
        assert_eq!(status["all_original_requests_accounted"], true);
        let preparation = frozen_json(&output.join("input-preparation-status.json")).unwrap();
        let capture = frozen_json(&output.join("capture.json")).unwrap().0;
        assert_eq!(
            capture["resolved_plan"]["preparation_evidence"]["input-preparation-status.json"]
                ["sha256"],
            serde_json::to_value(preparation.1).unwrap()
        );
        assert_eq!(
            capture["resolved_plan"]["preparation_evidence"]["input-preparation-status.json"]
                ["value"],
            preparation.0
        );
        let first_status = fs::read(output.join("status.json")).unwrap();
        assert!(recover_capture_configured(
            &original,
            &output,
            None,
            &[],
            &preparer,
            config.clone()
        )
        .is_err());
        assert_eq!(fs::read(output.join("status.json")).unwrap(), first_status);
        // Simulate interruption after the completed diagnostic was synced but
        // before canonical aggregate/status installation. No preparer reruns.
        let diagnostic_before =
            fs::read(output.join("diagnostic-checkpoints/000/00.json")).unwrap();
        for name in [
            "capture.json",
            "capture-values.json",
            "capture-completed.json",
            "status.json",
        ] {
            fs::remove_file(output.join(name)).unwrap();
        }
        recover_capture_configured_mode(
            &original,
            &output,
            None,
            &[],
            &preparer,
            config.clone(),
            true,
            false,
        )
        .unwrap();
        assert_eq!(
            fs::read(output.join("diagnostic-checkpoints/000/00.json")).unwrap(),
            diagnostic_before
        );
        assert!(!output.join("diagnostic-checkpoints/000/01.json").exists());
        assert!(!output.join("preparation-attempts/01").exists());
        let saved_capture = fs::read(output.join("capture.json")).unwrap();
        // Publication-only must reach the strict retained publisher without
        // executing this fixture's preparer. Its nonauthor/no-target policy is
        // intentionally rejected; the capture stays byte-identical.
        assert!(recover_capture_configured_mode(
            &original,
            &output,
            None,
            &[],
            &preparer,
            config.clone(),
            false,
            true
        )
        .is_err());
        assert_eq!(
            fs::read(output.join("capture.json")).unwrap(),
            saved_capture
        );
        assert!(!output.join("preparation-attempts/01").exists());
        // Resume authenticates the prior committed staging set before it can
        // replace that inventory with newly produced legitimate work.
        let completion_path = output.join("capture-completed.json");
        let completion_bytes = fs::read(&completion_path).unwrap();
        let mut completion = frozen_json(&completion_path).unwrap().0;
        completion["staged_drafts"]["draft_count"] = serde_json::json!(1);
        fs::write(&completion_path, serde_json::to_vec(&completion).unwrap()).unwrap();
        let changed_completion = fs::read(&completion_path).unwrap();
        let error = recover_capture_configured_mode(
            &original,
            &output,
            None,
            &[],
            &preparer,
            config.clone(),
            true,
            false,
        )
        .unwrap_err();
        assert!(error.to_string().contains("staging inventory"));
        assert_eq!(fs::read(&completion_path).unwrap(), changed_completion);
        fs::write(&completion_path, &completion_bytes).unwrap();
        // A valid previous completion cannot bless a modified archived export
        // when resuming. Canonical reviewed bytes and completion stay intact.
        let retained_record: xc_cache::CaptureArtifact =
            serde_json::from_value(frozen_json(&output.join("capture.json")).unwrap().0).unwrap();
        let capture_digest = xc_cache::ContentDigest::sha256_json(&retained_record).unwrap();
        let archived_export = output
            .join("capture-attempts")
            .join(format!("{}.values.json", capture_digest.0));
        let archived_bytes = fs::read(&archived_export).unwrap();
        let canonical_values = fs::read(output.join("capture-values.json")).unwrap();
        fs::write(&archived_export, b"{\"corrupted\":true}\n").unwrap();
        let error = recover_capture_configured_mode(
            &original,
            &output,
            None,
            &[],
            &preparer,
            config.clone(),
            true,
            false,
        )
        .unwrap_err();
        assert!(error.to_string().contains("aggregate export"));
        assert_eq!(
            fs::read(output.join("capture-values.json")).unwrap(),
            canonical_values
        );
        assert_eq!(fs::read(&completion_path).unwrap(), completion_bytes);
        fs::write(archived_export, archived_bytes).unwrap();
        // Changed selection or prepared bytes must fail before recomputation.
        assert!(recover_capture_configured_mode(
            &original,
            &output,
            None,
            &["energy_allowance".into()],
            &preparer,
            config.clone(),
            true,
            false
        )
        .is_err());
        let prepared_path = output.join("preparation-attempts/00/prepared-research-inputs.json");
        let prepared_bytes = fs::read(&prepared_path).unwrap();
        fs::write(&prepared_path, b"{\"changed\":true}").unwrap();
        assert!(recover_capture_configured_mode(
            &original,
            &output,
            None,
            &[],
            &preparer,
            config.clone(),
            true,
            false
        )
        .is_err());
        assert_eq!(
            fs::read(output.join("capture.json")).unwrap(),
            saved_capture
        );
        fs::write(prepared_path, prepared_bytes).unwrap();
        // A completed checkpoint never calls its supplied numerical callback.
        let session = ManagedArtifactCacheSession::new(config.clone()).unwrap();
        let plan = frozen_json(&output.join("capture-request.json")).unwrap().0;
        checkpoint_diagnostic(
            &output,
            0,
            "energy_allowance",
            &plan,
            &session.context(),
            || panic!("completed diagnostic recomputed"),
        )
        .unwrap();
        // A malformed immutable checkpoint fails closed and remains untouched.
        let diagnostic_path = output.join("diagnostic-checkpoints/000/00.json");
        fs::write(&diagnostic_path, b"{").unwrap();
        assert!(checkpoint_diagnostic(
            &output,
            0,
            "energy_allowance",
            &plan,
            &session.context(),
            || panic!("corrupt checkpoint recomputed")
        )
        .is_err());
        fs::write(diagnostic_path, diagnostic_before).unwrap();
        drop(session);
        for (name, bytes) in &old_bytes {
            assert_eq!(&fs::read(original.join(name)).unwrap(), bytes);
        }
        let mut forbidden = config.clone();
        forbidden.publication_target = xc_core::PublicationTarget::Public;
        assert!(recover_capture_configured(
            &original,
            &root.join("public-refused"),
            None,
            &[],
            &preparer,
            forbidden
        )
        .is_err());
        let mut forbidden = config.clone();
        forbidden.replace_existing_publication = true;
        assert!(recover_capture_configured(
            &original,
            &root.join("replace-refused"),
            None,
            &[],
            &preparer,
            forbidden
        )
        .is_err());
        let mut tampered = manifests.clone();
        tampered[0].size_bytes += 1;
        fs::write(
            original.join("primary-sources.json"),
            serde_json::to_vec(&tampered).unwrap(),
        )
        .unwrap();
        let mut request = frozen_json(&original.join("input-preparation-request.json"))
            .unwrap()
            .0;
        request["primary_sources"] = serde_json::to_value(&tampered).unwrap();
        fs::write(
            original.join("input-preparation-request.json"),
            serde_json::to_vec(&request).unwrap(),
        )
        .unwrap();
        assert!(recover_capture_configured(
            &original,
            &root.join("tampered-source"),
            None,
            &[],
            &preparer,
            config.clone()
        )
        .is_err());
        for (name, bytes) in &old_bytes {
            fs::write(original.join(name), bytes).unwrap();
        }
        // Remove only this fixture's cache. No fallback may reconstruct the state.
        drop(store);
        fs::remove_dir_all(&cache_root).unwrap();
        assert!(recover_capture_configured(
            &original,
            &root.join("missing-sources"),
            None,
            &[],
            &preparer,
            config
        )
        .is_err());
        assert_eq!(
            frozen_json(&root.join("missing-sources/status.json"))
                .unwrap()
                .0["primary_recomputed"],
            false
        );
        fs::remove_dir_all(root).unwrap();
    }
}
