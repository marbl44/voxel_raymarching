use serde::ser::Error;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

pub fn read_json_file<T>(file_path: &str) -> Result<T, serde_json::Error>
where
    T: for<'a> Deserialize<'a>,
    T: for<'a> Default,
{
    match File::open(file_path) {
        Ok(mut file) => {
            let mut json_data = String::new();
            file.read_to_string(&mut json_data).unwrap_or_default();
            let result = serde_json::from_str::<T>(&json_data)?;
            Ok(result)
        }
        Err(_) => Err(serde_json::Error::custom("No file")),
    }
}
