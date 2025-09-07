use std::fs::File;
use std::path::Path;

use tcm::input::{Input, PlayerButton, RestartType};
use tcm::meta::{Meta, MetaV1, MetaV2};
use tcm::replay::{Replay, ReplayDeserializer};

#[test]
fn test_v1_content_validation() {
    let example_path = Path::new("examples/data/restartv1.tcm");
    assert!(
        example_path.exists(),
        "Example file restartv1.tcm not found"
    );

    let mut file = File::open(example_path).expect("Failed to open restartv1.tcm");
    let replay = Replay::<MetaV1>::deserialize(&mut file)
        .expect("Failed to deserialize restartv1.tcm");

    assert_eq!(MetaV1::version(), 1);

    assert!(replay.meta.tps() > 0.0, "TPS should be positive");
    assert!(!replay.meta.uses_dt(), "V1 should not use delta time");
    assert!(
        replay.meta.rng_seed().is_none(),
        "V1 should not have RNG seed"
    );
    assert!(
        !replay.meta.is_rng_seed_set(),
        "V1 should not have RNG seed set"
    );

    assert!(!replay.inputs.is_empty(), "File should contain inputs");

    for input_cmd in &replay.inputs {
        match &input_cmd.input {
            Input::Vanilla(vanilla) => {
                assert!(matches!(
                    vanilla.button,
                    PlayerButton::Jump | PlayerButton::Left | PlayerButton::Right
                ));
            }
            Input::Restart(restart) => {
                assert!(matches!(
                    restart.restart_type,
                    RestartType::Restart | RestartType::RestartFull | RestartType::Death
                ));
                assert!(
                    restart.new_seed.is_none(),
                    "V1 restarts should not have new seeds"
                );
            }
            Input::Tps(_) | Input::Bugpoint(_) => {
                panic!("V1 format should not contain TPS or Bugpoint inputs");
            }
        }

        assert!(
            input_cmd.frame < 1_000_000,
            "Frame number seems unreasonably high"
        );
    }

    println!(
        "V1 file validation passed with {} inputs",
        replay.inputs.len()
    );
}

#[test]
fn test_v2_restart_content_validation() {
    let example_path = Path::new("examples/data/restartv2.tcm");
    assert!(
        example_path.exists(),
        "Example file restartv2.tcm not found"
    );

    let mut file = File::open(example_path).expect("Failed to open restartv2.tcm");
    let replay = Replay::<MetaV2>::deserialize(&mut file)
        .expect("Failed to deserialize restartv2.tcm");

    assert_eq!(MetaV2::version(), 2);

    assert!(replay.meta.tps() > 0.0, "TPS should be positive");
    assert!(
        replay.meta.tps_dt() > 0.0,
        "TPS delta time should be positive"
    );

    assert!(!replay.inputs.is_empty(), "File should contain inputs");

    for input_cmd in &replay.inputs {
        match &input_cmd.input {
            Input::Vanilla(vanilla) => {
                assert!(matches!(
                    vanilla.button,
                    PlayerButton::Jump | PlayerButton::Left | PlayerButton::Right
                ));
            }
            Input::Restart(restart) => {
                assert!(matches!(
                    restart.restart_type,
                    RestartType::Restart | RestartType::RestartFull | RestartType::Death
                ));
            }
            Input::Tps(tps) => {
                assert!(tps.tps > 0.0, "TPS input should have positive value");
            }
            Input::Bugpoint(_) => {}
        }

        assert!(
            input_cmd.frame < 10_000_000,
            "Frame number seems unreasonably high"
        );
    }

    println!(
        "V2 restart file validation passed with {} inputs",
        replay.inputs.len()
    );
}

#[test]
fn test_v2_long_content_validation() {
    let example_path = Path::new("examples/data/longv2.tcm");
    assert!(example_path.exists(), "Example file longv2.tcm not found");

    let mut file = File::open(example_path).expect("Failed to open longv2.tcm");
    let replay = Replay::<MetaV2>::deserialize(&mut file)
        .expect("Failed to deserialize longv2.tcm");

    assert_eq!(MetaV2::version(), 2);

    assert!(replay.meta.tps() > 0.0, "TPS should be positive");
    assert!(
        replay.meta.tps_dt() > 0.0,
        "TPS delta time should be positive"
    );

    assert!(!replay.inputs.is_empty(), "File should contain inputs");

    let mut vanilla_count = 0;
    let mut restart_count = 0;
    let mut tps_count = 0;
    let mut bugpoint_count = 0;

    for input_cmd in &replay.inputs {
        match &input_cmd.input {
            Input::Vanilla(vanilla) => {
                vanilla_count += 1;
                assert!(matches!(
                    vanilla.button,
                    PlayerButton::Jump | PlayerButton::Left | PlayerButton::Right
                ));
            }
            Input::Restart(restart) => {
                restart_count += 1;
                assert!(matches!(
                    restart.restart_type,
                    RestartType::Restart | RestartType::RestartFull | RestartType::Death
                ));
            }
            Input::Tps(tps) => {
                tps_count += 1;
                assert!(tps.tps > 0.0, "TPS input should have positive value");
            }
            Input::Bugpoint(_) => {
                bugpoint_count += 1;
            }
        }

        assert!(
            input_cmd.frame < 100_000_000,
            "Frame number seems unreasonably high"
        );
    }

    println!(
        "V2 long file validation passed with {} inputs ({} vanilla, {} restart, {} tps, {} bugpoint)",
        replay.inputs.len(),
        vanilla_count,
        restart_count,
        tps_count,
        bugpoint_count
    );

    assert!(
        replay.inputs.len() > 10,
        "Long file should have more than 10 inputs"
    );
}

#[test]
fn test_frame_ordering() {
    for (file_name, is_v1) in [
        ("examples/data/restartv1.tcm", true),
        ("examples/data/restartv2.tcm", false),
        ("examples/data/longv2.tcm", false),
    ] {
        let example_path = Path::new(file_name);
        assert!(
            example_path.exists(),
            "Example file {} not found",
            file_name
        );

        let replay = if is_v1 {
            let mut file = File::open(example_path).expect("Failed to open file");
            let replay = Replay::<MetaV1>::deserialize(&mut file)
                .expect("Failed to deserialize file");
            (replay.inputs, replay.meta.tps())
        } else {
            let mut file = File::open(example_path).expect("Failed to open file");
            let replay = Replay::<MetaV2>::deserialize(&mut file)
                .expect("Failed to deserialize file");
            (replay.inputs, replay.meta.tps())
        };

        let (inputs, tps) = replay;

        if inputs.is_empty() {
            continue;
        }

        let mut last_frame = 0;
        let mut restart_occurred = false;

        for input_cmd in &inputs {
            if let Input::Restart(_) = input_cmd.input {
                restart_occurred = true;
                last_frame = 0;
                continue;
            }

            if restart_occurred {
                last_frame = input_cmd.frame;
                restart_occurred = false;
            } else {
                assert!(
                    input_cmd.frame >= last_frame,
                    "Frame ordering violation in {}: frame {} follows frame {}",
                    file_name,
                    input_cmd.frame,
                    last_frame
                );
                last_frame = input_cmd.frame;
            }
        }

        println!("Frame ordering validated for {} (TPS: {})", file_name, tps);
    }
}
