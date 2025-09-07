# TCM - TCBot Macro Format Library

[![Crates.io](https://img.shields.io/crates/v/tcm.svg)](https://crates.io/crates/tcm)
[![Documentation](https://docs.rs/tcm/badge.svg)](https://docs.rs/tcm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust library for parsing and serializing TCM (TCBot Macro) format files used by the [tcbot.pro](https://tcbot.pro) Geometry Dash bot.

## Features

- 🎮 **Full TCM Format Support** - Parse and serialize both v1 and v2 TCM formats
- 🚀 **High Performance** - Efficient binary format handling with minimal allocations
- 🧪 **Thoroughly Tested** - Extensive test suite ensuring reliability

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tcm = "0.1"
```

## Quick Start

### Auto-Detection (Recommended)

The easiest way to work with TCM files is to use auto-detection, which automatically determines the format version:

```rust
use tcm::{DynamicReplay, meta::Meta};
use std::fs::File;
use std::io::BufReader;

// Open and parse any TCM file automatically
let file = File::open("replay.tcm")?;
let mut reader = BufReader::new(file);
let replay = DynamicReplay::from_reader(&mut reader)?;

// Work with the replay directly without matching on enum variants!
println!("TCM v{} - TPS: {}", replay.meta.version_instance(), replay.meta.tps());
println!("Input count: {}", replay.inputs.len());

// V2-specific features are available through trait methods
if let Some(seed) = replay.meta.rng_seed() {
    println!("RNG seed: {}", seed);
}
```

### Type-Specific Reading

If you know the format version in advance, you can use the typed API:

```rust
use tcm::{meta::MetaV2, replay::{Replay, ReplayDeserializer}};
use std::fs::File;

// Open a TCM file
let mut file = File::open("replay.tcm")?;

// Create a replay template (the type determines the format version)
let replay = Replay::<MetaV2>::deserialize(&mut file)?;

println!("TPS: {}", replay.meta.tps());
println!("Input count: {}", replay.inputs.len());
```

### Creating a TCM file

```rust
use tcm::{
    input::{Input, InputCommand, VanillaInput, PlayerButton},
    meta::MetaV2,
    replay::{Replay, ReplaySerializer}
};
use std::fs::File;

// Create metadata
let meta = MetaV2::new(240.0, 0, None);

// Create some inputs
let inputs = vec![
    InputCommand::new(100, Input::Vanilla(VanillaInput {
        button: PlayerButton::Jump,
        push: true,
        player2: false,
    })),
    InputCommand::new(150, Input::Vanilla(VanillaInput {
        button: PlayerButton::Jump,
        push: false,
        player2: false,
    })),
];

// Create replay
let replay = Replay::new(meta, inputs);

// Serialize to file
let mut file = File::create("output.tcm")?;
replay.serialize(&mut file)?;
```

## TCM Format Versions

This library supports both TCM format versions:

### Version 1 (MetaV1)
- Basic input recording
- Fixed TPS (ticks per second)
- Restart inputs
- Simple binary format

### Version 2 (MetaV2)
- All v1 features
- Optional RNG seed override
- TPS changes during replay
- More efficient encoding
- Bugpoint markers

## Input Types

The library supports various input types:

- **Vanilla Inputs**: Standard Geometry Dash inputs (jump, left, right)
- **Restart Inputs**: Level restarts with optional seed override
- **TPS Inputs**: Change replay speed during playback
- **Bugpoint Inputs**: Mark specific points for debugging

## Documentation

For detailed API documentation, visit [docs.rs/tcm](https://docs.rs/tcm).

## Examples

Check out the `examples/` directory for more comprehensive usage examples:

- `auto_detection.rs` - Auto-detect format version and parse any TCM file
- `basic_usage.rs` - Basic reading and writing operations
- `input_types.rs` - Working with different input types
- `format_conversion.rs` - Converting between v1 and v2 formats

## Contributing

Contributions regarding this rust port are welcome. Format design choices are not a part of this repository.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Related Projects

- [tcbot.pro](https://tcbot.pro)

