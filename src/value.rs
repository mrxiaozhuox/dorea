use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, tag_no_case, take_till1, take_while_m_n},
    character::complete::multispace0,
    combinator::{map, peek, value as n_value},
    error::context,
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair},
    IResult,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataValue {
    /// None Value
    ///
    /// Just use for deserialize.
    None,

    /// String Value
    ///
    /// ```
    /// DataValue::String("hello world".to_string());
    /// ```
    String(String),

    /// Number Value
    ///
    /// ```
    /// DataValue::Number(10_f64);
    /// ```
    Number(f64),

    /// Boolean Value
    ///
    /// ```
    /// DataValue::Boolean(true);
    /// ```
    Boolean(bool),

    /// List Value
    ///
    /// ```
    /// DataValue::List(vec![DataValue::Integer(1), DataValue::Integer(2), DataValue::Integer(3)]);
    /// ```
    List(Vec<DataValue>),

    /// Dict Value
    ///
    /// ```
    /// DataValue::List(HashMap::new());
    /// ```
    Dict(HashMap<String, DataValue>),

    /// Tuple Value
    ///
    /// ```
    /// DataValue::Tuple((DataValue::Boolean(true), DataValue::Boolean(false)));
    /// ```
    Tuple((Box<DataValue>, Box<DataValue>)),

    /// Undefined Value
    ///
    /// just use for data remove
    Undefined,
}

impl std::string::ToString for DataValue {
    fn to_string(&self) -> String {
        match self {
            DataValue::None => "none".to_string(),
            DataValue::String(s) => format!("\"{}\"", s),
            DataValue::Number(n) => n.to_string(),
            DataValue::Boolean(bool) => match bool {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            DataValue::List(l) => {
                let mut res = String::from("[");

                for v in l {
                    res += &format!("{},", v.to_string());
                }

                res = res[..res.len() - 1].to_string();
                res += "]";

                res
            }
            DataValue::Dict(d) => {
                let mut res = String::from("{");

                for v in d {
                    res += &format!("\"{}\":{},", v.0, v.1.to_string());
                }

                res = res[..res.len() - 1].to_string();
                res += "}";

                res
            }
            DataValue::Tuple(v) => {
                let first = v.0.to_string();
                let second = v.1.to_string();

                format!("({},{})", first, second)
            }
            DataValue::Undefined => "undefined".to_string(),
        }
    }
}

impl DataValue {
    pub fn from(data: &str) -> Self {
        match ValueParser::parse(data) {
            Ok((_, v)) => v,
            Err(_) => Self::None,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            DataValue::None => 0,
            DataValue::String(str) => str.len(),
            DataValue::Number(_) => 8,
            DataValue::Boolean(_) => 1,

            DataValue::List(list) => {
                let mut result = 0;

                for item in list {
                    result += item.size();
                }

                result
            }
            DataValue::Dict(dict) => {
                let mut result = 0;

                for item in dict {
                    result += item.1.size();
                }

                result
            }

            DataValue::Tuple(tuple) => tuple.0.size() + tuple.1.size(),

            DataValue::Undefined => 0,
        }
    }
}

struct ValueParser {}
impl ValueParser {
    fn normal(message: &str) -> IResult<&str, &str> {
        take_till1(|c: char| c == '\\' || c == '"' || c.is_ascii_control())(message)
    }

    fn escapable(i: &str) -> IResult<&str, &str> {
        context(
            "escaped",
            alt((
                tag("\""),
                tag("\\"),
                tag("/"),
                tag("b"),
                tag("f"),
                tag("n"),
                tag("r"),
                tag("t"),
                ValueParser::parse_hex,
            )),
        )(i)
    }

    fn string_format(message: &str) -> IResult<&str, &str> {
        escaped(ValueParser::normal, '\\', ValueParser::escapable)(message)
    }

    fn parse_hex(message: &str) -> IResult<&str, &str> {
        context(
            "hex string",
            preceded(
                peek(tag("u")),
                take_while_m_n(5, 5, |c: char| c.is_ascii_hexdigit() || c == 'u'),
            ),
        )(message)
    }

    fn parse_string(message: &str) -> IResult<&str, &str> {
        context(
            "string",
            alt((
                tag("\"\""),
                delimited(tag("\""), ValueParser::string_format, tag("\"")),
            )),
        )(message)
    }

    fn parse_number(message: &str) -> IResult<&str, f64> {
        double(message)
    }

    fn parse_boolean(message: &str) -> IResult<&str, bool> {
        let parse_true = n_value(true, tag_no_case("true"));
        let parse_false = n_value(false, tag_no_case("false"));
        alt((parse_true, parse_false))(message)
    }

    fn parse_list(message: &str) -> IResult<&str, Vec<DataValue>> {
        context(
            "list",
            delimited(
                tag("["),
                separated_list0(
                    tag(","),
                    delimited(multispace0, ValueParser::parse, multispace0),
                ),
                tag("]"),
            ),
        )(message)
    }

    fn parse_dict(message: &str) -> IResult<&str, HashMap<String, DataValue>> {
        context(
            "object",
            delimited(
                tag("{"),
                map(
                    separated_list0(
                        tag(","),
                        separated_pair(
                            delimited(multispace0, ValueParser::parse_string, multispace0),
                            tag(":"),
                            delimited(multispace0, ValueParser::parse, multispace0),
                        ),
                    ),
                    |tuple_vec: Vec<(&str, DataValue)>| {
                        tuple_vec
                            .into_iter()
                            .map(|(k, v)| (String::from(k), v))
                            .collect()
                    },
                ),
                tag("}"),
            ),
        )(message)
    }

    fn parse_tuple(message: &str) -> IResult<&str, (Box<DataValue>, Box<DataValue>)> {
        context(
            "tuple",
            delimited(
                tag("("),
                map(
                    separated_pair(
                        delimited(multispace0, ValueParser::parse, multispace0),
                        tag(","),
                        delimited(multispace0, ValueParser::parse, multispace0),
                    ),
                    |pair: (DataValue, DataValue)| (Box::new(pair.0), Box::new(pair.1)),
                ),
                tag(")"),
            ),
        )(message)
    }

    fn parse(message: &str) -> IResult<&str, DataValue> {
        context(
            "value",
            delimited(
                multispace0,
                alt((
                    map(ValueParser::parse_number, DataValue::Number),
                    map(ValueParser::parse_boolean, DataValue::Boolean),
                    map(ValueParser::parse_string, |s| {
                        DataValue::String(String::from(s))
                    }),
                    map(ValueParser::parse_list, DataValue::List),
                    map(ValueParser::parse_dict, DataValue::Dict),
                    map(ValueParser::parse_tuple, DataValue::Tuple),
                )),
                multispace0,
            ),
        )(message)
    }
}

#[cfg(test)]
mod test {

    use crate::value::{DataValue, ValueParser};

    #[test]
    fn list() {
        let value = "[1, 2, 3, 4, 5, 6]";
        assert_eq!(
            ValueParser::parse(value),
            Ok((
                "",
                DataValue::List(vec![
                    DataValue::Number(1_f64),
                    DataValue::Number(2_f64),
                    DataValue::Number(3_f64),
                    DataValue::Number(4_f64),
                    DataValue::Number(5_f64),
                    DataValue::Number(6_f64),
                ])
            ))
        );
    }

    #[test]
    fn tuple() {
        let value = "(true,1)";
        assert_eq!(
            ValueParser::parse(value),
            Ok((
                "",
                DataValue::Tuple((
                    Box::new(DataValue::Boolean(true)),
                    Box::new(DataValue::Number(1_f64))
                ))
            ))
        );
    }
}
