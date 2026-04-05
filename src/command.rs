#[derive(Debug)]

pub enum Command{
    Set{key:String, value:String},
    SetEx {key: String, value: String, secs: u64},
    Get{key:String},
    Delete{key:String},
    Ttl{key: String},
    Keys,
    Save,
    Unknown(String)
}

impl Command {
    pub fn parse(input: &str) -> Command{

        let mut parts = input.trim().splitn(4, " ");
 
        match parts.next() {
            
            Some("SET") => {
                match(parts.next(), parts.next()){
                    (Some(k), Some(v))=> Command::Set {
                        key: k.to_string(),
                        value: v.to_string(),
                    },
                    _ => Command::Unknown(input.to_string()),
                }},
            Some("SETEX") => match(parts.next(), parts.next(), parts.next()) {
                (Some(k), Some(secs_str), Some(v)) => { 
                    match secs_str.parse::<u64>() {
                        Ok(secs) => Command::SetEx{
                            key: k.to_string(),
                            value: v.to_string(), 
                            secs,
                            },
                        Err(_) => Command::Unknown(input.to_string()),
                        }
                }
                _ => Command::Unknown(input.to_string()),
                    
                }
            Some("GET") => match parts.next() {
                Some(k) => Command::Get {key: k.to_string()},
                _ => Command::Unknown(input.to_string()),
                },
            Some("DELETE") => match parts.next() {
                Some(k) => Command::Delete {key: k.to_string()},
                _ => Command::Unknown(input.to_string()),
                },
            Some("TTL") => match parts.next() {
                Some(k) => Command::Ttl { key: k.to_string() },
                _ => Command::Unknown(input.to_string()),
            },
            Some("SAVE") => Command::Save,
            Some("KEYS") => Command::Keys,
            _ => Command::Unknown(input.to_string()),
        }
    }
}
