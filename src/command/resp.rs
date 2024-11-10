use std::num::ParseIntError;

use bytes::{BufMut, Bytes, BytesMut};

pub type RespResult = Result<Resp, RespError>;

pub trait AsyncSequenceReader {
    async fn next_sequence(&mut self) -> Result<Option<String>, NextSequenceError>;
}

#[derive(Debug)]
pub enum Resp {
    Array(Vec<Resp>),
    BulkString(String),
    SimpleString(String),
    NullBulkString,
}

impl From<Resp> for Bytes {
    fn from(value: Resp) -> Self {
        match value {
            Resp::BulkString(bulk_string) => {
                let length = bulk_string.len().to_string();
                let capacity = length.len() + bulk_string.len() + 5;

                let mut b = BytesMut::with_capacity(capacity);
                b.put_u8(b'$');
                b.put_slice(length.as_bytes());
                b.put_slice(b"\r\n");
                b.put_slice(bulk_string.as_bytes());
                b.put_slice(b"\r\n");

                b.into()
            }
            Resp::SimpleString(simple_string) => {
                let capacity = simple_string.len() + 3;

                let mut b = BytesMut::with_capacity(capacity);
                b.put_u8(b'+');
                b.put_slice(simple_string.as_bytes());
                b.put_slice(b"\r\n");

                b.into()
            }
            Resp::NullBulkString => Bytes::from("$-1\r\n"),
            Resp::Array(array) => {
                let mut b = BytesMut::new();
                b.put_u8(b'*');
                b.put_slice(array.len().to_string().as_bytes());
                b.put_slice(b"\r\n");
                for item in array {
                    let item_bytes: Bytes = item.into();
                    b.put_slice(&item_bytes);
                }

                b.into()
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to get next sequence")]
pub struct NextSequenceError;

#[derive(thiserror::Error, Debug)]
pub enum RespError {
    #[error(transparent)]
    NextSequence(#[from] NextSequenceError),
    #[error("Sequence is empty")]
    EmptySequence,
    #[error("Unrecognised RESP: {0}")]
    Unrecognised(char),
    #[error("The array RESP requires a length. Got: {0}")]
    MissingArrayLength(ParseIntError),
    #[error("The bulk string RESP requires a length. Got: {0}")]
    MissingBulkStringLength(ParseIntError),
    #[error("The bulk string length did not match payload. Expected length: {0}, received: {1}")]
    BulkStringLength(usize, usize),
}

pub struct RespBuilder<'asr, S>
where
    S: AsyncSequenceReader,
{
    sequence_reader: &'asr mut S,
}

impl<'asr, S> RespBuilder<'asr, S>
where
    S: AsyncSequenceReader,
{
    pub fn new(sequence_reader: &'asr mut S) -> Self {
        Self { sequence_reader }
    }

    pub async fn next_resp(&mut self) -> RespResult {
        let sequence = self.next_sequence().await?;

        self.parse_resp(&sequence).await
    }

    async fn parse_resp(&mut self, sequence: &str) -> RespResult {
        let mut chars = sequence.chars();
        let first_char = chars.next().ok_or(RespError::EmptySequence)?;

        match first_char {
            '*' => {
                let number_of_elements: String = chars.collect();
                let number_of_elements: usize = number_of_elements
                    .parse()
                    .map_err(RespError::MissingArrayLength)?;

                self.parse_array(number_of_elements).await
            }
            '$' => {
                let string_length: String = chars.collect();
                let string_length: usize = string_length
                    .parse()
                    .map_err(RespError::MissingBulkStringLength)?;

                self.parse_bulk_string(string_length).await
            }
            c => Err(RespError::Unrecognised(c)),
        }
    }

    async fn parse_array(&mut self, number_of_elements: usize) -> RespResult {
        let mut array = Vec::with_capacity(number_of_elements);
        for _ in 0..number_of_elements {
            let element = Box::pin(self.next_resp()).await?;
            array.push(element);
        }

        let resp = Resp::Array(array);

        Ok(resp)
    }

    async fn parse_bulk_string(&mut self, string_length: usize) -> RespResult {
        let bulk_string = self.next_sequence().await?;
        let bulk_string_length = bulk_string.len();

        if bulk_string_length != string_length {
            Err(RespError::BulkStringLength(
                string_length,
                bulk_string_length,
            ))
        } else {
            let resp = Resp::BulkString(bulk_string);

            Ok(resp)
        }
    }

    async fn next_sequence(&mut self) -> Result<String, RespError> {
        self.sequence_reader
            .next_sequence()
            .await?
            .ok_or(RespError::EmptySequence)
    }
}
