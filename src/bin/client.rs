use dorea::client::{Client, ClientOption};
use dorea::server::DataValue;
use clap::clap_app;
use dorea::tools::parse_value_type;
use rustyline::Editor;
use rustyline::error::ReadlineError;

#[tokio::main]
pub async fn main() {

    let matches = clap_app!(dorea =>
        (version: "0.1.0")
        (author: "ZhuoEr Liu <mrxzx@qq.com>")
        (about: "Does awesome things")
        (@arg HOSTNAME: -h --hostname +takes_value "Set the server hostname")
        (@arg PORT: -p --port +takes_value "Set the server port")
        (@arg PASSWORD: -a --password +takes_value "Connect password")
    ).get_matches();

    let hostname = match matches.value_of("HOSTNAME") {
        None => "127.0.0.1",
        Some(v) => v
    };

    let port = match matches.value_of("PORT") {
        None => 3450,
        Some(v) => {
            match v.parse::<u16>() {
                Ok(n) => n,
                Err(_) => 3450
            }
        }
    };

    let password = match matches.value_of("PASSWORD") {
        None => "",
        Some(v) => v
    };

    let password = password.clone();

    let client = connect(hostname, port, &password);

    let mut client = match client {
        Ok(c) => c,
        Err(e) => {
            println!("[ERROR] {}", e);
            std::process::exit(1);
        }
    };

    let prompt = format!("{}:{} ~> ",hostname,port);
    let mut readline = Editor::<()>::new();
    loop {
        let read = readline.readline(&prompt);
        match read {
            Ok(line) => {
                let line = line.trim().to_string();
                if line == "exit" { break; }

                let res = execute(&mut client,line);
                if res != "" { println!("{}",res); }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            _ => {}
        }
    }
}

fn execute<'a>(client: &mut Client,message: String) -> String {

    let mut result = String::new();

    let segment: Vec<&str> = message.split(" ").collect();
    let length = segment.len();

    if length < 1 { return "Unknown input".to_string(); }

    let operator = segment.get(0).unwrap();
    let operator = operator.to_uppercase();
    let operator = operator.as_str();

    if operator == "GET" {

        if length != 2 { return "Incorrect number of parameters".to_string(); }
        let key = segment.get(1).unwrap();

        return match client.get(key) {
            None => format!("Undefined()"),
            Some(v) => format!("{:?}",v)
        }

    } else if operator == "SET" {
        if length < 3 { return "Missing parameters".to_string(); }

        let mut temp = segment.clone();

        let key = segment.get(1).unwrap();

        for _ in 0..2 { temp.remove(0); }
        temp.retain(|x| { x.trim() != "" });

        let value: String = temp.join(" ");
        let value: DataValue = match parse_value_type(value) {
            Ok(v) => v,
            Err(e) => { return e; }
        };

        if client.set(key, value.clone()) == true {
            result = String::from("OK");
        } else {
            return format!("Set error {} -> {:?}", key, value);
        }

    }else if operator == "SETEX"{

        if length < 4 { return "Missing parameters".to_string(); }

        let mut temp = segment.clone();

        let key = segment.get(1).unwrap();
        let expire = segment.get(segment.len() - 1).unwrap();

        let expire = match expire.parse::<u16>() {
            Ok(v) => v,
            Err(_) => 0
        };

        for _ in 0..2 { temp.remove(0); }
        temp.remove(temp.len() - 1);

        temp.retain(|x| { x.trim() != "" });

        let value: String = temp.join(" ");
        let value: DataValue = match parse_value_type(value) {
            Ok(v) => v,
            Err(e) => { return e; }
        };

        if client.setex(key, value.clone(),expire) == true {
            result = String::from("OK");
        } else {
            return format!("Set error {} -> {:?}", key, value);
        }

    } else if operator == "CLEAN" {

        if length != 1 { return "Incorrect number of parameters".to_string() }

        if client.clean() == true {
            result = String::from("OK");
        } else {
            return format!("Clean error {}", client.current_db);
        }

    } else if operator == "SELECT" {

        if length != 2 { return "Incorrect number of parameters".to_string(); }
        let key = segment.get(1).unwrap();

        if client.select(key) == true {
            result = String::from("OK");
        } else {
            return format!("Select error {}", client.current_db);
        }

    } else if operator == "REMOVE" {

        if length != 2 { return "Incorrect number of parameters".to_string(); }
        let key = segment.get(1).unwrap();

        if client.remove(key) == true {
            result = String::from("OK");
        } else {
            return format!("Remove error {}", key);
        }

    } else if operator == "DICT" {

        if length < 3 { return "Missing parameters".to_string(); }
        let key = segment.get(1).unwrap();

        let todo = segment.get(2).unwrap();
        let todo: &str = &todo.to_uppercase();

        let mut sub: String = String::new();

        if todo == "INSERT" {
            if length < 5 { return "Incorrect number of parameters".to_string(); }

            let temp = segment[4..].to_vec();

            let sub_key: &str = segment.get(3).unwrap().as_ref();
            let mut sub_value: String = temp.join(";@space;");

            if &sub_value[0..1] == "\"" && &sub_value[(sub_value.len() - 1)..] == "\"" {
                sub_value = sub_value[1..(sub_value.len() - 1)].parse().unwrap();
            }

            sub = format!("insert {} {}", sub_key, sub_value);

        } else if todo == "FIND" {

            if length < 4 { return "Incorrect number of parameters".to_string(); }

            let sub_key: &str = segment.get(3).unwrap().as_ref();

            sub = format!("find {}", sub_key);

        } else if todo == "REMOVE" {

            if length < 4 { return "Incorrect number of parameters".to_string(); }

            let sub_key: &str = segment.get(3).unwrap().as_ref();

            sub = format!("remove {}", sub_key);

        } else {
            return format!("Unknown subcommand: @{}", todo);
        }

        let exec = format!("dict {} {}",key, sub);
        match client.execute(&exec) {
            Ok(v) => { result = v; }
            Err(e) => { return e; }
        };
    }

    result
}

fn connect(hostname: &str,port: u16, password: &str) -> dorea::Result<Client> {
    Client::new(hostname, port, ClientOption { password })
}