use std::fs::File;
use std::io::BufReader;
use tcm::{DynamicReplay, meta::Meta, replay::ReplaySerializer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Lossless Conversion");
    
    // First, convert V1 to V2 (always works)
    println!("\n1. Loading V1 file...");
    let input_file = File::open("examples/data/restartv1.tcm")?;
    let mut reader = BufReader::new(input_file);
    let v1_replay = DynamicReplay::from_reader(&mut reader)?;
    
    println!("   Original: TCM v{}, TPS: {}", v1_replay.meta.version_instance(), v1_replay.meta.tps());
    
    // Convert to V2 without any V2-specific features
    println!("\n2. Converting V1 → V2 (no RNG seed)...");
    let v2_replay = v1_replay.to_v2(None); // No RNG seed = should be convertible back
    
    // Save the V2 version
    let temp_v2_path = std::env::temp_dir().join("temp_v2_simple.tcm");
    let mut temp_file = File::create(&temp_v2_path)?;
    v2_replay.serialize(&mut temp_file)?;
    
    println!("   Saved simple V2 file");
    
    // Now load it back and try to convert to V1
    println!("\n3. Loading the simple V2 file...");
    let verify_file = File::open(&temp_v2_path)?;
    let mut verify_reader = BufReader::new(verify_file);
    let loaded_v2 = DynamicReplay::from_reader(&mut verify_reader)?;
    
    println!("   Loaded: TCM v{}, TPS: {}", loaded_v2.meta.version_instance(), loaded_v2.meta.tps());
    println!("   RNG seed: {:?}", loaded_v2.meta.rng_seed());
    println!("   Uses DT: {}", loaded_v2.meta.uses_dt());
    
    // Try to convert back to V1
    println!("\n4. Attempting V2 → V1 conversion...");
    match loaded_v2.to_v1() {
        Ok(converted_v1) => {
            println!("   ✓ Success!");
            println!("   Final: TPS: {}", converted_v1.meta.tps());
        }
        Err(e) => {
            println!("   ✗ Failed: {}", e);
        }
    }
    
    Ok(())
}
