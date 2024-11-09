mod argument;
mod resp;

pub use argument::SetArguments;
pub use resp::{AsyncSequenceReader, NextSequenceError, Resp};

use resp::{RespBuilder, RespError};

pub enum Command {
    Ping,
    Echo(String),
    Set(SetArguments),
    Get(String),
}

impl TryFrom<Resp> for Command {
    type Error = CommandParseError;

    fn try_from(value: Resp) -> Result<Self, Self::Error> {
        match value {
            Resp::Array(array) => {
                let mut resp_iter = array.into_iter();
                let command_name = resp_iter.next().ok_or(CommandParseError::EmptyRespArray)?;
                match command_name {
                    Resp::BulkString(bulk_string) => match bulk_string.to_lowercase().as_str() {
                        "ping" => Ok(Command::Ping),
                        "echo" => {
                            let echo =
                                resp_iter.next().ok_or(CommandParseError::MissingArgument)?;
                            let echo = extract_bulk_string(echo)?;

                            Ok(Command::Echo(echo))
                        }
                        "set" => {
                            let key = resp_iter.next().ok_or(CommandParseError::MissingArgument)?;
                            let key = extract_bulk_string(key)?;

                            let value =
                                resp_iter.next().ok_or(CommandParseError::MissingArgument)?;
                            let value = extract_bulk_string(value)?;

                            let mut set_arguments = SetArguments::new(key, value);

                            while let Some(arg) = resp_iter.next() {
                                let arg = extract_bulk_string(arg)?;

                                match arg.to_lowercase().as_str() {
                                    "px" => {
                                        let px = resp_iter
                                            .next()
                                            .ok_or(CommandParseError::MissingArgument)?;
                                        let px = extract_bulk_string(px)?;
                                        let px: u128 = px
                                            .parse()
                                            .map_err(|_| CommandParseError::ArgumentType)?;

                                        set_arguments.px = Some(px);
                                    }
                                    _ => {
                                        return Err(CommandParseError::UnknownArgument);
                                    }
                                }
                            }

                            Ok(Command::Set(set_arguments))
                        }
                        "get" => {
                            let key = resp_iter.next().ok_or(CommandParseError::MissingArgument)?;
                            let key = extract_bulk_string(key)?;

                            Ok(Command::Get(key))
                        }
                        _ => Err(CommandParseError::UnknownCommandName(bulk_string)),
                    },
                    _ => Err(CommandParseError::CommandName),
                }
            }
            _ => Err(CommandParseError::Unrecognised(value)),
        }
    }
}

pub struct AsyncCommandReader<'sr, S>
where
    S: AsyncSequenceReader,
{
    resp_builder: RespBuilder<'sr, S>,
}

impl<'sr, S> AsyncCommandReader<'sr, S>
where
    S: AsyncSequenceReader,
{
    pub fn new(sequence_reader: &'sr mut S) -> Self {
        let resp_builder = RespBuilder::new(sequence_reader);
        Self { resp_builder }
    }

    pub async fn next_command(&mut self) -> anyhow::Result<Option<Command>> {
        let resp = self.resp_builder.next_resp().await;
        match resp {
            Ok(resp) => {
                let command = Command::try_from(resp)?;

                Ok(Some(command))
            }
            Err(e) => match e {
                RespError::EmptySequence => Ok(None),
                _ => Err(e.into()),
            },
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CommandParseError {
    #[error("Unrecognised command resp: {0:#?}")]
    Unrecognised(Resp),
    #[error("The RESP array is empty")]
    EmptyRespArray,
    #[error("Command name should be a bulk string")]
    CommandName,
    #[error("Command {0} is not known")]
    UnknownCommandName(String),
    #[error("The argument type for the command is wrong")]
    ArgumentType,
    #[error("Command is missing required arguments")]
    MissingArgument,
    #[error("The argument passed to this command is not known")]
    UnknownArgument,
}

fn extract_bulk_string(resp: Resp) -> Result<String, CommandParseError> {
    match resp {
        Resp::BulkString(bulk_string) => Ok(bulk_string),
        _ => Err(CommandParseError::ArgumentType),
    }
}
