use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BENCHMARK_SCHEMA_VERSION: u32 = 1;
const TOOLKIT_VERSION: &str = "0.13.5";
const TOOLKIT_REVISION: &str = "38447273088381611b82268053c3ebfe8e4c7838";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BenchmarkOptions {
    pub(crate) report_path: Option<PathBuf>,
    pub(crate) baseline_path: Option<PathBuf>,
    pub(crate) label: Option<String>,
    pub(crate) parallel_gl_roots: bool,
    pub(crate) comparison_mode: BenchmarkComparisonMode,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BenchmarkComparisonMode {
    /// Require the baseline and candidate to use the same runtime policy.
    #[default]
    SamePolicy,
    /// Compare serial and parallel GL-root scheduling while holding every other
    /// comparability field fixed.
    GlRootPolicyDelta,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum BenchmarkStatus {
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct BenchmarkTiming {
    duration_nanoseconds: u64,
    duration_seconds: f64,
    invocations: u64,
}

impl BenchmarkTiming {
    fn from_duration(duration: Duration) -> Self {
        Self {
            duration_nanoseconds: saturating_nanoseconds(duration),
            duration_seconds: duration.as_secs_f64(),
            invocations: 1,
        }
    }

    fn add_duration(&mut self, duration: Duration) {
        self.duration_nanoseconds = self
            .duration_nanoseconds
            .saturating_add(saturating_nanoseconds(duration));
        self.duration_seconds += duration.as_secs_f64();
        self.invocations = self.invocations.saturating_add(1);
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct BenchmarkDelta {
    before_nanoseconds: u64,
    after_nanoseconds: u64,
    delta_nanoseconds: i64,
    percent_change: Option<f64>,
    speedup: Option<f64>,
}

impl BenchmarkDelta {
    fn between(before: &BenchmarkTiming, after: &BenchmarkTiming) -> Self {
        let delta =
            i128::from(after.duration_nanoseconds) - i128::from(before.duration_nanoseconds);
        let delta_nanoseconds = i64::try_from(delta).unwrap_or_else(|_| {
            if delta.is_negative() {
                i64::MIN
            } else {
                i64::MAX
            }
        });
        let percent_change = (before.duration_nanoseconds != 0)
            .then(|| delta as f64 * 100.0 / before.duration_nanoseconds as f64);
        let speedup = (after.duration_nanoseconds != 0)
            .then(|| before.duration_nanoseconds as f64 / after.duration_nanoseconds as f64);
        Self {
            before_nanoseconds: before.duration_nanoseconds,
            after_nanoseconds: after.duration_nanoseconds,
            delta_nanoseconds,
            percent_change,
            speedup,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct BenchmarkComparison {
    baseline_report: PathBuf,
    baseline_label: Option<String>,
    #[serde(default)]
    comparison_mode: BenchmarkComparisonMode,
    #[serde(default)]
    baseline_parallel_gl_roots: bool,
    #[serde(default)]
    candidate_parallel_gl_roots: bool,
    interpretation: String,
    timings: BTreeMap<String, BenchmarkDelta>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct BenchmarkReport {
    schema_version: u32,
    label: Option<String>,
    status: BenchmarkStatus,
    started_unix_milliseconds: u64,
    completed_unix_milliseconds: u64,
    application_version: String,
    toolkit_version: String,
    #[serde(default)]
    toolkit_revision: String,
    optimized_build: bool,
    cache_mode: String,
    rayon_workers: usize,
    available_cpus: usize,
    rayon_num_threads_environment: Option<String>,
    #[serde(default)]
    parallel_gl_roots: bool,
    #[serde(default)]
    comparison_mode: BenchmarkComparisonMode,
    workload_signature: String,
    timings: BTreeMap<String, BenchmarkTiming>,
    comparison: Option<BenchmarkComparison>,
}

pub(crate) struct BenchmarkRecorder {
    options: BenchmarkOptions,
    started_unix_milliseconds: u64,
    workload_signature: String,
    timings: BTreeMap<String, BenchmarkTiming>,
}

impl BenchmarkRecorder {
    pub(crate) fn new(options: BenchmarkOptions, workload_signature: String) -> Result<Self> {
        if let Some(path) = &options.report_path {
            if path.exists() {
                bail!(
                    "benchmark report already exists; refusing to overwrite {}",
                    path.display()
                );
            }
        }
        Ok(Self {
            options,
            started_unix_milliseconds: unix_milliseconds(SystemTime::now()),
            workload_signature,
            timings: BTreeMap::new(),
        })
    }

    pub(crate) fn enabled(&self) -> bool {
        self.options.report_path.is_some()
    }

    pub(crate) fn record_duration(&mut self, name: &str, duration: Duration) {
        if !self.enabled() {
            return;
        }
        self.timings
            .entry(name.to_owned())
            .and_modify(|timing| timing.add_duration(duration))
            .or_insert_with(|| BenchmarkTiming::from_duration(duration));
    }

    pub(crate) fn record_seconds(&mut self, name: &str, seconds: f64) -> Result<()> {
        if !self.enabled() {
            return Ok(());
        }
        if !seconds.is_finite() || seconds < 0.0 {
            bail!("benchmark timing {name:?} must be finite and non-negative");
        }
        self.record_duration(name, Duration::from_secs_f64(seconds));
        Ok(())
    }

    pub(crate) fn finish(mut self, succeeded: bool) -> Result<Option<PathBuf>> {
        let Some(report_path) = self.options.report_path.clone() else {
            return Ok(None);
        };
        if self
            .options
            .baseline_path
            .as_ref()
            .is_some_and(|baseline| baseline == &report_path)
        {
            bail!("benchmark report and baseline paths must be different");
        }

        let mut report = BenchmarkReport {
            schema_version: BENCHMARK_SCHEMA_VERSION,
            label: self.options.label.clone(),
            status: if succeeded {
                BenchmarkStatus::Succeeded
            } else {
                BenchmarkStatus::Failed
            },
            started_unix_milliseconds: self.started_unix_milliseconds,
            completed_unix_milliseconds: unix_milliseconds(SystemTime::now()),
            application_version: env!("CARGO_PKG_VERSION").to_owned(),
            toolkit_version: TOOLKIT_VERSION.to_owned(),
            toolkit_revision: TOOLKIT_REVISION.to_owned(),
            optimized_build: !cfg!(debug_assertions),
            cache_mode: std::env::var("XC_CACHE_MODE").unwrap_or_else(|_| "reuse".to_owned()),
            rayon_workers: rayon::current_num_threads(),
            available_cpus: std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
            rayon_num_threads_environment: std::env::var("RAYON_NUM_THREADS").ok(),
            parallel_gl_roots: self.options.parallel_gl_roots,
            comparison_mode: self.options.comparison_mode,
            workload_signature: self.workload_signature,
            timings: std::mem::take(&mut self.timings),
            comparison: None,
        };

        if succeeded {
            if let Some(baseline_path) = &self.options.baseline_path {
                let baseline = read_report(baseline_path)?;
                validate_comparable(&baseline, &report, self.options.comparison_mode)?;
                let timings = report
                    .timings
                    .iter()
                    .filter_map(|(name, after)| {
                        baseline
                            .timings
                            .get(name)
                            .map(|before| (name.clone(), BenchmarkDelta::between(before, after)))
                    })
                    .collect();
                report.comparison = Some(BenchmarkComparison {
                    baseline_report: baseline_path.clone(),
                    baseline_label: baseline.label,
                    comparison_mode: self.options.comparison_mode,
                    baseline_parallel_gl_roots: baseline.parallel_gl_roots,
                    candidate_parallel_gl_roots: report.parallel_gl_roots,
                    interpretation:
                        "negative percent_change is faster; speedup greater than 1 is faster"
                            .to_owned(),
                    timings,
                });
            }
        }

        write_report(&report_path, &report)?;
        print_summary(&report_path, &report);
        Ok(Some(report_path))
    }
}

fn validate_comparable(
    baseline: &BenchmarkReport,
    candidate: &BenchmarkReport,
    comparison_mode: BenchmarkComparisonMode,
) -> Result<()> {
    if baseline.schema_version != BENCHMARK_SCHEMA_VERSION {
        bail!(
            "benchmark baseline uses schema {}, expected {}",
            baseline.schema_version,
            BENCHMARK_SCHEMA_VERSION
        );
    }
    if baseline.status != BenchmarkStatus::Succeeded {
        bail!("benchmark baseline must describe a successful run");
    }
    if baseline.workload_signature != candidate.workload_signature {
        bail!("benchmark baseline workload does not match the candidate workload");
    }
    if baseline.optimized_build != candidate.optimized_build {
        bail!("benchmark baseline and candidate must use the same build profile");
    }
    if baseline.cache_mode != candidate.cache_mode {
        bail!("benchmark baseline and candidate must use the same cache mode");
    }
    if baseline.rayon_workers != candidate.rayon_workers {
        bail!("benchmark baseline and candidate must use the same Rayon worker count");
    }
    if baseline.available_cpus != candidate.available_cpus {
        bail!("benchmark baseline and candidate must report the same available CPU count");
    }
    match comparison_mode {
        BenchmarkComparisonMode::SamePolicy
            if baseline.parallel_gl_roots != candidate.parallel_gl_roots =>
        {
            bail!("benchmark baseline and candidate must use the same GL root scheduling policy");
        }
        BenchmarkComparisonMode::GlRootPolicyDelta
            if baseline.parallel_gl_roots == candidate.parallel_gl_roots =>
        {
            bail!(
                "GL-root policy-delta comparison requires baseline and candidate to use different GL root scheduling policies"
            );
        }
        _ => {}
    }
    Ok(())
}

fn read_report(path: &Path) -> Result<BenchmarkReport> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read benchmark baseline {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse benchmark baseline {}", path.display()))
}

fn write_report(path: &Path, report: &BenchmarkReport) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create benchmark report directory {}",
                parent.display()
            )
        })?;
    }
    let mut bytes = serde_json::to_vec_pretty(report)?;
    bytes.push(b'\n');
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| {
            format!(
                "failed to create new benchmark report {}; existing reports are never overwritten",
                path.display()
            )
        })?;
    output
        .write_all(&bytes)
        .with_context(|| format!("failed to write benchmark report {}", path.display()))
}

fn print_summary(path: &Path, report: &BenchmarkReport) {
    eprintln!("benchmark report: {}", path.display());
    let Some(comparison) = &report.comparison else {
        return;
    };
    for name in [
        "toolkit_primary",
        "claim_and_research_capture",
        "process_total",
    ] {
        let Some(delta) = comparison.timings.get(name) else {
            continue;
        };
        let percent = delta
            .percent_change
            .map_or_else(|| "n/a".to_owned(), |value| format!("{value:+.2}%"));
        let speedup = delta
            .speedup
            .map_or_else(|| "n/a".to_owned(), |value| format!("{value:.3}x"));
        eprintln!(
            "  {name}: {:.3}s -> {:.3}s ({percent}, speedup {speedup})",
            delta.before_nanoseconds as f64 / 1_000_000_000.0,
            delta.after_nanoseconds as f64 / 1_000_000_000.0
        );
    }
}

fn saturating_nanoseconds(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn unix_milliseconds(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_report(workload: &str, duration: Duration) -> BenchmarkReport {
        BenchmarkReport {
            schema_version: BENCHMARK_SCHEMA_VERSION,
            label: Some("before".to_owned()),
            status: BenchmarkStatus::Succeeded,
            started_unix_milliseconds: 1,
            completed_unix_milliseconds: 2,
            application_version: env!("CARGO_PKG_VERSION").to_owned(),
            toolkit_version: TOOLKIT_VERSION.to_owned(),
            toolkit_revision: TOOLKIT_REVISION.to_owned(),
            optimized_build: !cfg!(debug_assertions),
            cache_mode: "reuse".to_owned(),
            rayon_workers: rayon::current_num_threads(),
            available_cpus: std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
            rayon_num_threads_environment: std::env::var("RAYON_NUM_THREADS").ok(),
            parallel_gl_roots: false,
            comparison_mode: BenchmarkComparisonMode::SamePolicy,
            workload_signature: workload.to_owned(),
            timings: BTreeMap::from([(
                "toolkit_primary".to_owned(),
                BenchmarkTiming::from_duration(duration),
            )]),
            comparison: None,
        }
    }

    #[test]
    fn delta_sign_and_speedup_identify_an_improvement() {
        let before = BenchmarkTiming::from_duration(Duration::from_secs(10));
        let after = BenchmarkTiming::from_duration(Duration::from_secs(8));
        let delta = BenchmarkDelta::between(&before, &after);
        assert_eq!(delta.delta_nanoseconds, -2_000_000_000);
        assert_eq!(delta.percent_change, Some(-20.0));
        assert_eq!(delta.speedup, Some(1.25));
    }

    #[test]
    fn comparison_rejects_different_workloads() {
        let baseline = fixture_report("run N=120", Duration::from_secs(10));
        let candidate = fixture_report("run N=121", Duration::from_secs(8));
        let error = validate_comparable(&baseline, &candidate, BenchmarkComparisonMode::SamePolicy)
            .unwrap_err();
        assert!(error.to_string().contains("workload"));
    }

    #[test]
    fn comparison_rejects_different_gl_root_scheduling_policies() {
        let baseline = fixture_report("run N=120", Duration::from_secs(10));
        let mut candidate = fixture_report("run N=120", Duration::from_secs(8));
        candidate.parallel_gl_roots = true;
        let error = validate_comparable(&baseline, &candidate, BenchmarkComparisonMode::SamePolicy)
            .unwrap_err();
        assert!(error.to_string().contains("GL root scheduling policy"));
    }

    #[test]
    fn policy_delta_comparison_requires_and_records_the_intended_difference() {
        let baseline = fixture_report("run N=120", Duration::from_secs(10));
        let mut candidate = fixture_report("run N=120", Duration::from_secs(8));
        let error = validate_comparable(
            &baseline,
            &candidate,
            BenchmarkComparisonMode::GlRootPolicyDelta,
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("requires baseline and candidate"));

        candidate.parallel_gl_roots = true;
        validate_comparable(
            &baseline,
            &candidate,
            BenchmarkComparisonMode::GlRootPolicyDelta,
        )
        .unwrap();
    }

    #[test]
    fn policy_delta_finish_stamps_the_comparison_contract_and_both_policies() {
        let directory = std::env::temp_dir().join(format!(
            "ccm-benchmark-policy-delta-{}-{}",
            std::process::id(),
            unix_milliseconds(SystemTime::now())
        ));
        fs::create_dir_all(&directory).unwrap();
        let baseline_path = directory.join("baseline.json");
        let candidate_path = directory.join("candidate.json");
        write_report(
            &baseline_path,
            &fixture_report("run N=120", Duration::from_secs(10)),
        )
        .unwrap();

        let mut recorder = BenchmarkRecorder::new(
            BenchmarkOptions {
                report_path: Some(candidate_path.clone()),
                baseline_path: Some(baseline_path.clone()),
                label: Some("parallel".to_owned()),
                parallel_gl_roots: true,
                comparison_mode: BenchmarkComparisonMode::GlRootPolicyDelta,
            },
            "run N=120".to_owned(),
        )
        .unwrap();
        recorder.record_duration("toolkit_primary", Duration::from_secs(8));
        recorder.finish(true).unwrap();

        let candidate = read_report(&candidate_path).unwrap();
        assert_eq!(
            candidate.comparison_mode,
            BenchmarkComparisonMode::GlRootPolicyDelta
        );
        let comparison = candidate.comparison.unwrap();
        assert_eq!(
            comparison.comparison_mode,
            BenchmarkComparisonMode::GlRootPolicyDelta
        );
        assert!(!comparison.baseline_parallel_gl_roots);
        assert!(comparison.candidate_parallel_gl_roots);

        fs::remove_file(candidate_path).unwrap();
        fs::remove_file(baseline_path).unwrap();
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn reports_round_trip_with_nanosecond_timings() {
        let report = fixture_report("run N=120", Duration::from_nanos(1_234_567_890));
        let bytes = serde_json::to_vec(&report).unwrap();
        let decoded: BenchmarkReport = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, report);
    }

    #[test]
    fn report_paths_are_never_overwritten() {
        let directory = std::env::temp_dir().join(format!(
            "ccm-benchmark-no-overwrite-{}-{}",
            std::process::id(),
            unix_milliseconds(SystemTime::now())
        ));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("report.json");
        write_report(&path, &fixture_report("run N=120", Duration::from_secs(1))).unwrap();

        let error = BenchmarkRecorder::new(
            BenchmarkOptions {
                report_path: Some(path.clone()),
                baseline_path: None,
                label: None,
                parallel_gl_roots: false,
                comparison_mode: BenchmarkComparisonMode::SamePolicy,
            },
            "run N=120".to_owned(),
        )
        .err()
        .expect("an existing report must be rejected");
        assert!(error.to_string().contains("refusing to overwrite"));

        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn older_benchmark_reports_default_to_serial_gl_roots() {
        let report = fixture_report("run N=120", Duration::from_secs(1));
        let mut value = serde_json::to_value(report).unwrap();
        value.as_object_mut().unwrap().remove("toolkit_revision");
        value.as_object_mut().unwrap().remove("parallel_gl_roots");
        value.as_object_mut().unwrap().remove("comparison_mode");
        let decoded: BenchmarkReport = serde_json::from_value(value).unwrap();
        assert!(decoded.toolkit_revision.is_empty());
        assert!(!decoded.parallel_gl_roots);
        assert_eq!(decoded.comparison_mode, BenchmarkComparisonMode::SamePolicy);
    }

    #[test]
    fn benchmark_toolkit_identity_matches_the_locked_dependency() {
        let lock = include_str!("../Cargo.lock");
        assert!(lock.contains(&format!("version = \"{TOOLKIT_VERSION}\"")));
        assert!(lock.contains(&format!("#{TOOLKIT_REVISION}")));
    }
}
