//! Replay serialization and deserialization.

use std::io::{Read, Seek, Write};

use crate::{
    input::{BugpointInput, Input, InputCommand, PlayerButton, RestartInput, VanillaInput},
    meta::{Meta, MetaV1, MetaV2},
    Frame,
};

pub trait ReplaySerializer<W: Write + Seek> {
    fn serialize(&self, writer: &mut W) -> std::io::Result<()>;
}

pub trait ReplayDeserializer<R: Read + Seek, M: Meta> {
    fn deserialize(reader: &mut R) -> std::io::Result<Replay<M>>;
}

#[derive(Debug, Clone)]
pub struct Replay<M: Meta> {
    pub meta: M,
    pub inputs: Vec<InputCommand>,
}

impl<M: Meta> Replay<M> {
    pub fn new_empty(tps: f32) -> Self {
        Self {
            meta: M::new_empty(tps),
            inputs: Vec::new(),
        }
    }

    pub fn new(meta: M, inputs: Vec<InputCommand>) -> Self {
        Self { meta, inputs }
    }
}

trait InternalSerializer<W: Write + Seek> {
    fn serialize_inputs_v1(&self, writer: &mut W) -> std::io::Result<()>;
    fn serialize_inputs_v2(&self, writer: &mut W) -> std::io::Result<()>;
}

trait InternalDeserializer<R: Read + Seek> {
    fn deserialize_inputs_v1(reader: &mut R) -> std::io::Result<Vec<InputCommand>>;
    fn deserialize_inputs_v2(reader: &mut R) -> std::io::Result<Vec<InputCommand>>;
}

mod v1 {
    use crate::input::{Input, RestartInput, RestartType, VanillaInput};

    pub const INPUT_MASK: u8 = 0b111;
    pub const PUSH_OFFSET: u8 = 7;
    pub const PUSH_MASK: u8 = 1 << PUSH_OFFSET;
    pub const PLAYER2_OFFSET: u8 = 6;
    pub const PLAYER2_MASK: u8 = 1 << PLAYER2_OFFSET;
    pub const EOM: u8 = 0xCC;

    pub fn serialize_input(input: &Input) -> Option<u8> {
        match input {
            Input::Vanilla(vanilla) => Some(
                ((vanilla.button as u8 - 1) & INPUT_MASK)
                    | ((vanilla.push as u8) << PUSH_OFFSET)
                    | ((vanilla.player2 as u8) << PLAYER2_OFFSET),
            ),
            Input::Restart(RestartInput { restart_type, .. }) => match restart_type {
                RestartType::Restart => Some(3),
                RestartType::RestartFull => Some(4),
                RestartType::Death => Some(5),
            },
            _ => None,
        }
    }

    pub fn deserialize_input(data: u8) -> Option<Input> {
        let input = data & INPUT_MASK;
        if input < 3 {
            return Some(Input::Vanilla(VanillaInput {
                button: (input + 1).try_into().ok()?,
                push: (data & PUSH_MASK) != 0,
                player2: (data & PLAYER2_MASK) != 0,
            }));
        }

        match input {
            3 => Some(Input::Restart(RestartInput {
                restart_type: RestartType::Restart,
                new_seed: None,
            })),
            4 => Some(Input::Restart(RestartInput {
                restart_type: RestartType::RestartFull,
                new_seed: None,
            })),
            5 => Some(Input::Restart(RestartInput {
                restart_type: RestartType::Death,
                new_seed: None,
            })),
            _ => None,
        }
    }
}

/// Reads a variable-length u32 from a reader using LEB128 encoding.
fn read_var_u32(reader: &mut (impl Read + Seek)) -> std::io::Result<u32> {
    let mut value = 0u32;
    let mut shift = 0usize;
    let mut buf = [0u8; 1];

    loop {
        reader.read_exact(&mut buf)?;
        let byte = buf[0];
        value |= ((byte & 0x7F) as u32) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
    }
}

/// Writes a variable-length u32 to a writer using LEB128 encoding.
fn write_var_u32(writer: &mut (impl Write + Seek), mut value: u32) -> std::io::Result<()> {
    let mut buf = [0u8; 1];

    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 {
            buf[0] = byte;
            writer.write_all(&buf)?;
            return Ok(());
        }
        buf[0] = byte | 0x80;
        writer.write_all(&buf)?;
    }
}

