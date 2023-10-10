use std::collections::HashMap;
use std::error::Error;
use serde_json::Value;
use crate::config::config::Property::{Audit, ConvertName, ReturnAttribute, Type as PropertyType};
use crate::config::config::Type::{Boolean, JSON, Number, String as TypeString};

#[derive(Hash, Eq, PartialEq, Debug)]
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

pub enum Property {
    Type(Type),
    ConvertName(String),
    Audit,
    ReturnAttribute(String),
}

pub struct Config {
    attr_groups: Vec<(String,Vec<AttributeGroup>)>
}

impl Config {
    pub fn new(attr_groups: Vec<(String,Vec<AttributeGroup>)>) -> Self {
        Self { attr_groups }
    }
}

pub struct AttributeGroup {
    conn: Connection,
    query: String,
    exp_rows: ExpectedRows,
    select_attrs: Vec<(String, Vec<Property>)>
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

pub fn parse(data: &[u8]) -> Result<Config, Box<dyn Error>> {
    let raw: HashMap<String, Value> = serde_json::from_slice(data)?;
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
                        _=> return Err("group is not an object".into())
                    }
                }
            }
            _ => return Err("not an array".into())
        }
        groups.push((group_name.to_owned(), props))
    }

    Ok(Config::new(groups))
}

fn parse_connection(value: &Value) -> Result<Connection, Box<dyn Error>> {
    let r = match value {
        Value::String(conn) => {
            match conn.as_str() {
                "postgresql" | "postgres" => Connection::PostgresSQL,
                "mysql" => Connection::MySQL,
                "mongodb" => Connection::MongoDB,
                _ =>return Err("unknown connection".into())
            }
        }
        _ => return Err("connection is not a string".into())
    };

    Ok(r)
}

fn parse_query(value: &Value) -> Result<String, Box<dyn Error>> {
    let r = match value {
        Value::String(query) => query.to_string(),
        _ => return Err("invalid query".into())
    };

    Ok(r)
}

fn parse_exp_rows(value: &Value) -> Result<ExpectedRows, Box<dyn Error>> {
    let r = match value {
        Value::String(exp) => match exp.as_str() {
            "single" => ExpectedRows::Single,
            "multiple" => ExpectedRows::Multiple,
            _ => return Err("invalid exp rows value".into())
        }
        _ => return Err("invalid exp rows".into())
    };

    Ok(r)
}

fn parse_select_attributes(value: &Value) -> Result<Vec<(String, Vec<Property>)>, Box<dyn Error>> {
    let r = match value {
        Value::Object(obj) => {
            let mut attrs = vec![];
            for (k, v) in obj {
                let name = k;
                match v {
                    Value::Array(values) => {
                        let mut props = vec![];
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
                                                _=> return Err("invalid type".into())
                                            };

                                            props.push(PropertyType(t));
                                        },
                                        "!ConvertName" => props.push(ConvertName(attr_value.to_string())),
                                        "ReturnAttribute" => props.push(ReturnAttribute(attr_value.to_string())),
                                        "!Audit" => props.push(Audit),
                                        _=> return Err("invalid attr type".into())
                                    }
                                }
                                _ => return Err("invalid property".into())
                            }
                        }
                        attrs.push((name.to_owned(), props));
                    }
                    _=> return Err("invalid select_attrs".into())
                }
            }

            attrs
        }
        _ => return Err("select_attrs is not an object".into())
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