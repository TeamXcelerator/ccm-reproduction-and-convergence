//! Optional retained-source analysis. Ordinary claim commands never call this module.
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
};
use xc_cache::{
    ArtifactManifest, ContentDigest, ManagedArtifactCacheConfig, ManagedArtifactCacheSession,
};
use xc_spectral::ccm::explicit_cohort::{self, CohortPairRequest, FrozenCohortPlan};

const METADATA_LIMIT: u64 = 64 << 20;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    path: PathBuf,
    sha256: ContentDigest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestFile {
    schema_version: u32,
    manifest_files: Vec<Inventory>,
    cohort: FrozenCohortPlan,
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    ensure!(
        file.metadata()?.len() <= limit,
        "metadata exceeds admitted bytes: {}",
        path.display()
    );
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "metadata grew beyond admitted bytes"
    );
    Ok(bytes)
}

fn save_new(path: &Path, value: &impl Serialize) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("preserving existing output {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    Ok(())
}

/// Freeze only the specified inventories, in the supplied order. No cache search.
pub fn freeze(
    journals: &[PathBuf],
    ordinals: &[usize],
    residual: bool,
    output: &Path,
) -> Result<()> {
    ensure!(
        journals.len() >= 2 && journals.len() <= 1024,
        "select 2..1024 explicit journals"
    );
    ensure!(
        (ordinals.is_empty() || ordinals[0] > 0) && ordinals.windows(2).all(|w| w[0] < w[1]),
        "root indices must be positive and strictly increasing"
    );
    ensure!(
        !output.exists(),
        "cohort plan already exists; preserve the original plan"
    );
    let mut files = Vec::new();
    let mut members = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (index, journal) in journals.iter().enumerate() {
        let path = journal
            .join("primary-sources.json")
            .canonicalize()
            .with_context(|| format!("explicit journal inventory {}", journal.display()))?;
        ensure!(seen.insert(path.clone()), "duplicate selected journal");
        let bytes = read_bounded(&path, METADATA_LIMIT)?;
        let mut manifests: Vec<ArtifactManifest> = serde_json::from_slice(&bytes)?;
        if ordinals.is_empty() {
            manifests.retain(|m| {
                !matches!(
                    m.key.kind.as_str(),
                    "ccm_root_refinement" | "ccm_root_discovery_window"
                )
            });
        }
        members.push(explicit_cohort::freeze_member(
            format!("member-{}", index + 1),
            &manifests,
        )?);
        files.push(Inventory {
            path,
            sha256: ContentDigest::sha256(&bytes),
        });
    }
    let pairs = members
        .windows(2)
        .map(|p| CohortPairRequest {
            left: p[0].id.clone(),
            right: p[1].id.clone(),
            root_ordinals: ordinals.to_vec(),
            embedded_residual: residual,
        })
        .collect();
    let request = RequestFile {
        schema_version: 1,
        manifest_files: files,
        cohort: FrozenCohortPlan {
            schema_version: 1,
            members,
            pairs,
        },
    };
    request.cohort.validate()?;
    save_new(output, &request)?;
    println!("Frozen cohort plan: {}", output.display());
    println!("Metadata selection only; source authentication and numerical comparisons occur during cohort execution.");
    Ok(())
}

