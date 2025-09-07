//! Example showing how to convert between TCM format versions.

use std::fs::File;
use std::io;
use tcm::{
    input::{Input, InputCommand},
    meta::{Meta, MetaV1, MetaV2},
    replay::{Replay, ReplayDeserializer, ReplaySerializer},
};

fn main() -> io::Result<()> {
    println!("TCM Format Conversion Example");

    // Try to read a v1 format file
    if let Ok(mut file) = File::open("examples/data/restartv1.tcm") {
        println!("Reading v1 format file...");

        let v1_replay = Replay::<MetaV1>::deserialize(&mut file)?;

        println!("V1 TPS: {}", v1_replay.meta.tps());
        println!("V1 Input count: {}", v1_replay.inputs.len());

        // Convert to v2 format
        println!("Converting to v2 format...");

        let compatible_inputs: Vec<InputCommand> = v1_replay
            .inputs
            .into_iter()
            .filter(|cmd| matches!(cmd.input, Input::Vanilla(_) | Input::Restart(_)))
            .collect();

        let v2_meta = MetaV2::new(
            v1_replay.meta.tps(),
            v1_replay.meta.append_counter,
            Some(12345),
        );

        let v2_replay = Replay::new(v2_meta, compatible_inputs);

        println!("V2 TPS: {}", v2_replay.meta.tps());
        println!("V2 Input count: {}", v2_replay.inputs.len());

        // Write the converted v2 file to temp
        let temp_path = std::env::temp_dir().join("converted_v1_to_v2.tcm");
        let mut output_file = File::create(&temp_path)?;
        v2_replay.serialize(&mut output_file)?;

        println!("Wrote converted file to: {}", temp_path.display());

        // Clean up
        std::fs::remove_file(&temp_path)?;
    } else {
        println!("No v1 file found for conversion");
    }

    Ok(())
}
