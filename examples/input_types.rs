//! Example demonstrating different types of inputs in TCM replays.

use std::fs::File;
use std::io;
use tcm::{
    input::{
        BugpointInput, Input, InputCommand, PlayerButton, RestartInput, RestartType, TpsInput,
        VanillaInput,
    },
    meta::MetaV2,
    replay::{Replay, ReplaySerializer},
};

fn main() -> io::Result<()> {
    println!("TCM Input Types Example");

    let meta = MetaV2::new(240.0, 0, Some(98765));

    let inputs = vec![
        // Jump
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
        
        // Bugpoint input
        InputCommand::new(250, Input::Bugpoint(BugpointInput)),

        // TPS change
        InputCommand::new(300, Input::Tps(TpsInput { tps: 120.0 })),

        // Restart with new seed
        InputCommand::new(450, Input::Restart(RestartInput {
            restart_type: RestartType::Restart,
            new_seed: Some(54321),
        })),
    ];

    let replay = Replay::new(meta, inputs);

    println!("Created replay with {} inputs", replay.inputs.len());

    // Write to temp file
    let temp_path = std::env::temp_dir().join("tcm_input_types_example.tcm");
    let mut file = File::create(&temp_path)?;
    replay.serialize(&mut file)?;

    println!("Wrote to: {}", temp_path.display());
    
    // Clean up
    std::fs::remove_file(&temp_path)?;

    Ok(())
}