mod v2 {
    use std::io::{Read, Seek, Write};

    use crate::{
        input::{Input, RestartInput, VanillaInput},
        Frame,
    };

    pub const PUSH_OFFSET: u8 = 2;
    pub const PLAYER2_OFFSET: u8 = 3;
    pub const CUSTOM_OFFSET: u8 = 2;
    pub const EXTRA_OFFSET: u8 = 4;
    pub const DELTA_OFFSET: u8 = 5;

    pub const INPUT_MASK: u8 = 0b11;
    pub const PUSH_MASK: u8 = 1 << PUSH_OFFSET;
    pub const PLAYER2_MASK: u8 = 1 << PLAYER2_OFFSET;
    pub const CUSTOM_MASK: u8 = 0b11 << CUSTOM_OFFSET;
    pub const EXTRA_MASK: u8 = 1 << EXTRA_OFFSET;
    pub const DELTA_DATA_MASK: u8 = 0b111 << DELTA_OFFSET;

    pub fn craft_input(button: u8, push: bool, player2: bool, swift: bool) -> u8 {
        let mut byte = button & INPUT_MASK;
        byte |= (push as u8) << PUSH_OFFSET;
        byte |= (player2 as u8) << PLAYER2_OFFSET;
        byte |= (swift as u8) << EXTRA_OFFSET;
        byte
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ByteBlob {
        Zero = 0,
        One = 1,
        Two = 2,
        Four = 3,
    }

    impl TryFrom<u8> for ByteBlob {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(ByteBlob::Zero),
                1 => Ok(ByteBlob::One),
                2 => Ok(ByteBlob::Two),
                3 => Ok(ByteBlob::Four),
                _ => Err(()),
            }
        }
    }

    impl ByteBlob {
        pub const fn max(self) -> Frame {
            match self {
                ByteBlob::Zero => 0,
                ByteBlob::One => u8::MAX as Frame,
                ByteBlob::Two => u16::MAX as Frame,
                ByteBlob::Four => u32::MAX as Frame,
            }
        }

        pub fn serialize(self, writer: &mut impl Write, value: Frame) -> std::io::Result<()> {
            assert!(value <= self.max());
            match self {
                ByteBlob::Zero => Ok(()),
                ByteBlob::One => writer.write_all(&[value as u8]),
                ByteBlob::Two => writer.write_all(&(value as u16).to_le_bytes()),
                ByteBlob::Four => writer.write_all(&(value as u32).to_le_bytes()),
            }
        }

        pub fn iter() -> impl Iterator<Item = Self> {
            [Self::Zero, Self::One, Self::Two, Self::Four]
                .iter()
                .copied()
        }
    }

    pub struct SerializerDeltaInfo {
        pub blob: ByteBlob,
        magic: Option<Frame>,
        pub delta: Frame,
    }

    impl SerializerDeltaInfo {
        pub fn new(delta: Frame, last_delta: Frame) -> Self {
            for byte in ByteBlob::iter() {
                if delta <= byte.max() {
                    return Self {
                        blob: byte,
                        magic: None,
                        delta,
                    };
                }

                if last_delta == 0 || last_delta > delta {
                    continue;
                }

                let magic = delta - last_delta;
                if magic <= byte.max() {
                    return Self {
                        blob: byte,
                        magic: Some(last_delta),
                        delta,
                    };
                }
            }

            panic!("Delta too big: {} (last: {})", delta, last_delta);
        }

        pub fn empty(&self) -> bool {
            self.blob == ByteBlob::Zero
        }

        pub fn magic(&self) -> bool {
            self.magic.is_some()
        }

        pub fn craft(&self) -> u8 {
            (self.magic() as u8) | ((self.blob as u8) << 1)
        }

        pub fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
            if self.empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot serialize empty delta",
                ));
            }

            let value_to_serialize = if let Some(last_delta) = self.magic {
                self.delta - last_delta
            } else {
                self.delta
            };
            self.blob.serialize(writer, value_to_serialize)
        }
    }

    impl Default for SerializerDeltaInfo {
        fn default() -> Self {
            Self {
                blob: ByteBlob::Zero,
                magic: None,
                delta: 0,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum SerializerBlob {
        Action,
        FrameDelta,
        Tps(f32),
        Seed(u64),
    }

    pub fn serialize_input(input: &Input, swift: bool) -> u8 {
        if let Input::Vanilla(VanillaInput {
            button,
            push,
            player2,
        }) = input
        {
            return craft_input(*button as u8, *push, *player2, swift);
        }

        assert!(!swift, "Non-vanilla inputs cannot be swift");

        let t: u8;
        let mut extra = false;

        match input {
            Input::Restart(RestartInput {
                restart_type,
                new_seed,
            }) => {
                t = *restart_type as u8;
                extra = new_seed.is_some();
            }
            Input::Tps(_) => {
                t = 3;
            }
            Input::Bugpoint(_) => {
                t = 3;
                extra = true;
            }
            _ => unreachable!(),
        }

        (t | (extra as u8) << 2) << CUSTOM_OFFSET
    }

    pub struct DeserializerDeltaInfo {
        pub blob: ByteBlob,
        pub last_delta: Option<Frame>,
    }

    impl Default for DeserializerDeltaInfo {
        fn default() -> Self {
            Self {
                blob: ByteBlob::Zero,
                last_delta: None,
            }
        }
    }

    pub enum DeserializerBlob {
        Action,
        FrameDelta,
        Tps,
        Seed,
    }

    impl DeserializerDeltaInfo {
        pub fn new(data: u8, last_delta: Frame) -> Self {
            let bytes = (data >> 1) & 0b11;
            let magic = (data & 1) != 0;

            Self {
                blob: bytes.try_into().unwrap(),
                last_delta: if magic { Some(last_delta) } else { None },
            }
        }

        pub fn empty(&self) -> bool {
            self.blob == ByteBlob::Zero
        }

        pub fn read<R: Read + Seek>(
            &self,
            reader: &mut R,
            p_last_delta: &mut Frame,
        ) -> std::io::Result<Frame> {
            if self.empty() {
                let result = self.last_delta.unwrap_or(0); // Empty delta: magic returns last_delta, non-magic returns 0
                return Ok(result);
            }

            let value = match self.blob {
                ByteBlob::Zero => 0,
                ByteBlob::One => {
                    let mut buf = [0u8; 1];
                    reader.read_exact(&mut buf)?;
                    buf[0] as Frame
                }
                ByteBlob::Two => {
                    let mut buf = [0u8; 2];
                    reader.read_exact(&mut buf)?;
                    u16::from_le_bytes(buf) as Frame
                }
                ByteBlob::Four => {
                    let mut buf = [0u8; 4];
                    reader.read_exact(&mut buf)?;
                    u32::from_le_bytes(buf) as Frame
                }
            };

            let result = self.last_delta.unwrap_or(0) + value;

            if result != 0 {
                *p_last_delta = result;
            }

            Ok(result)
        }
    }
}

impl<W: Write + Seek, M: Meta> InternalSerializer<W> for Replay<M> {
    fn serialize_inputs_v1(&self, writer: &mut W) -> std::io::Result<()> {
        write_var_u32(writer, self.inputs.len() as u32)?;

        self.inputs
            .iter()
            .try_for_each(|input| -> std::io::Result<()> {
                let input_data = v1::serialize_input(&input.input).ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Unsupported input type in v1 replay",
                ))?;
                write_var_u32(writer, input.frame as u32)?;
                writer.write_all(&[input_data])?;
                Ok(())
            })?;

        writer.write_all(&[v1::EOM])?;
        Ok(())
    }

    fn serialize_inputs_v2(&self, writer: &mut W) -> std::io::Result<()> {
        if self.inputs.is_empty() {
            return Ok(());
        }

        use v2::{SerializerBlob, SerializerDeltaInfo};

        write_var_u32(writer, self.inputs[0].frame as u32)?;

        let mut next_blob = SerializerBlob::Action;
        let mut last_delta = 0u64;
        let mut next_delta = SerializerDeltaInfo::default();

        let mut iter = self.inputs.iter().peekable();

        while iter.peek().is_some() {
            match next_blob {
                SerializerBlob::Action => {
                    let input = iter.next().unwrap();
                    let mut next = iter.peek();
                    let swift = next.is_some_and(|n| {
                        if n.frame != input.frame {
                            return false;
                        }

                        match (&input.input, &n.input) {
                            (Input::Vanilla(a), Input::Vanilla(b)) => {
                                a.button == b.button && a.push != b.push && a.player2 == b.player2
                            }
                            _ => false,
                        }
                    });

                    if swift {
                        iter.next();
                        next = iter.peek();
                    }

                    if let Some(next1) = next {
                        let this_frame = input.adjusted_frame();
                        let next_frame = next1.frame;
                        let delta = next_frame.saturating_sub(this_frame);
                        next_delta = SerializerDeltaInfo::new(delta, last_delta);
                    } else {
                        next_delta = SerializerDeltaInfo::default();
                    }

                    if next_delta.delta != 0 {
                        last_delta = next_delta.delta;
                    }

                    let data = v2::serialize_input(&input.input, swift);
                    let delta_data = next_delta.craft();
                    let data = data | (delta_data << v2::DELTA_OFFSET);

                    writer.write_all(&[data])?;

                    match &input.input {
                        Input::Tps(tps) => {
                            next_blob = SerializerBlob::Tps(tps.tps);
                        }
                        Input::Restart(restart) => {
                            if let Some(seed) = restart.new_seed {
                                next_blob = SerializerBlob::Seed(seed);
                            } else if next_delta.empty() {
                                next_blob = SerializerBlob::Action;
                            } else {
                                next_blob = SerializerBlob::FrameDelta;
                            }
                        }
                        _ => {
                            if next_delta.empty() {
                                next_blob = SerializerBlob::Action;
                            } else {
                                next_blob = SerializerBlob::FrameDelta;
                            }
                        }
                    }
                }
                SerializerBlob::FrameDelta => {
                    next_delta.serialize(writer)?;
                    next_blob = SerializerBlob::Action;
                }
                SerializerBlob::Tps(tps) => {
                    let tps_bytes = tps.to_le_bytes();
                    writer.write_all(&tps_bytes)?;
                    if next_delta.empty() {
                        next_blob = SerializerBlob::Action;
                    } else {
                        next_blob = SerializerBlob::FrameDelta;
                    }
                }
                SerializerBlob::Seed(seed) => {
                    writer.write_all(&seed.to_le_bytes())?;
                    if next_delta.empty() {
                        next_blob = SerializerBlob::Action;
                    } else {
                        next_blob = SerializerBlob::FrameDelta;
                    }
                }
            }
        }

        Ok(())
    }
}

