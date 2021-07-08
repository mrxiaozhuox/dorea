use std::collections::HashMap;

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
    Dict(HashMap<Vec<u8>, DataValue>),

    /// Boolean Value
    /// 
    /// ```
    /// DataValue::Tuple((DataValue::Boolean(true), DataValue::Boolean(false)));
    /// ```
    Tuple((Box<DataValue>, Box<DataValue>)),
}

impl DataValue { }