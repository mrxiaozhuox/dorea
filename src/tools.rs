//! export useful tool functions.
//!
//! Author: ZhuoEr Liu <mrxzx@qq.com>


/// you can use this function to parse "DataValue"
///
/// input -> Message | output -> Option<DataValue>
///
/// example:
///
/// ```rust
/// use dorea::tools::parse_value_type;
/// use dorea::server::DataValue;
/// use std::collections::HashMap;
///
/// let parse = parse_value_type("{\"foo\":\"bar\"}".to_string());
///
/// let mut list: HashMap<String,String> = HashMap::new();
/// list.insert("foo".to_string(),"bar".to_string());
///
/// assert_eq!(DataValue::Dict(list), parse);
/// ```
///
pub use crate::handle::parse_value_type;