macro_rules! break_if_eof {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                return Err(e);
            }
        }
    };
}

impl<R: Read + Seek, M: Meta> InternalDeserializer<R> for Replay<M> {
    fn deserialize_inputs_v1(reader: &mut R) -> std::io::Result<Vec<InputCommand>> {
        let input_count = read_var_u32(reader)? as usize;
        let mut inputs = Vec::with_capacity(input_count);

        let mut buf = [0u8; 1];

        for _ in 0..input_count {
            let frame = read_var_u32(reader)? as Frame;
            reader.read_exact(&mut buf)?;
            let byte = buf[0];
            let input = v1::deserialize_input(byte).ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid input byte in v1 replay",
            ))?;
            inputs.push(InputCommand { frame, input });
        }

        Ok(inputs)
    }

    fn deserialize_inputs_v2(reader: &mut R) -> std::io::Result<Vec<InputCommand>> {
        use v2::{DeserializerBlob, DeserializerDeltaInfo};

        let mut inputs = Vec::new();

        let mut current_frame = read_var_u32(reader)? as Frame;
        let mut last_delta = 0u64;
        let mut next_blob = DeserializerBlob::Action;
        let mut next_delta = DeserializerDeltaInfo::default();
        let mut awaiting_seed = None;

        loop {
            match next_blob {
                DeserializerBlob::Action => {
                    use v2::{
                        CUSTOM_MASK, CUSTOM_OFFSET, DELTA_DATA_MASK, EXTRA_MASK, INPUT_MASK,
                        PLAYER2_MASK, PUSH_MASK,
                    };

                    let mut buf = [0u8; 1];
                    let read = reader.read_exact(&mut buf);
                    break_if_eof!(read);
                    let byte = buf[0];

                    let delta_data = (byte & DELTA_DATA_MASK) >> v2::DELTA_OFFSET;
                    next_delta = DeserializerDeltaInfo::new(delta_data, last_delta);

                    let input_data = byte & INPUT_MASK;

                    if input_data > 0 {
                        let button: PlayerButton = input_data.try_into().unwrap();
                        let push = (byte & PUSH_MASK) != 0;
                        let player2 = (byte & PLAYER2_MASK) != 0;
                        let swift = (byte & EXTRA_MASK) != 0;

                        inputs.push(InputCommand {
                            frame: current_frame,
                            input: Input::Vanilla(VanillaInput {
                                button,
                                push,
                                player2,
                            }),
                        });

                        if swift {
                            inputs.push(InputCommand {
                                frame: current_frame,
                                input: Input::Vanilla(VanillaInput {
                                    button,
                                    push: !push,
                                    player2,
                                }),
                            });
                        }

                        next_blob = DeserializerBlob::FrameDelta;
                    } else {
                        let custom_type = (byte & CUSTOM_MASK) >> CUSTOM_OFFSET;
                        let extra = (byte & EXTRA_MASK) != 0;

                        if custom_type == 3 {
                            if extra {
                                inputs.push(InputCommand {
                                    frame: current_frame,
                                    input: Input::Bugpoint(BugpointInput),
                                });
                                next_blob = DeserializerBlob::FrameDelta;
                            } else {
                                next_blob = DeserializerBlob::Tps;
                            }
                        } else {
                            let restart_type = custom_type.try_into().unwrap();
                            if extra {
                                awaiting_seed = Some((restart_type, current_frame));
                                next_blob = DeserializerBlob::Seed;
                            } else {
                                inputs.push(InputCommand {
                                    frame: current_frame,
                                    input: Input::Restart(RestartInput {
                                        restart_type,
                                        new_seed: None,
                                    }),
                                });
                                next_blob = DeserializerBlob::FrameDelta;
                            }
                            current_frame = 0;
                        }
                    }
                }
                DeserializerBlob::FrameDelta => {
                    let dt = next_delta.read(reader, &mut last_delta);
                    let dt = break_if_eof!(dt);
                    current_frame += dt;
                    next_blob = DeserializerBlob::Action;
                }
                DeserializerBlob::Tps => {
                    let mut buf = [0u8; 4];
                    reader.read_exact(&mut buf)?;
                    let tps = f32::from_le_bytes(buf);
                    inputs.push(InputCommand {
                        frame: current_frame,
                        input: Input::Tps(crate::input::TpsInput { tps }),
                    });
                    next_blob = DeserializerBlob::FrameDelta;
                }
                DeserializerBlob::Seed => {
                    if awaiting_seed.is_none() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Unexpected seed input",
                        ));
                    }

                    let (t, frame) = awaiting_seed.take().unwrap();
                    let seed = {
                        let mut buf = [0u8; 8];
                        reader.read_exact(&mut buf)?;
                        u64::from_le_bytes(buf)
                    };
                    inputs.push(InputCommand {
                        frame,
                        input: Input::Restart(RestartInput {
                            restart_type: t,
                            new_seed: Some(seed),
                        }),
                    });
                }
            }
        }

        Ok(inputs)
    }
}

