use std::fs::File;
use std::io::BufReader;
use tcm::{DynamicReplay, meta::Meta, replay::ReplaySerializer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TCM Format Conversion Example");
    println!("Loading V1 file and converting to V2...");
    
    // Load any TCM file using auto-detection
    let input_file = File::open("examples/data/restartv1.tcm")?;
    let mut reader = BufReader::new(input_file);
    let dynamic_replay = DynamicReplay::from_reader(&mut reader)?;
    
    println!("Loaded: TCM v{}", dynamic_replay.meta.version_instance());
    println!("Original TPS: {}", dynamic_replay.meta.tps());
    println!("Input count: {}", dynamic_replay.inputs.len());
    
    // Convert to V2 format with optional RNG seed
    let v2_replay = dynamic_replay.to_v2(Some(12345)); // Add custom RNG seed
    
    // Save as V2 format
    let output_path = std::env::temp_dir().join("converted_v1_to_v2.tcm");
    let mut output_file = File::create(&output_path)?;
    v2_replay.serialize(&mut output_file)?;
    
    println!("Converted to V2 and saved to: {:?}", output_path);
    
    // Verify by loading the converted file
    let verify_file = File::open(&output_path)?;
    let mut verify_reader = BufReader::new(verify_file);
    let verified_replay = DynamicReplay::from_reader(&mut verify_reader)?;
    
    println!("Verification: TCM v{}", verified_replay.meta.version_instance());
    println!("Verified TPS: {}", verified_replay.meta.tps());
    println!("Verified input count: {}", verified_replay.inputs.len());
    if let Some(seed) = verified_replay.meta.rng_seed() {
        println!("RNG seed: {}", seed);
    }
    
    println!("✓ Conversion successful!");
    
    Ok(())
}