pub fn execute(plan: &Path, output: &Path, defer_publication: bool) -> Result<()> {
    ensure!(
        !output.exists(),
        "cohort output already exists; use a new directory"
    );
    let policy = xc_spectral::ccm::capture_runtime::CaptureResourcePolicy::from_environment()?;
    let resources = xc_core::RunResources::new(policy.maximum_working_bytes)?;
    let _scope = resources.enter();
    let plan_limit = METADATA_LIMIT.min(policy.maximum_working_bytes / 16);
    let bytes = read_bounded(plan, plan_limit)?;
    let _plan_admission = resources.try_reserve_working(
        (bytes.len() as u64)
            .saturating_mul(8)
            .saturating_add(1 << 20),
    )?;
    let request_digest = ContentDigest::sha256(&bytes);
    let request: RequestFile = serde_json::from_slice(&bytes)?;
    ensure!(
        request.schema_version == 1
            && !request.manifest_files.is_empty()
            && request.manifest_files.len() <= 1024,
        "invalid cohort request schema or inventory count"
    );
    request.cohort.validate()?;
    let mut manifests = Vec::new();
    let mut admissions = Vec::new();
    let base = plan.parent().unwrap_or_else(|| Path::new("."));
    for inventory in &request.manifest_files {
        ensure!(inventory.sha256.validate(), "invalid inventory digest");
        let path = if inventory.path.is_absolute() {
            inventory.path.clone()
        } else {
            base.join(&inventory.path)
        };
        let bytes = read_bounded(&path, plan_limit)?;
        admissions.push(
            resources.try_reserve_working(
                (bytes.len() as u64)
                    .saturating_mul(8)
                    .saturating_add(1 << 20),
            )?,
        );
        ensure!(
            ContentDigest::sha256(&bytes) == inventory.sha256,
            "selected manifest inventory changed: {}",
            path.display()
        );
        let selected: Vec<ArtifactManifest> = serde_json::from_slice(&bytes)?;
        manifests.extend(selected);
    }
    let config = ManagedArtifactCacheConfig::from_environment()?
        .context("explicit cohort requires a configured managed cache")?;
    if defer_publication {
        ensure!(
            config.profile == xc_cache::ManagedRunProfile::Author
                && config.execute_remote_mutations
                && config.publication_target != xc_core::PublicationTarget::None
                && config.staging_root.is_some()
                && config.output_validation.is_none(),
            "deferred cohort publication requires author staging, execution, and a destination"
        );
    }
    let managed = ManagedArtifactCacheSession::new_with_resources(
        config,
        xc_cache::managed_resource_policy_from_environment()?,
    )?;
    fs::create_dir(output)?;
    save_new(&output.join("request.json"), &request)?;
    save_new(
        &output.join("build.json"),
        &serde_json::json!({
            "software_version": env!("CARGO_PKG_VERSION"), "cargo_lock": include_str!("../Cargo.lock"),
            "scope":"optional retained-source cohort; original claim verdicts unchanged"
        }),
    )?;
    let result = explicit_cohort::analyze_frozen_cohort(
        &request.cohort,
        &manifests,
        &managed.context(),
        &resources,
        &output.join("scratch"),
    );
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            save_new(
                &output.join("status.json"),
                &serde_json::json!({
                    "request_sha256":request_digest, "numerical_identity":null,
                    "execution_complete":false, "requested_coverage_complete":false,
                    "primary_recomputed":false, "error":format!("{error:#}")
                }),
            )?;
            return Err(error);
        }
    };
    save_new(&output.join("report.json"), &result.measurements)?;
    save_new(&output.join("operations.json"), &result.operations)?;
    save_new(&output.join("artifact.json"), &result.artifact)?;
    let complete =
        result.measurements.execution_complete && result.measurements.requested_coverage_complete;
    save_new(
        &output.join("status.json"),
        &serde_json::json!({
            "request_sha256":request_digest, "numerical_identity":result.measurements.numerical_identity,
            "execution_complete":result.measurements.execution_complete,
            "requested_coverage_complete":result.measurements.requested_coverage_complete,
            "primary_recomputed":false,"original_claim_verdicts_unchanged":true,
            "cache_status":result.cache_status,"publication_deferred":defer_publication
        }),
    )?;
    if complete && !defer_publication {
        match managed.finalize_publication_inventory() {
            Ok(inventory) => save_new(
                &output.join("publication-status.json"),
                &serde_json::json!({"finalization_completed":true,"inventory":inventory}),
            )?,
            Err(error) => {
                save_new(
                    &output.join("publication-status.json"),
                    &serde_json::json!({"finalization_completed":false,"error":format!("{error:#}")}),
                )?;
                return Err(error.into());
            }
        }
    }
    println!(
        "Optional cohort report: {}",
        output.join("report.json").display()
    );
    println!(
        "Cohort requested coverage: {}",
        if complete { "COMPLETE" } else { "INCOMPLETE" }
    );
    if !complete {
        bail!("optional cohort incomplete; inspect retained report and operations; original claims unchanged");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_input_and_new_outputs_preserve_prior_files() {
        let root = std::env::temp_dir().join(format!(
            "paper1-cohort-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let p = root.join("record.json");
        save_new(&p, &serde_json::json!({"original":true})).unwrap();
        let before = fs::read(&p).unwrap();
        assert!(save_new(&p, &serde_json::json!({"original":false})).is_err());
        assert_eq!(before, fs::read(&p).unwrap());
        assert!(read_bounded(&p, 1).is_err());
        assert!(freeze(&[], &[1], false, &root.join("plan.json")).is_err());
        fs::remove_file(&p).unwrap();
        fs::remove_dir(&root).unwrap();
    }
}
