pub type DataValue = doson::DataValue;

// 这里使用了 DOSON 数据拓展集
// http://github.com/doreadb/doson.git
// Doson 是 Json 的一种拓展格式，在Json的基础上添加了一些新的类型
pub fn value_ser_string(value: DataValue, style: &str) -> String {
    if style == "json" {
        return value.to_json();
    }
    value.to_string()
}
