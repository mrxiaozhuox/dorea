use bytes::Bytes;
use std::collections::HashMap;

#[derive(Debug)]
pub struct FormData {
    pub data: Bytes,
    pub file_name: Option<String>,
}

#[allow(dead_code)]
impl FormData {
    pub fn is_file(&self) -> bool {
        self.file_name != None
    }

    pub fn text(&self) -> crate::Result<String> {
        let s = String::from_utf8(self.data.to_vec())?;
        Ok(s)
    }
}

#[allow(dead_code)]
pub async fn multipart(mut value: axum::extract::Multipart) -> HashMap<String, FormData> {
    let mut total = HashMap::new();
    while let Some(field) = value.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        let file_name = match field.file_name().clone() {
            None => None,
            Some(v) => Some(String::from(v)),
        };

        let data = field.bytes().await.unwrap().clone();

        total.insert(name, FormData { data, file_name });
    }

    total
}