const HEADER_SIZE: usize = 0x10;
const TCBOT_HEADER: [u8; HEADER_SIZE] = [
    0x9f, 0x88, 0x89, 0x84, 0x9f, 0x3b, 0x1d, 0xd8, 0xcc, 0xa1, 0x86, 0x8a, 0x88, 0x99, 0x84, 0x00,
];

impl<W: Write + Seek, M: Meta> ReplaySerializer<W> for Replay<M> {
    fn serialize(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&TCBOT_HEADER)?;
        writer.write_all(&self.meta.to_bytes())?;
        if M::version() == 1 {
            self.serialize_inputs_v1(writer)?;
        } else if M::version() == 2 {
            self.serialize_inputs_v2(writer)?;
        } else {
            panic!("Unsupported meta version: {}", M::version());
        }
        Ok(())
    }
}

impl<R: Read + Seek, M: Meta> ReplayDeserializer<R, M> for Replay<M> {
    fn deserialize(reader: &mut R) -> std::io::Result<Replay<M>> {
        let mut header = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header)?;
        if header != TCBOT_HEADER {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid header",
            ));
        }

        let mut meta_bytes = vec![0u8; M::size()];
        reader.read_exact(&mut meta_bytes)?;
        let meta = M::from_bytes(&meta_bytes);

        let inputs = if M::version() == 1 {
            Self::deserialize_inputs_v1(reader)?
        } else if M::version() == 2 {
            Self::deserialize_inputs_v2(reader)?
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unsupported meta version",
            ));
        };

        Ok(Replay { meta, inputs })
    }
}

