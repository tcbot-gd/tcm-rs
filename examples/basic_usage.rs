//! Basic usage example showing how to read and write TCM files.

use std::fs::File;
use std::io;
use tcm::{
    input::{Input, InputCommand, PlayerButton, VanillaInput},
    meta::{Meta, MetaV2},
    replay::{Replay, ReplayDeserializer, ReplaySerializer},
};

fn main() -> io::Result<()> {
    println!("TCM Basic Usage Example");

    // Read an existing TCM file (if available)
    if let Ok(mut file) = File::open("examples/data/longv2.tcm") {
        println!("Reading existing TCM file...");
        
        let replay = Replay::<MetaV2>::deserialize(&mut file)?;
        
        println!("TPS: {}", replay.meta.tps());
        println!("Input count: {}", replay.inputs.len());
        
        if let Some(seed) = replay.meta.rng_seed() {
            println!("RNG Seed: {}", seed);
        }
    }

    // Create a new replay
    println!("Creating a new replay...");
    
    let meta = MetaV2::new(240.0, 0, Some(12345));
    
    let inputs = vec![
        InputCommand::new(60, Input::Vanilla(VanillaInput {
            button: PlayerButton::Jump,
            push: true,
            player2: false,
        })),
        InputCommand::new(65, Input::Vanilla(VanillaInput {
            button: PlayerButton::Jump,
            push: false,
            player2: false,
        })),
    ];
    
    let replay = Replay::new(meta, inputs);
    
    // Write to temp file
    let temp_path = std::env::temp_dir().join("tcm_example_output.tcm");
    let mut output_file = File::create(&temp_path)?;
    replay.serialize(&mut output_file)?;
    
    println!("Wrote replay to: {}", temp_path.display());
    
    // Verify by reading it back
    let mut verify_file = File::open(&temp_path)?;
    let verified_replay = Replay::<MetaV2>::deserialize(&mut verify_file)?;
    
    println!("Verified TPS: {}", verified_replay.meta.tps());
    println!("Verified input count: {}", verified_replay.inputs.len());
    
    // Clean up
    std::fs::remove_file(&temp_path)?;
    
    Ok(())
}
