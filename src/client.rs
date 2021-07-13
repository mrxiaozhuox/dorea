use tokio::net::TcpStream;

use crate::{
    network::{self, Frame, NetPacket, NetPacketState},
    value::DataValue,
};

pub struct DoreaClient {
    connection: TcpStream,
}

impl DoreaClient {
    pub async fn connect(addr: (&'static str, u16)) -> crate::Result<Self> {
        let addr = format!("{}:{}", addr.0, addr.1);

        let mut conn = TcpStream::connect(addr).await?;

        network::NetPacket::make("ping".as_bytes().to_vec(), network::NetPacketState::IGNORE)
            .send(&mut conn)
            .await?;

        let mut frame = Frame::new();

        let v = frame.parse_frame(&mut conn).await?;

        if frame.latest_state != NetPacketState::OK {
            let err = String::from_utf8_lossy(&v[..]).to_string();
            return Err(anyhow::anyhow!(err));
        }

        let obj = Self { connection: conn };

        Ok(obj)
    }

    pub async fn setex(
        &mut self,
        key: &str,
        value: DataValue,
        expire: usize,
    ) -> crate::Result<()> {
        let command = format!("set {} {} {}", key, value.to_string(), expire);

        let v = self.execute(&command).await?;
        if v.0 == NetPacketState::OK {
            return Ok(());
        }

        let result = String::from_utf8_lossy(&v.1).to_string();

        Err(anyhow::anyhow!(result))
    }

    pub async fn execute(&mut self, command: &str) -> crate::Result<(NetPacketState, Vec<u8>)> {

        let command_byte = command.as_bytes().to_vec();

        // println!("{}",NetPacket::make(command_byte.clone(), NetPacketState::IGNORE).to_string());

        NetPacket::make(command_byte, NetPacketState::IGNORE)
            .send(&mut self.connection)
            .await?;

        let mut frame = Frame::new();

        let result = frame.parse_frame(&mut self.connection).await?;

        Ok((frame.latest_state, result))
    }
}

#[cfg(test)]
mod test {

    use crate::value::DataValue;

    use super::DoreaClient;

    #[tokio::test]
    async fn ping() {
        let mut dorea = DoreaClient::connect(("127.0.0.1", 3450)).await.unwrap();

        for i in 0..10000 {
            dorea.setex(&i.to_string(),DataValue::Number(i as f64),0).await.unwrap();
        }
    }
}