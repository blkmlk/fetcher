use std::collections::HashMap;
use serde_json::Value;
use crate::config::config::Type::{Boolean, JSON, Number, String as TypeString};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("parse error")]
    ParseError(String),
    #[error("invalid format")]
    InvalidFormat(String),
    #[error("unknown connection")]
    UnknownConnection(String),
    #[error("unknown type")]
    UnknownType(String),
    #[error("unknown property")]
    UnknownProperty(String),
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Connection {
    PostgresSQL,
    MySQL,
    MongoDB,
}

#[derive(PartialEq, Debug)]
pub enum ExpectedRows {
    Single,
    Multiple
}

pub enum Type {
    String,
    Number,
    Boolean,
    JSON,
}

pub struct Properties {
    pub ptype: Type,
    pub convert_name: Option<String>,
    pub return_attribute: Option<String>
}

impl Properties {
    pub fn new() -> Self {
        Self { ptype: TypeString, convert_name: None, return_attribute: None }
    }
}

pub struct Config {
    pub attr_groups: Vec<(String,Vec<AttributeGroup>)>
}

impl Config {
    pub fn new(attr_groups: Vec<(String,Vec<AttributeGroup>)>) -> Self {
        Self { attr_groups }
    }
}

pub struct AttributeGroup {
    pub conn: Connection,
    pub query: String,
    pub exp_rows: ExpectedRows,
    pub select_attrs: Vec<(String, Properties)>
}

impl AttributeGroup {
    pub fn new() -> Self {
        Self {
            conn: Connection::PostgresSQL,
            query: String::default(),
            exp_rows: ExpectedRows::Single,
            select_attrs: vec![],
        }
    }
}

pub fn parse(data: &[u8]) -> Result<Config, Error> {
    let raw: HashMap<String, Value> = serde_json::from_slice(data).
        map_err(|e| Error::ParseError(e.to_string()))?;

    let mut groups = vec![];

    for (k, v) in raw.iter() {
        let group_name = k;
        let mut props = vec![];
        match v {
            Value::Array(group) => {
                for v in group {
                    match v {
                        Value::Object(group_obj) => {
                            let mut attr_group = AttributeGroup::new();
                            for (k, v) in group_obj {
                                match k.to_string().as_str() {
                                    "connection"=> {
                                        attr_group.conn = parse_connection(v)?
                                    }
                                    "query" => {
                                        attr_group.query = parse_query(v)?
                                    }
                                    "expected_rows" => {
                                        attr_group.exp_rows = parse_exp_rows(v)?
                                    }
                                    "select_attributes" => {
                                        attr_group.select_attrs = parse_select_attributes(v)?
                                    }
                                    _ => {}
                                }
                            }

                            props.push(attr_group);
                        }
                        _=> return Err(Error::InvalidFormat(String::from("group is not an object")))
                    }
                }
            }
            _ => return Err(Error::InvalidFormat(String::from("not an array")))
        }
        groups.push((group_name.to_owned(), props))
    }

    Ok(Config::new(groups))
}

fn parse_connection(value: &Value) -> Result<Connection, Error> {
    let r = match value {
        Value::String(conn) => {
            match conn.as_str() {
                "postgresql" | "postgres" => Connection::PostgresSQL,
                "mysql" => Connection::MySQL,
                "mongodb" => Connection::MongoDB,
                _ => return Err(Error::UnknownConnection(conn.to_owned()))
            }
        }
        _ => return Err(Error::InvalidFormat(String::from("connection is not a string")))
    };

    Ok(r)
}

fn parse_query(value: &Value) -> Result<String, Error> {
    let r = match value {
        Value::String(query) => query.to_string(),
        _ => return Err(Error::InvalidFormat(String::from("invalid query")))
    };

    Ok(r)
}

fn parse_exp_rows(value: &Value) -> Result<ExpectedRows, Error> {
    let r = match value {
        Value::String(exp) => match exp.as_str() {
            "single" => ExpectedRows::Single,
            "multiple" => ExpectedRows::Multiple,
            _ => return Err(Error::InvalidFormat(String::from("invalid exp rows value")))
        }
        _ => return Err(Error::InvalidFormat(String::from("invalid exp rows")))
    };

    Ok(r)
}

fn parse_select_attributes(value: &Value) -> Result<Vec<(String, Properties)>, Error> {
    let r = match value {
        Value::Object(obj) => {
            let mut attrs = vec![];
            for (k, v) in obj {
                let name = k;
                match v {
                    Value::Array(values) => {
                        let mut props = Properties::new();
                        for v in values {
                            match v {
                                Value::String(prop) => {
                                    let (attr_type, attr_value) = prop.split_once("::").unwrap_or((prop, ""));
                                    match attr_type {
                                        "Type" => {
                                            let t = match attr_value {
                                                "String" => TypeString,
                                                "Number" => Number,
                                                "JSON" => JSON,
                                                "Boolean" => Boolean,
                                                _=> return Err(Error::UnknownType(String::from(attr_value)))
                                            };

                                            props.ptype = t;
                                        },
                                        "!ConvertName" => props.convert_name = Some(attr_value.to_string()),
                                        "ReturnAttribute" => props.return_attribute = Some(attr_value.to_string()),
                                        _=> return Err(Error::UnknownProperty(String::from(attr_type)))
                                    }
                                }
                                _ => return Err(Error::InvalidFormat(String::from("select_attributes")))
                            }
                        }
                        attrs.push((name.to_owned(), props));
                    }
                    _ => return Err(Error::InvalidFormat(String::from("select_attributes")))
                }
            }

            attrs
        }
        _ => return Err(Error::InvalidFormat(String::from("select_attributes is not an object")))
    };

    Ok(r)
}

#[cfg(test)]
mod test {
    use crate::config::config::{Connection, ExpectedRows};

    #[test]
    fn parse() {
        let data = r#"{
"attributes": [
    {
        "connection": "postgres",
        "query": "select * from users where id = '__PID__'",
        "expected_rows": "single",
        "select_attributes": {
            "fn": ["Type::String", "!ConvertName::firstname", "!Audit"],
            "ln": ["Type::String", "!ConvertName::lastname", "!Audit"],
            "currency": ["Type::String"],
            "age": ["Type::Number"]
        }
    },
    {
        "connection": "mysql",
        "query": "select * from org where user_id = '__PID__'",
        "expected_rows": "multiple",
        "select_attributes": {
            "manager": ["Type::String", "!ConvertName::managers"]
        }
    }
]}"#.as_bytes();
        let res = super::parse(data).unwrap();
        assert_eq!(res.attr_groups.len(), 1);

        let (attr, groups) = res.attr_groups.get(0).unwrap();
        assert_eq!(attr, "attributes");
        assert_eq!(groups.len(), 2);

        let gr1 = groups.get(0).unwrap();
        assert_eq!(gr1.conn, Connection::PostgresSQL, "invalid connection");
        assert_eq!(gr1.query, "select * from users where id = '__PID__'");
        assert_eq!(gr1.exp_rows, ExpectedRows::Single);
        assert_eq!(gr1.select_attrs.len(), 4);

        let gr2 = groups.get(1).unwrap();
        assert_eq!(gr2.conn, Connection::MySQL, "invalid connection");
        assert_eq!(gr2.query, "select * from org where user_id = '__PID__'");
        assert_eq!(gr2.exp_rows, ExpectedRows::Multiple);
        assert_eq!(gr2.select_attrs.len(), 1);
    }
}