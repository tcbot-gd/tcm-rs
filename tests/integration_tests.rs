use std::fs::File;
use std::path::Path;
use tempfile::NamedTempFile;

use tcm::meta::{Meta, MetaV1, MetaV2};
use tcm::replay::{Replay, ReplayDeserializer, ReplaySerializer};
use tcm::input::Input;

#[test]
fn test_v1_round_trip() {
    let example_path = Path::new("examples/restartv1.tcm");
    assert!(example_path.exists(), "Example file restartv1.tcm not found");

        let mut file = File::open(example_path).expect("Failed to open restartv1.tcm");
    let replay_template = Replay::<MetaV1>::new_empty(240.0);
    let replay = replay_template
        .deserialize(&mut file)
        .expect("Failed to deserialize restartv1.tcm");

        assert_eq!(MetaV1::version(), 1);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let mut writer = File::create(temp_file.path()).expect("Failed to create writer");

        replay
        .serialize(&mut writer)
        .expect("Failed to serialize replay");

        let mut reader = File::open(temp_file.path()).expect("Failed to open written file");
    let replay_template2 = Replay::<MetaV1>::new_empty(240.0);
    let replay_reread = replay_template2
        .deserialize(&mut reader)
        .expect("Failed to deserialize written file");

        assert_eq!(replay.meta.tps(), replay_reread.meta.tps());
    assert_eq!(replay.inputs.len(), replay_reread.inputs.len());

        for (original, reread) in replay.inputs.iter().zip(replay_reread.inputs.iter()) {
        assert_eq!(original.frame, reread.frame);
        match (&original.input, &reread.input) {
            (Input::Vanilla(a), Input::Vanilla(b)) => {
                assert_eq!(a.button, b.button);
                assert_eq!(a.push, b.push);
                assert_eq!(a.player2, b.player2);
            }
            (Input::Restart(a), Input::Restart(b)) => {
                assert_eq!(a.restart_type, b.restart_type);
                assert_eq!(a.new_seed, b.new_seed);
            }
            (Input::Tps(a), Input::Tps(b)) => {
                assert_eq!(a.tps, b.tps);
            }
            (Input::Bugpoint(_), Input::Bugpoint(_)) => {
                            }
            _ => panic!("Input types don't match: {:?} vs {:?}", original.input, reread.input),
        }
    }

        let original_bytes = std::fs::read(example_path).expect("Failed to read original file");
    let written_bytes = std::fs::read(temp_file.path()).expect("Failed to read written file");
    assert_eq!(
        original_bytes, written_bytes,
        "Binary content differs between original and written file"
    );
}

#[test]
fn test_v2_restart_round_trip() {
    let example_path = Path::new("examples/restartv2.tcm");
    assert!(example_path.exists(), "Example file restartv2.tcm not found");

        let mut file = File::open(example_path).expect("Failed to open restartv2.tcm");
    let replay_template = Replay::<MetaV2>::new_empty(240.0);
    let replay = replay_template
        .deserialize(&mut file)
        .expect("Failed to deserialize restartv2.tcm");

        assert_eq!(MetaV2::version(), 2);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let mut writer = File::create(temp_file.path()).expect("Failed to create writer");

        replay
        .serialize(&mut writer)
        .expect("Failed to serialize replay");

        let mut reader = File::open(temp_file.path()).expect("Failed to open written file");
    let replay_template2 = Replay::<MetaV2>::new_empty(240.0);
    let replay_reread = replay_template2
        .deserialize(&mut reader)
        .expect("Failed to deserialize written file");

        assert_eq!(replay.meta.tps(), replay_reread.meta.tps());
    assert_eq!(replay.meta.rng_seed(), replay_reread.meta.rng_seed());
    assert_eq!(replay.inputs.len(), replay_reread.inputs.len());

        for (original, reread) in replay.inputs.iter().zip(replay_reread.inputs.iter()) {
        assert_eq!(original.frame, reread.frame);
        match (&original.input, &reread.input) {
            (Input::Vanilla(a), Input::Vanilla(b)) => {
                assert_eq!(a.button, b.button);
                assert_eq!(a.push, b.push);
                assert_eq!(a.player2, b.player2);
            }
            (Input::Restart(a), Input::Restart(b)) => {
                assert_eq!(a.restart_type, b.restart_type);
                assert_eq!(a.new_seed, b.new_seed);
            }
            (Input::Tps(a), Input::Tps(b)) => {
                assert_eq!(a.tps, b.tps);
            }
            (Input::Bugpoint(_), Input::Bugpoint(_)) => {
                            }
            _ => panic!("Input types don't match"),
        }
    }

        let original_bytes = std::fs::read(example_path).expect("Failed to read original file");
    let written_bytes = std::fs::read(temp_file.path()).expect("Failed to read written file");
    assert_eq!(
        original_bytes, written_bytes,
        "Binary content differs between original and written file"
    );
}

#[test]
fn test_v2_long_round_trip() {
    let example_path = Path::new("examples/longv2.tcm");
    assert!(example_path.exists(), "Example file longv2.tcm not found");

        let mut file = File::open(example_path).expect("Failed to open longv2.tcm");
    let replay_template = Replay::<MetaV2>::new_empty(240.0);
    let replay = replay_template
        .deserialize(&mut file)
        .expect("Failed to deserialize longv2.tcm");

        assert_eq!(MetaV2::version(), 2);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let mut writer = File::create(temp_file.path()).expect("Failed to create writer");

        replay
        .serialize(&mut writer)
        .expect("Failed to serialize replay");

        let mut reader = File::open(temp_file.path()).expect("Failed to open written file");
    let replay_template2 = Replay::<MetaV2>::new_empty(240.0);
    let replay_reread = replay_template2
        .deserialize(&mut reader)
        .expect("Failed to deserialize written file");

        assert_eq!(replay.meta.tps(), replay_reread.meta.tps());
    assert_eq!(replay.meta.rng_seed(), replay_reread.meta.rng_seed());
    assert_eq!(replay.inputs.len(), replay_reread.inputs.len());

        for (original, reread) in replay.inputs.iter().zip(replay_reread.inputs.iter()) {
        assert_eq!(original.frame, reread.frame);
        match (&original.input, &reread.input) {
            (Input::Vanilla(a), Input::Vanilla(b)) => {
                assert_eq!(a.button, b.button);
                assert_eq!(a.push, b.push);
                assert_eq!(a.player2, b.player2);
            }
            (Input::Restart(a), Input::Restart(b)) => {
                assert_eq!(a.restart_type, b.restart_type);
                assert_eq!(a.new_seed, b.new_seed);
            }
            (Input::Tps(a), Input::Tps(b)) => {
                assert_eq!(a.tps, b.tps);
            }
            (Input::Bugpoint(_), Input::Bugpoint(_)) => {
                            }
            _ => panic!("Input types don't match"),
        }
    }

        let original_bytes = std::fs::read(example_path).expect("Failed to read original file");
    let written_bytes = std::fs::read(temp_file.path()).expect("Failed to read written file");
    assert_eq!(
        original_bytes, written_bytes,
        "Binary content differs between original and written file"
    );
}
