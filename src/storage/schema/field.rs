use serde::{Deserialize, Serialize};

use crate::storage::schema::_type::Type;
use crate::storage::schema::encoding::Encoding;
use crate::storage::schema::error::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    name: String,
    _type: Type,
}

impl Field {
    pub fn build(name: String, _type: Type) -> Field {
        Field { name, _type }
    }

    pub fn get_type(&self) -> &Type {
        &self._type
    }
}

impl Encoding<Field> for Field {
    fn from_str(field_str: &str) -> Result<Field, Error> {
        let name_and_type: Vec<&str> = field_str.split_whitespace().collect();
        if name_and_type.len() != 2 {
            Err(Error::InvalidField(format!(
                "\n- Invalid field syntax. Expected \"column_name column_type\" Actual \"{}\"",
                field_str
            )))
        } else {
            Ok(Field {
                name: name_and_type[0].to_string(),
                _type: Type::from_str(name_and_type[1])?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_from_str_should_return_struct() {
        assert_eq!(
            Field::from_str("id INT").unwrap(),
            Field::build("id".to_string(), Type::Int)
        );
    }

    #[test]
    #[should_panic]
    fn field_from_str_should_panic_if_invalid_string() {
        Field::from_str("id INT fail").unwrap();
    }
}
