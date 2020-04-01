use std::io::Read;
use std::str::FromStr;

#[derive(Debug)]
pub struct Error(String);

impl<T: ToString> From<T> for Error {
    fn from(error: T) -> Self {
        Error(error.to_string())
    }
}

pub struct Program {
    pub data: Box<[i64]>,
}

impl Program {
    pub fn parse(mut input: impl Read) -> Result<Self, Error> {
        let mut data_string = String::new();
        input.read_to_string(&mut data_string)?;
        let data = data_string
            .split(',')
            .map(str::trim)
            .map(i64::from_str)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        Ok(Program { data })
    }
}
