use crate::storage::page::encoding::Encoding;
use crate::storage::page::page_error::PageError;
use crate::storage::page::tuple_header::TupleHeader;
use crate::storage::schema::schema::Schema;

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    pub header: TupleHeader,
    pub data: Vec<u8>,
}

impl Tuple {
    pub fn build(schema: &Schema, nulls: &[u8], data: &[u8]) -> Result<Tuple, PageError> {
        if schema.tuple_size(Some(nulls)) != data.len() {
            Err(PageError::CorruptedTuple(format!(
                "Data {:?} with nulls {:?} don't match with given schema {:?}",
                data, nulls, schema
            )))
        } else {
            Ok(Tuple {
                header: TupleHeader::build(nulls),
                data: data.to_vec(),
            })
        }
    }
}

impl Encoding<Tuple> for Tuple {
    fn as_bytes(&self) -> Vec<u8> {
        let mut concat_bytes: Vec<u8> = Vec::new();
        concat_bytes.extend_from_slice(&self.header.as_bytes());
        concat_bytes.extend_from_slice(self.data.as_slice());
        concat_bytes
    }

    fn from_bytes(bytes: &[u8], schema: Option<&Schema>) -> Result<Tuple, PageError> {
        let schema = schema.ok_or(PageError::MissingSchema)?;
        let columns_total = schema.fields.len();
        let nulls = &bytes[1..(columns_total + 1)];
        Tuple::build(schema, &nulls, &bytes[(columns_total + 1)..])
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::schema::encoding::Encoding as SchemaEncoding;

    use super::*;

    fn get_test_schema() -> Schema {
        Schema::from_str("id BIGINT, cost FLOAT, available BOOLEAN, date TIMESTAMP").unwrap()
    }

    #[test]
    #[should_panic]
    fn build_should_panic_if_data_dont_match_schema() {
        Tuple::build(&get_test_schema(), &[0, 0, 1, 0], &[4; 33]).unwrap();
    }

    #[test]
    fn as_bytes_should_convert_tuple() {
        assert_eq!(
            Tuple::build(&get_test_schema(), &[0, 0, 1, 0], &[4; 32])
                .unwrap()
                .as_bytes(),
            vec![
                0, 0, 0, 1, 0, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4, 4,
            ]
        )
    }

    #[test]
    fn from_bytes_should_convert_bytes() {
        assert_eq!(
            Tuple::from_bytes(
                &[
                    0, 0, 0, 1, 0, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                ],
                Some(&get_test_schema()),
            )
            .unwrap(),
            Tuple::build(&get_test_schema(), &[0, 0, 1, 0], &[4; 32]).unwrap()
        )
    }
}
