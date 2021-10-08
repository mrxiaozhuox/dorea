pub type DataValue = doson::DataValue;

pub fn value_ser_string(value: DataValue, style: &str) -> String 
{
    if style == "json" { return value.to_json(); }
    return value.to_string();
}