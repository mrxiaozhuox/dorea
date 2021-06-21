#[macro_export]
macro_rules! database_type {
    (
        @$key:ident : $value:expr
    ) => {
        DataValue::$key($value)
    };
}