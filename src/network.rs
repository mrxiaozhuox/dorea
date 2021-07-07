use tokio::{io::AsyncWriteExt, net::TcpStream};

pub(crate) struct NetPacket {
    body: Vec<u8>,
    state: NetPacketState 
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NetPacketState {
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

        text.push_str(format!("Header-State: {:?}\r\n",self.state).as_str());
        text.push_str(format!("Body-Size: {}\r\n",body.len()).as_str());
        text.push_str(format!("Body-Content: {}\r\n",body).as_str());

        text
    }

}