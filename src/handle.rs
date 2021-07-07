use std::fs;

use tokio::net::TcpStream;

use crate::{config::DoreaFileConfig};
use crate::network::NetPacket;
use crate::Result;

pub(crate) async fn process(
    socket: &mut TcpStream, 
    config: DoreaFileConfig,
    current: String,
) -> Result<()> {

    let current = current;
    let readonly: bool;

    // check readonly state
    if config.database.readonly_group.contains(&current) {
        readonly = true;
    } else {
        readonly = false;
    }

    if config.connection.connection_password != "" {

    }

    NetPacket::make(vec![1,2,3,4,5,6], &current).send(socket).unwrap();


    Ok(())
}