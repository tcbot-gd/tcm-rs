use std::fs::File;
use std::io::BufReader;
use tcm::{meta::Meta, replay::ReplaySerializer, DynamicReplay};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TCM V2 to V1 Conversion Example");
    println!("Loading V2 file and converting to V1...");

    // Load a V2 TCM file
    let input_file = File::open("examples/data/restartv2.tcm")?;
    let mut reader = BufReader::new(input_file);
    let dynamic_replay = DynamicReplay::from_reader(&mut reader)?;

    println!("Loaded: TCM v{}", dynamic_replay.meta.version_instance());
    println!("Original TPS: {}", dynamic_replay.meta.tps());
    println!("Input count: {}", dynamic_replay.inputs.len());
    if let Some(seed) = dynamic_replay.meta.rng_seed() {
        println!(
            "Original RNG seed: {} (will be lost in V1 conversion)",
            seed
        );
    }

    // Try to convert to V1 format
    match dynamic_replay.to_v1() {
        Ok(v1_replay) => {
            println!("✓ Converted to V1");

            // Save as V1 format
            let output_path = std::env::temp_dir().join("converted_v2_to_v1.tcm");
            let mut output_file = File::create(&output_path)?;
            v1_replay.serialize(&mut output_file)?;

            println!("Saved to: {:?}", output_path);

            // Verify by loading the converted file
            let verify_file = File::open(&output_path)?;
            let mut verify_reader = BufReader::new(verify_file);
            let verified_replay = DynamicReplay::from_reader(&mut verify_reader)?;

            println!(
                "Verification: TCM v{}",
                verified_replay.meta.version_instance()
            );
            println!("Verified TPS: {}", verified_replay.meta.tps());
            println!("Verified input count: {}", verified_replay.inputs.len());

            println!("✓ Done");
        }
        Err(e) => {
            println!("✗ Cannot convert: {}", e);
        }
    }

    Ok(())
}
