use anyhow::Result;
use xc_spectral::ccm::cutoff_free::{assemble, CutoffFreeConfig};
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    anyhow::ensure!(
        args.len() == 4,
        "Usage: paper1-interval-export N BITS OUTPUT.json"
    );
    let modes: usize = args[1].parse()?;
    let bits: u32 = args[2].parse()?;
    anyhow::ensure!(
        matches!(modes, 10 | 120),
        "This control supports N=10 or N=120 at C=13"
    );
    let start = std::time::Instant::now();
    let matrix = assemble(&CutoffFreeConfig::new(13, modes, bits))?;
    let entries: Vec<_> = matrix
        .tau
        .iter()
        .map(|x| {
            serde_json::json!({
                "lower":x.lower().to_string(), "upper":x.upper().to_string()
            })
        })
        .collect();
    let result = serde_json::json!({"C":13,"N":modes,"precision_bits":bits,
        "geometric_terms":matrix.config.geometric_terms,"scalar_backend":matrix.scalar_backend,
        "component_evidence_digest":matrix.component_evidence_digest()?.to_string(),
        "tau":entries,"seconds":start.elapsed().as_secs_f64(),
        "toolkit_revision":"2bea90ec7cb23d4d615448c293c8af0f94e14119"});
    std::fs::write(&args[3], serde_json::to_vec(&result)?)?;
    println!(
        "interval Tau exported: N={}, bits={}, seconds={:.3}",
        modes,
        bits,
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
