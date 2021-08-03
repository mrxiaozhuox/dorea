use tokio::net::TcpStream;

use crate::{
    network::{self, Frame, NetPacket, NetPacketState},
    value::DataValue,
};

pub struct DoreaClient {
    connection: TcpStream,
}

impl DoreaClient {

    /// connect dorea-server
    ///
    /// ```rust
    /// use dorea::client::DoreaClient;
    /// #[tokio::main]
    /// pub async fn main() {
    ///     match DoreaClient::connect(("127.0.0.1", 3450), "").await {
    ///         Ok(mut c) => {
    ///             let result = c.execute("PING").await.unwrap();
    ///             println!("{:?}", result);
    ///         }
    ///         Err(_) => { println!("Connection error"); }
    ///     };
    /// }
    /// ```
    pub async fn connect(addr: (&'static str, u16), password: &str) -> crate::Result<Self> {
        let addr = format!("{}:{}", addr.0, addr.1);

        let mut conn = TcpStream::connect(addr).await?;

        if password != "" {
            network::NetPacket::make(
                format!("auth {}", password).as_bytes().to_vec(),
                network::NetPacketState::IGNORE
            ).send(&mut conn).await?;
        } else {
            network::NetPacket::make(
                "ping".as_bytes().to_vec(),
                network::NetPacketState::IGNORE
            ).send(&mut conn).await?;
        }

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
        let command = format!(
            "set {} b:{}: {}",
            key,
            base64::encode(value.to_string()),
            expire
        );

        let v = self.execute(&command).await?;
        if v.0 == NetPacketState::OK {
            return Ok(());
        }

        let result = String::from_utf8_lossy(&v.1).to_string();

        Err(anyhow::anyhow!(result))
    }

    pub async fn delete(
        &mut self,
        key: &str,
    ) -> crate::Result<()> {
        let command = format!("delete {} ", key);

        let v = self.execute(&command).await?;
        if v.0 == NetPacketState::OK {
            return Ok(());
        }

        let result = String::from_utf8_lossy(&v.1).to_string();

        Err(anyhow::anyhow!(result))
    }

    pub async fn get(&mut self, key: &str) -> crate::Result<DataValue> {
        let command = format!("get {}", key);

        let v = self.execute(&command).await?;
        if v.0 == NetPacketState::OK {
            let info = String::from_utf8_lossy(&v.1).to_string();
            return Ok(DataValue::from(&info));
        }

        let result = String::from_utf8_lossy(&v.1).to_string();

        Err(anyhow::anyhow!(result))

    }

    pub async fn clean(&mut self) -> crate::Result<()> {
        let command = format!("clean");

        let v = self.execute(&command).await?;
        if v.0 == NetPacketState::OK {
            return Ok(());
        }

        let result = String::from_utf8_lossy(&v.1).to_string();

        Err(anyhow::anyhow!(result))
    }

    pub async fn select(&mut self, db_name: &str) -> crate::Result<()> {
        let command = format!("select {}", db_name);

        let v = self.execute(&command).await?;
        if v.0 == NetPacketState::OK {
            return Ok(());
        }

        let result = String::from_utf8_lossy(&v.1).to_string();

        Err(anyhow::anyhow!(result))
    }

    pub async fn info(&mut self, dtype: InfoType) -> crate::Result<String> {
        let command = format!("info {}", dtype.to_string());

        let v = self.execute(&command).await?;
        if v.0 == NetPacketState::OK {
            let info = String::from_utf8_lossy(&v.1).to_string();
            return Ok(info);
        }

        let result = String::from_utf8_lossy(&v.1).to_string();

        Err(anyhow::anyhow!(result))
    }

    pub async fn execute(&mut self, command: &str) -> crate::Result<(NetPacketState, Vec<u8>)> {

        let command_byte = command.as_bytes().to_vec();

        NetPacket::make(command_byte, NetPacketState::IGNORE)
            .send(&mut self.connection)
            .await?;

        let mut frame = Frame::new();

        let result = frame.parse_frame(&mut self.connection).await?;

        Ok((frame.latest_state, result))
    }

    // pub async fn list(&mut self, key:&str) -> Option<CompList> {
    //     let list = self.get(key).await;
    //     if let Err(_) = list { return None; }
    //
    // }
}


// pub struct CompList<'a> {
//     client: DoreaClient
// }
//
// impl CompList {
//     pub fn new(client: DoreaClient) -> CompList {
//
//     }
// }

#[derive(Debug)]
pub enum InfoType {
    CurrentDataBase,
    MaxConnectionNumber,
    CurrentConnectionNumber,
    PreloadDatabaseList,
    ServerStartupTime,
    ServerVersion,
    // DataType(String),
}

impl std::string::ToString for InfoType {
    fn to_string(&self) -> String {
        match self {
            InfoType::CurrentDataBase => "current",
            InfoType::MaxConnectionNumber => "max-connect-num",
            InfoType::CurrentConnectionNumber => "current-connect-num",
            InfoType::PreloadDatabaseList => "preload-db-list",
            InfoType::ServerStartupTime => "server-startup-time",
            InfoType::ServerVersion => "version",
            // InfoType::DataType(v) => {
            //     format!("${} type", v).as_str()
            // },
            // _ => "", /* 预留数据 */
        }.to_string()
    }
}

// #[cfg(test)]
// mod client_test {
//
//     use crate::value::DataValue;
//     use super::DoreaClient;
//
//
//     #[tokio::test]
//     async fn test() {
//         let mut c = DoreaClient::connect(
//             ("127.0.0.1", 3450),
//             ""
//         ).await.unwrap();
//
//         c.setex("hello", DataValue::List(vec![
//             DataValue::Number(1.0),
//             DataValue::Number(2.0),
//             DataValue::Number(3.0),
//             DataValue::Number(4.0),
//             DataValue::Number(5.0),
//             DataValue::Number(6.0),
//         ]), 0).await;
//
//         let v = c.execute("edit hello incr 2").await;
//
//         let v = c.get("hello").await;
//
//         println!("{:?}",v);
//
//     //
//     //     let list = ["sam", "kevin", "leo"];
//     //
//     //     for i in list {
//     //         c.select(i).await.unwrap();
//     //         c.setex("name",DataValue::String(i.to_string()), 0).await;
//     //     }
//     //
//     //     for i in list {
//     //         c.select(i).await.unwrap();
//     //         println!("{}: {:?}", i, c.get("name").await);
//     //     }
//     }
// }