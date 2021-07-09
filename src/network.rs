use tokio::{io::AsyncWriteExt, net::TcpStream};
use std::collections::HashMap;

use nom::bytes::complete;
use tokio::io::AsyncReadExt;

pub struct NetPacket {
    body: Vec<u8>,
    state: NetPacketState 
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

    pub(crate) async fn send(&self ,socket: &mut TcpStream) -> crate::Result<()> {
        
        socket.write_all(self.to_string().as_bytes()).await?;

        Ok(())
    }
}

impl std::string::ToString for NetPacket {

    fn to_string(&self) -> String {
        
        let body = format!("{:?}", self.body);

        let mut text = String::new();

        text.push_str(format!("$: {} | ",body.as_bytes().len() + 4).as_str());
        text.push_str(format!("%: {:?} | ",self.state).as_str());
        text.push_str(format!("#: Byte{};",body).as_str());

        text
    }

}

// read message from socket [std]
pub async fn parse_frame(socket: &mut TcpStream) -> Vec<u8> {

    let mut buffer = [0; 20];

    let mut item_tab: HashMap<&str, String> = HashMap::new();

    // try to read item data
    let size = match socket.read(&mut buffer).await {
        Ok(v) => v,
        Err(_) => {
            return vec![];
        }
    };

    if size == 0 {
        return vec![];
    };

    let buf_vec = buffer[0..size].to_vec();

    let str = String::from_utf8_lossy(&buf_vec).to_string();
    let str = str.trim().to_string();
    let slice: Vec<&str> = str.split(" | ").collect();

    if slice.len() > 1 {
        for item in slice {
            let nom: nom::IResult<&str, &str> = complete::take_till(|c| c == ':')(item);
            let mut nom = nom.unwrap();

            // not a new section.
            if nom.0 == "" {
                break;
            }

            nom.0 = &nom.0[2..];

            let mut content = nom.0.to_string();

            if nom.1 == "#" {
                let content_i = nom.0;
                if &content_i[0..5] == "Byte[" && &content_i[content_i.len() - 1..] == "]" {
                    // is a byte data.
                    let v = match serde_json::from_str::<Vec<u8>>(content_i) {
                        Ok(v) => v,
                        Err(_) => vec![],
                    };

                    content = String::from_utf8_lossy(&v[..]).to_string();
                }
            }

            item_tab.insert(nom.1, content);
        }
    }

    // check body size
    let body_size = match item_tab.get("$") {
        Some(v) => {
            match v.parse::<usize>() {
                Ok(v) => v,
                Err(_) => 0,
            }
        },
        None => 0,
    };

    let mut body_content = match item_tab.get("#") {
        Some(v) => v.clone(),
        None => { return vec![]; }
    };

    println!("SIZE: {}",body_size); 
    while body_content.as_bytes().len() < body_size {

        let diff = body_size - body_content.as_bytes().len();

        let max_buffer: usize;

        if diff >= 2048 {
            max_buffer = 2048;
        } else {
            max_buffer = diff;
        }

        let mut buffer = [0; 2048];

        let v = match socket.read_exact(&mut buffer[0..max_buffer]).await {
            Ok(v) => v,
            Err(_) => { break; }
        };

        body_content += String::from_utf8_lossy(&buffer[0..v]).to_string().as_str();

    }

    println!("{:?}",body_content);
    return body_content.as_bytes().to_vec();
}