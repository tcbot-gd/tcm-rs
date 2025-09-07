use crate::Frame;



#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerButton {
    Jump = 1,
    Left = 2,
    Right = 3,
}

impl TryFrom<u8> for PlayerButton {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(PlayerButton::Jump),
            2 => Ok(PlayerButton::Left),
            3 => Ok(PlayerButton::Right),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RestartType {
    Restart = 0,
    RestartFull = 1,
    Death = 2,
}

impl TryFrom<u8> for RestartType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RestartType::Restart),
            1 => Ok(RestartType::RestartFull),
            2 => Ok(RestartType::Death),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct RestartInput {
    pub restart_type: RestartType,
    pub new_seed: Option<u64>,
}

#[derive(Debug)]
pub struct TpsInput {
    pub tps: f32,
}

#[derive(Debug)]
pub struct VanillaInput {
    pub button: PlayerButton,
    pub push: bool,
    pub player2: bool,
}

#[derive(Debug)]
pub struct BugpointInput;

#[derive(Debug)]
pub enum Input {
    Vanilla(VanillaInput),
    Restart(RestartInput),
    Tps(TpsInput),
    Bugpoint(BugpointInput),
}

pub struct InputCommand {
    pub frame: Frame,
    pub input: Input,
}

impl InputCommand {
    pub fn new(frame: Frame, input: Input) -> Self {
        Self { frame, input }
    }

    pub fn is_vanilla(&self) -> bool {
        matches!(self.input, Input::Vanilla(_))
    }

    pub fn is_custom(&self) -> bool {
        !self.is_vanilla()
    }

    pub fn adjusted_frame(&self) -> Frame {
        match self.input {
            Input::Restart(_) => 0,
            _ => self.frame,
        }
    }
}
