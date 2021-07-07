use tokio::net::TcpStream;
use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
pub(crate) struct NetPacket {
    header: usize,
    body: Vec<u8>,
    current: String,
}

impl NetPacket {

    pub(crate) fn make(body: Vec<u8>, current: &str) -> Self {
        Self {
            header: 0,
            body: body,
            current: current.to_string()
        }
    }

    pub(crate) fn send(&self ,socket: &mut TcpStream) -> crate::Result<()> {
        
        println!("struct: {}", self.to_string());

        Ok(())
    }
}

impl std::string::ToString for NetPacket {

    fn to_string(&self) -> String {
        
        let mut text = String::new();

        text.push_str(format!("Body-Size: {}\r\n",self.header).as_str());
        text.push_str(format!("Current-Group: {}\r\n",self.current).as_str());
        text.push_str("\r\n");
        text.push_str(format!("{:?}", self.body).as_str());
        
        text
    }

}