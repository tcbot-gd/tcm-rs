use std::fs::File;
use std::path::Path;

use tcm::input::Input;
use tcm::meta::{Meta, MetaV1, MetaV2};
use tcm::replay::{Replay, ReplayDeserializer};

#[test]
fn test_v1_read_only() {
    let example_path = Path::new("examples/data/restartv1.tcm");
    assert!(
        example_path.exists(),
        "Example file restartv1.tcm not found"
    );

    let mut file = File::open(example_path).expect("Failed to open restartv1.tcm");
    let replay =
        Replay::<MetaV1>::deserialize(&mut file).expect("Failed to deserialize restartv1.tcm");

    assert_eq!(MetaV1::version(), 1);
    assert!(replay.meta.tps() > 0.0, "TPS should be positive");
    assert!(!replay.meta.uses_dt(), "V1 should not use delta time");
    assert!(
        replay.meta.rng_seed().is_none(),
        "V1 should not have RNG seed"
    );

    assert!(!replay.inputs.is_empty(), "File should contain inputs");

    println!("V1 file read successfully:");
    println!("  TPS: {}", replay.meta.tps());
    println!("  Input count: {}", replay.inputs.len());

    for (i, input_cmd) in replay.inputs.iter().enumerate() {
        println!(
            "  Input {}: frame={}, type={:?}",
            i, input_cmd.frame, input_cmd.input
        );
        assert!(
            input_cmd.frame < 10_000_000,
            "Frame number seems unreasonable"
        );

        match &input_cmd.input {
            Input::Vanilla(_) | Input::Restart(_) => {}
            _ => panic!("V1 should only contain Vanilla and Restart inputs"),
        }
    }
}

#[test]
fn test_v2_restart_read_only() {
    let example_path = Path::new("examples/data/restartv2.tcm");
    assert!(
        example_path.exists(),
        "Example file restartv2.tcm not found"
    );

    let mut file = File::open(example_path).expect("Failed to open restartv2.tcm");
    let replay =
        Replay::<MetaV2>::deserialize(&mut file).expect("Failed to deserialize restartv2.tcm");

    assert_eq!(MetaV2::version(), 2);
    assert!(replay.meta.tps() > 0.0, "TPS should be positive");

    assert!(!replay.inputs.is_empty(), "File should contain inputs");

    println!("V2 restart file read successfully:");
    println!("  TPS: {}", replay.meta.tps());
    println!("  Uses DT: {}", replay.meta.uses_dt());
    println!("  RNG Seed: {:?}", replay.meta.rng_seed());
    println!("  Input count: {}", replay.inputs.len());

    for (i, input_cmd) in replay.inputs.iter().enumerate() {
        println!(
            "  Input {}: frame={}, type={:?}",
            i, input_cmd.frame, input_cmd.input
        );
        assert!(
            input_cmd.frame < 10_000_000,
            "Frame number seems unreasonable"
        );
    }
}

#[test]
fn test_v2_long_read_only() {
    let example_path = Path::new("examples/data/longv2.tcm");
    assert!(example_path.exists(), "Example file longv2.tcm not found");

    let mut file = File::open(example_path).expect("Failed to open longv2.tcm");
    let replay =
        Replay::<MetaV2>::deserialize(&mut file).expect("Failed to deserialize longv2.tcm");

    assert_eq!(MetaV2::version(), 2);
    assert!(replay.meta.tps() > 0.0, "TPS should be positive");

    assert!(!replay.inputs.is_empty(), "File should contain inputs");

    println!("V2 long file read successfully:");
    println!("  TPS: {}", replay.meta.tps());
    println!("  Uses DT: {}", replay.meta.uses_dt());
    println!("  RNG Seed: {:?}", replay.meta.rng_seed());
    println!("  Input count: {}", replay.inputs.len());

    let mut vanilla_count = 0;
    let mut restart_count = 0;
    let mut tps_count = 0;
    let mut bugpoint_count = 0;

    for input_cmd in &replay.inputs {
        assert!(
            input_cmd.frame < 100_000_000,
            "Frame number seems unreasonable"
        );

        match &input_cmd.input {
            Input::Vanilla(_) => vanilla_count += 1,
            Input::Restart(_) => restart_count += 1,
            Input::Tps(_) => tps_count += 1,
            Input::Bugpoint(_) => bugpoint_count += 1,
        }
    }

    println!("  Vanilla inputs: {}", vanilla_count);
    println!("  Restart inputs: {}", restart_count);
    println!("  TPS inputs: {}", tps_count);
    println!("  Bugpoint inputs: {}", bugpoint_count);

    assert!(
        replay.inputs.len() > 10,
        "Long file should have many inputs"
    );
}

#[test]
fn test_file_headers() {
    const TCBOT_HEADER: [u8; 16] = [
        0x9f, 0x88, 0x89, 0x84, 0x9f, 0x3b, 0x1d, 0xd8, 0xcc, 0xa1, 0x86, 0x8a, 0x88, 0x99, 0x84,
        0x00,
    ];

    for file_name in [
        "examples/data/restartv1.tcm",
        "examples/data/restartv2.tcm",
        "examples/data/longv2.tcm",
    ] {
        let file_bytes = std::fs::read(file_name).expect("Failed to read file");
        assert!(file_bytes.len() > 16, "File {} is too short", file_name);

        let header = &file_bytes[0..16];
        assert_eq!(
            header, TCBOT_HEADER,
            "File {} has incorrect header",
            file_name
        );

        println!("File {} has correct TCM header", file_name);
    }
}

#[test]
fn test_metadata_parsing() {
    {
        let file_bytes =
            std::fs::read("examples/data/restartv1.tcm").expect("Failed to read V1 file");
        assert!(
            file_bytes.len() > 16 + 0x40,
            "V1 file too short for metadata"
        );

        let meta_bytes = &file_bytes[16..16 + 0x40];
        let meta = MetaV1::from_bytes(meta_bytes);

        println!(
            "V1 metadata: TPS={}, append_counter={}",
            meta.tps(),
            meta.append_counter
        );
        assert!(
            meta.tps() > 0.0 && meta.tps() < 10000.0,
            "V1 TPS seems unreasonable"
        );
    }

    for file_name in ["examples/data/restartv2.tcm", "examples/data/longv2.tcm"] {
        let file_bytes = std::fs::read(file_name).expect("Failed to read V2 file");
        assert!(
            file_bytes.len() > 16 + 0x40,
            "V2 file too short for metadata"
        );

        let meta_bytes = &file_bytes[16..16 + 0x40];
        let meta = MetaV2::from_bytes(meta_bytes);

        println!(
            "{} metadata: TPS={}, uses_dt={}, rng_seed={:?}, append_counter={}",
            file_name,
            meta.tps(),
            meta.uses_dt(),
            meta.rng_seed(),
            meta.append_counter
        );
        assert!(
            meta.tps() > 0.0 && meta.tps() < 10000.0,
            "V2 TPS seems unreasonable"
        );
    }
}
