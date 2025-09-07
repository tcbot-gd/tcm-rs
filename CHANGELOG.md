# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial public release
- Support for TCM format v1 and v2
- Comprehensive documentation and examples
- Full serialization and deserialization support

## [0.1.0] - 2025-09-07

### Added
- Core TCM format parsing and serialization
- Support for MetaV1 and MetaV2 metadata structures
- Input types: Vanilla, Restart, TPS, and Bugpoint
- Comprehensive test suite
- Examples demonstrating library usage
- MIT license

### Features
- **TCM v1 Format Support**
  - Basic input recording and playback
  - Fixed TPS (ticks per second)
  - Vanilla player inputs (jump, left, right)
  - Simple restart commands
  
- **TCM v2 Format Support**
  - All v1 features plus:
  - Optional RNG seed override
  - Dynamic TPS changes during replay
  - Bugpoint debugging markers
  - More efficient binary encoding
  - Advanced restart options with seed override

- **Type Safety**
  - Strongly typed APIs preventing common errors
  - Comprehensive error handling
  - Memory-safe Rust implementation

- **Performance**
  - Efficient binary format handling
  - Minimal memory allocations
  - Fast serialization/deserialization

### Documentation
- Complete rustdoc documentation for all public APIs
- Usage examples in the `examples/` directory
- Comprehensive README with quick start guide
- Format specification documentation

### Testing
- Integration tests with real TCM files
- Round-trip serialization testing
- Format validation tests
- Example file verification
