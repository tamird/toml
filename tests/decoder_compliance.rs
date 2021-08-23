fn main() {
    let decoder = Decoder;
    let mut harness = toml_test_harness::DecoderHarness::new(decoder);
    harness
        .ignore([
            "valid/inline-table/key-dotted.toml",
            "valid/key/dotted.toml",
            "valid/key/numeric-dotted.toml",
            "valid/string/multiline-quotes.toml",
        ])
        .unwrap();
    harness.test();
}

struct Decoder;

impl toml_test_harness::Decoder for Decoder {
    fn name(&self) -> &str {
        "toml_edit"
    }

    fn decode(&self, data: &[u8]) -> Result<toml_test_harness::Encoded, toml_test_harness::Error> {
        let data = std::str::from_utf8(data).map_err(toml_test_harness::Error::new)?;
        let document = data
            .parse::<toml_edit::Document>()
            .map_err(toml_test_harness::Error::new)?;
        document_to_encoded(&document)
    }
}

fn document_to_encoded(
    value: &toml_edit::Document,
) -> Result<toml_test_harness::Encoded, toml_test_harness::Error> {
    item_to_encoded(&value.root)
}

fn item_to_encoded(
    value: &toml_edit::Item,
) -> Result<toml_test_harness::Encoded, toml_test_harness::Error> {
    match value {
        toml_edit::Item::None => unreachable!("No nones"),
        toml_edit::Item::Value(v) => value_to_encoded(v),
        toml_edit::Item::Table(v) => table_to_encoded(v),
        toml_edit::Item::ArrayOfTables(v) => {
            let v: Result<_, toml_test_harness::Error> = v.iter().map(table_to_encoded).collect();
            Ok(toml_test_harness::Encoded::Array(v?))
        }
    }
}

fn value_to_encoded(
    value: &toml_edit::Value,
) -> Result<toml_test_harness::Encoded, toml_test_harness::Error> {
    match value {
        toml_edit::Value::Integer(v) => Ok(toml_test_harness::Encoded::Value(
            toml_test_harness::EncodedValue::from(*v.value()),
        )),
        toml_edit::Value::String(v) => Ok(toml_test_harness::Encoded::Value(
            toml_test_harness::EncodedValue::from(v.value()),
        )),
        toml_edit::Value::Float(v) => Ok(toml_test_harness::Encoded::Value(
            toml_test_harness::EncodedValue::from(*v.value()),
        )),
        toml_edit::Value::DateTime(v) => match *v.value() {
            toml_edit::DateTime::OffsetDateTime(v) => Ok(toml_test_harness::Encoded::Value(
                toml_test_harness::EncodedValue::Datetime(v.to_rfc3339()),
            )),
            toml_edit::DateTime::LocalDateTime(v) => Ok(toml_test_harness::Encoded::Value(
                toml_test_harness::EncodedValue::DatetimeLocal(v.to_string()),
            )),
            toml_edit::DateTime::LocalDate(v) => Ok(toml_test_harness::Encoded::Value(
                toml_test_harness::EncodedValue::DateLocal(v.to_string()),
            )),
            toml_edit::DateTime::LocalTime(v) => Ok(toml_test_harness::Encoded::Value(
                toml_test_harness::EncodedValue::TimeLocal(v.to_string()),
            )),
        },
        toml_edit::Value::Boolean(v) => Ok(toml_test_harness::Encoded::Value(
            toml_test_harness::EncodedValue::from(*v.value()),
        )),
        toml_edit::Value::Array(v) => {
            let v: Result<_, toml_test_harness::Error> = v.iter().map(value_to_encoded).collect();
            Ok(toml_test_harness::Encoded::Array(v?))
        }
        toml_edit::Value::InlineTable(v) => inline_table_to_encoded(v),
    }
}

fn table_to_encoded(
    value: &toml_edit::Table,
) -> Result<toml_test_harness::Encoded, toml_test_harness::Error> {
    let table: Result<_, toml_test_harness::Error> = value
        .iter()
        .map(|(k, v)| {
            let k = k.to_owned();
            let v = item_to_encoded(v)?;
            Ok((k, v))
        })
        .collect();
    Ok(toml_test_harness::Encoded::Table(table?))
}

fn inline_table_to_encoded(
    value: &toml_edit::InlineTable,
) -> Result<toml_test_harness::Encoded, toml_test_harness::Error> {
    let table: Result<_, toml_test_harness::Error> = value
        .iter()
        .map(|(k, v)| {
            let k = k.to_owned();
            let v = value_to_encoded(v)?;
            Ok((k, v))
        })
        .collect();
    Ok(toml_test_harness::Encoded::Table(table?))
}