/// A TCM replay with dynamic metadata type that can be either V1 or V2.
/// This allows working with replays without knowing the specific version at compile time.
pub type DynamicReplay = Replay<Box<dyn Meta>>;

impl Meta for Box<dyn Meta> {
    fn size() -> usize {
        0x40 // Both MetaV1 and MetaV2 have the same size
    }

    fn tps(&self) -> f32 {
        self.as_ref().tps()
    }

    fn tps_dt(&self) -> f32 {
        self.as_ref().tps_dt()
    }

    fn uses_dt(&self) -> bool {
        self.as_ref().uses_dt()
    }

    fn version() -> u8 {
        0 // This will be overridden by version_instance
    }

    fn version_instance(&self) -> u8 {
        self.as_ref().version_instance()
    }

    fn rng_seed(&self) -> Option<u64> {
        self.as_ref().rng_seed()
    }

    fn is_rng_seed_set(&self) -> bool {
        self.as_ref().is_rng_seed_set()
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        panic!("from_bytes cannot be called on Box<dyn Meta>, use specific types")
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.as_ref().to_bytes()
    }

    fn new_empty(_tps: f32) -> Self {
        panic!("new_empty cannot be called on Box<dyn Meta>, use specific types")
    }
}

impl DynamicReplay {
    /// Parse a replay from a reader, automatically detecting the format version.
    ///
    /// This method reads the version byte from the metadata and creates the appropriate
    /// metadata type, returning a single replay instance that can be used with any
    /// Meta trait methods.
    ///
    /// # Example
    /// ```no_run
    /// use std::fs::File;
    /// use tcm::DynamicReplay;
    ///
    /// let mut file = File::open("replay.tcm").unwrap();
    /// let replay = DynamicReplay::from_reader(&mut file).unwrap();
    /// println!("TPS: {}", replay.meta.tps());
    /// ```
    pub fn from_reader<R: Read + Seek>(reader: &mut R) -> std::io::Result<Self> {
        // Read version byte from metadata start (after header)
        let current_pos = reader.stream_position()?;

        // Skip header to get to metadata
        reader.seek(std::io::SeekFrom::Start(current_pos + HEADER_SIZE as u64))?;

        let mut version_buf = [0u8; 1];
        reader.read_exact(&mut version_buf)?;
        let version = version_buf[0];

        // Reset to start for deserialize methods
        reader.seek(std::io::SeekFrom::Start(current_pos))?;

        match version {
            1 => {
                let concrete_replay = Replay::<MetaV1>::deserialize(reader)?;
                Ok(Replay {
                    meta: Box::new(concrete_replay.meta) as Box<dyn Meta>,
                    inputs: concrete_replay.inputs,
                })
            }
            2 => {
                let concrete_replay = Replay::<MetaV2>::deserialize(reader)?;
                Ok(Replay {
                    meta: Box::new(concrete_replay.meta) as Box<dyn Meta>,
                    inputs: concrete_replay.inputs,
                })
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unsupported version: {}", version),
            )),
        }
    }

    /// Convert to V1 format.
    pub fn to_v1(self) -> Result<Replay<MetaV1>, String> {
        // Check for inputs that actually can't be represented in V1
        for input_cmd in &self.inputs {
            match &input_cmd.input {
                Input::Tps(_) => return Err("TPS changes not supported in V1".to_string()),
                Input::Bugpoint(_) => return Err("Bugpoint inputs not supported in V1".to_string()),
                _ => {}
            }
        }

        let meta_v1 = MetaV1 {
            tps: self.meta.tps(),
            append_counter: 0,
        };

        Ok(Replay {
            meta: meta_v1,
            inputs: self.inputs,
        })
    }

    /// Convert to V2 format.
    pub fn to_v2(self, rng_seed: Option<u64>) -> Replay<MetaV2> {
        let final_seed = rng_seed.or_else(|| self.meta.rng_seed());

        let meta_v2 = MetaV2::new(self.meta.tps(), 0, final_seed);

        Replay {
            meta: meta_v2,
            inputs: self.inputs,
        }
    }

    /// Convert to V2 format, preserving any existing RNG seed.
    pub fn to_v2_preserve_seed(self) -> Replay<MetaV2> {
        self.to_v2(None)
    }
}
