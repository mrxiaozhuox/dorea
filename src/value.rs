use std::collections::HashMap;

#[derive(Debug, Clone)]
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

    /// Integer Value
    /// 
    /// ```
    /// DataValue::Integer(10_i64);
    /// ```
    Integer(i64),

    /// Float Value
    /// 
    /// ```
    /// DataValue::Integer(3.14_f64);
    /// ```
    Float(f64),

    /// Boolean Value
    /// 
    /// ```
    /// DataValue::Boolean(true);
    /// ```
    Boolean(bool),

    /// Boolean Value
    /// 
    /// ```
    /// DataValue::List(vec![DataValue::Integer(1), DataValue::Integer(2), DataValue::Integer(3)]);
    /// ```
    List(Vec<DataValue>),

    /// Boolean Value
    /// 
    /// ```
    /// DataValue::List(HashMap::new());
    /// ```
    Dict(HashMap<String, DataValue>),

    /// Boolean Value
    /// 
    /// ```
    /// DataValue::Tuple((DataValue::Boolean(true), DataValue::Boolean(false)));
    /// ```
    Tuple((Box<DataValue>, Box<DataValue>)),
}

impl DataValue {
    pub fn size(&self) -> usize {
        match self {

            DataValue::None => 0,
            DataValue::String(str) => str.len(),
            DataValue::Integer(_) => 8,
            DataValue::Float(_) => 8,
            DataValue::Boolean(_) => 1,

            DataValue::List(list) => {

                let mut result = 0;

                for item in list {
                    result += item.size();
                }

                result
            },
            DataValue::Dict(dict) => {

                let mut result = 0;

                for item in dict {
                    result += item.1.size();
                }

                result

            },

            DataValue::Tuple(tuple) => { tuple.0.size() + tuple.1.size() },
        }
    }
}