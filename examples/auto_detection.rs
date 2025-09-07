use std::fs::File;
use std::io::BufReader;
use tcm::{DynamicReplay, meta::Meta};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get filename from command line or use default
    let args: Vec<String> = std::env::args().collect();
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "examples/data/restartv1.tcm"
    };
    
    println!("Auto-detecting and parsing: {}", filename);
    
    // Auto-detect and parse any TCM file without specifying the version
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    
    let replay = DynamicReplay::from_reader(&mut reader)?;
    
    // Now you can use the replay directly without matching on enum variants!
    println!("✓ Detected TCM v{} file", replay.meta.version_instance());
    println!("  TPS: {}", replay.meta.tps());
    println!("  Input count: {}", replay.inputs.len());
    
    // V2-specific features are available through trait methods
    if let Some(seed) = replay.meta.rng_seed() {
        println!("  RNG seed: {}", seed);
    }
    
    Ok(())
}
