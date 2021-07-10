use std::io::{Error, ErrorKind};

use bytes::{BufMut, BytesMut};
use nom::bytes::complete::{is_a, tag, take_until, take_while1};
use nom::combinator::map;
use nom::sequence::delimited;
use nom::IResult;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct NetPacket {
    body: Vec<u8>,
    state: NetPacketState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NetPacketState {
    OK,
    ERR,
    EMPTY,
    NOAUTH,
}

impl NetPacket {
    pub(crate) fn make(body: Vec<u8>, state: NetPacketState) -> Self {
        Self {
            body: body,
            state: state,
        }
    }

    pub(crate) async fn send(&self, socket: &mut TcpStream) -> crate::Result<()> {
        socket.write_all(self.to_string().as_bytes()).await?;

        Ok(())
    }
}

impl std::string::ToString for NetPacket {
    fn to_string(&self) -> String {
        let body = format!("{:?}", self.body);

        let mut text = String::new();

        text.push_str(format!("$: {} | ", body.as_bytes().len() + 4).as_str());
        text.push_str(format!("%: {:?} | ", self.state).as_str());
        text.push_str(format!("#: Byte{};", body).as_str());

        text
    }
}

pub struct Frame {
    legacy_content: Vec<u8>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            legacy_content: Vec::new(),
        }
    }

    // read message from socket [std]
    pub async fn parse_frame(&mut self, socket: &mut TcpStream) -> crate::Result<Vec<u8>> {

        let (mut reader, _) = socket.split();

        let mut buf = [0_u8; 20];
        let mut response = BytesMut::with_capacity(50);

        let size = reader.read(&mut buf).await?;

        response.put(&buf[0..size]);

        let mut message = String::from_utf8(response.as_ref().to_vec()).unwrap();
        
        if message.trim().len() == 0 {
            return Ok(vec![]);
        }

        message = message.trim().to_string();

        let (remain, total_size) = match Frame::parse_size(
            &message
        ) {
            Ok((remain, size)) => (remain.to_string(), size),
            Err(e) => {
                return Err(Box::new(Error::new(ErrorKind::NotFound, "size parsing error")));
            }
        };

        let (remain, mut data) = match Frame::parse_content(&remain) {
            Ok((remain, data)) => (remain.to_string(), data.to_string()),
            Err(e) => {
                return Err(Box::new(Error::new(ErrorKind::NotFound, "content parsing error")));
            }
        };

        if remain.len() > 0 {
            self.legacy_content = remain[1..].as_bytes().to_vec();
        }

        while data.as_bytes().len() < total_size {
            let mut append_buf = [0_u8; 1024];

            let diff = total_size - data.as_bytes().len();

            let max_buf_read: usize;

            if diff > 1024 {
                max_buf_read = 1024;
            } else {
                // + 1 was ;
                max_buf_read = diff + 1;
            }

            let len = reader.read(&mut append_buf[..max_buf_read]).await?;

            data = data + &String::from_utf8_lossy(&append_buf[0..len]);
        }

        if &data[data.len() - 1..] == ";" {
            data = data[..data.len() - 1].to_string();
        }

        Ok(data.as_bytes().to_vec())
    }

    fn parse_size(message: &str) -> IResult<&str, usize> {

        let (mut remain, length): (&str, usize) = delimited(
            tag("$: "),
            map(take_while1(|c: char| c.is_digit(10)), |int: &str| {
                int.parse::<usize>().unwrap()
            }),
            take_until(" | "),
        )(message)?;

        remain = &remain[3..];

        Ok((remain, length))
    }

    fn parse_content(message: &str) -> IResult<&str, &str> {

        let state: IResult<&str, &str> =
            delimited(
            tag("#: "),
            take_while1(|c| c != ';'),
            take_until(";")
            )(message);

        let (mut remain, mut result): (&str, &str) = ("", "");
        if state.is_err() {

            let (info, _) = tag("#: ")(message)?;

            result = info;
        } else {
            let temp: (&str, &str) = state.unwrap();
            remain = temp.0;
            result = temp.1;
        }

        Ok((remain, result))
    }
}

#[test]
fn test() {}
