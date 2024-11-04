mod resp;

pub use resp::{AsyncSequenceReader, NextSequenceError, Resp};

use resp::{RespBuilder, RespError};

pub enum Command {
    Ping,
    Echo(String),
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
                            let echo = resp_iter
                                .next()
                                .ok_or(CommandParseError::MissingEchoArgument)?;

                            match echo {
                                Resp::BulkString(bulk_string) => Ok(Command::Echo(bulk_string)),
                                _ => Err(CommandParseError::EchoArgument),
                            }
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
    #[error("Command ECHO requires an argument")]
    MissingEchoArgument,
    #[error("Argument for ECHO should be a bulk string")]
    EchoArgument,
}
