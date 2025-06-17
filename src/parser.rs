use std::vec::IntoIter;

#[derive(Debug)]
pub enum RESPtypes {
    BulkStrings(String),
    Arrays(Vec<RESPtypes>),
    SimpleString(String)
}

impl RESPtypes {
    pub fn from_string(input: &str) -> Self {
        RESPtypes::decode(&mut RESPtypes::tokenize(&input))
    }
    pub fn tokenize(input: &str) -> IntoIter<&str> {
        input.split("\r\n").collect::<Vec<&str>>().into_iter()
    }
    pub fn decode(tokens: &mut IntoIter<&str>) -> Self {
        if let Some(val) = tokens.next() {
            if val.chars().nth(0) == Option::from('*') {
                let num: i32 = *&val[1..].parse().unwrap();
                let mut vector = Vec::new();
                for i in 0..num {
                    vector.push(Self::decode(tokens));
                }
                return Self::Arrays(vector);
            } else if val.chars().nth(0) == Option::from('$') {
                return Self::BulkStrings(String::from(tokens.next().unwrap()));
            }
            else if val.chars().nth(0) == Option::from('+') {
                return Self::SimpleString(String::from(tokens.next().unwrap()));
            }
        }
        Self::BulkStrings(String::from("non"))
    }
    pub fn to_resp_string(&self) -> String {
        match &self {
            RESPtypes::BulkStrings(val) => format!("${}\r\n{}\r\n", val.len(), val),
            RESPtypes::Arrays(val) => {
                val.iter().fold(format!("*{}\r\n", val.len()), |initial, current| {
                    initial + &*current.to_resp_string()
                })
            }
            RESPtypes::SimpleString(val) => format!("+{}\r\n", val)
        }
    }

}