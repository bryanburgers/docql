use super::schema;
use serde::{
    ser::{SerializeSeq, Serializer},
    Serialize,
};

#[derive(Serialize)]
pub struct SearchIndex(Vec<SearchIndexItem>);

impl SearchIndex {
    pub fn build(schema: &schema::Schema) -> Self {
        let mut items = Vec::new();

        for typ in &schema.types {
            Self::build_type(typ, &mut items);
        }

        SearchIndex(items)
    }

    fn build_type(typ: &schema::FullType, items: &mut Vec<SearchIndexItem>) {
        let name = &typ.name;
        let kind = match typ.kind {
            schema::Kind::Union => "union",
            schema::Kind::Interface => "interface",
            schema::Kind::Object => "object",
            schema::Kind::InputObject => "input_object",
            schema::Kind::Scalar => "scalar",
            schema::Kind::Enum => "enum",
            schema::Kind::List | schema::Kind::NonNull => return,
        };

        let item = SearchIndexItem {
            index: vec![name.to_lowercase()],
            name: name.to_string(),
            kind: kind.to_string(),
            parent_name: None,
            parent_kind: None,
        };

        items.push(item);

        if let Some(ref fields) = typ.fields {
            for field in fields {
                Self::build_field(field, name, kind, items);
            }
        }
        if let Some(ref enum_values) = typ.enum_values {
            for enum_value in enum_values {
                Self::build_enum_value(enum_value, name, kind, items);
            }
        }
        if let Some(ref input_fields) = typ.input_fields {
            for input_field in input_fields {
                Self::build_input_field(input_field, name, kind, items);
            }
        }
    }

    fn build_field(
        field: &schema::Field,
        parent_name: &str,
        parent_kind: &str,
        items: &mut Vec<SearchIndexItem>,
    ) {
        let item = SearchIndexItem {
            index: vec![field.name.to_lowercase()],
            name: field.name.to_string(),
            kind: "field".to_string(),
            parent_name: Some(parent_name.to_string()),
            parent_kind: Some(parent_kind.to_string()),
        };

        items.push(item);
    }

    fn build_enum_value(
        enum_value: &schema::EnumValue,
        parent_name: &str,
        parent_kind: &str,
        items: &mut Vec<SearchIndexItem>,
    ) {
        let item = SearchIndexItem {
            index: vec![
                enum_value.name.to_lowercase(),
                enum_value.name.to_lowercase().replace("_", ""),
            ],
            name: enum_value.name.to_string(),
            kind: "enum_value".to_string(),
            parent_name: Some(parent_name.to_string()),
            parent_kind: Some(parent_kind.to_string()),
        };

        items.push(item);
    }

    fn build_input_field(
        input_field: &schema::InputValue,
        parent_name: &str,
        parent_kind: &str,
        items: &mut Vec<SearchIndexItem>,
    ) {
        let item = SearchIndexItem {
            index: vec![input_field.name.to_lowercase()],
            name: input_field.name.to_string(),
            kind: "input_field".to_string(),
            parent_name: Some(parent_name.to_string()),
            parent_kind: Some(parent_kind.to_string()),
        };

        items.push(item);
    }
}

pub struct SearchIndexItem {
    index: Vec<String>,
    name: String,
    kind: String,
    parent_name: Option<String>,
    parent_kind: Option<String>,
}

impl Serialize for SearchIndexItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element(&self.index)?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.kind)?;
        if let (Some(parent_name), Some(parent_kind)) =
            (self.parent_name.as_deref(), self.parent_kind.as_deref())
        {
            seq.serialize_element(parent_name)?;
            seq.serialize_element(parent_kind)?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_with_parent() {
        let item = SearchIndexItem {
            index: vec!["superadmin".to_string(), "super_admin".to_string()],
            name: "SUPER_ADMIN".to_string(),
            kind: "enumitem".to_string(),
            parent_name: Some("AccountType".to_string()),
            parent_kind: Some("enum".to_string()),
        };

        let value = serde_json::to_value(&item).unwrap();
        assert_eq!(
            value,
            json!([
                ["superadmin", "super_admin"],
                "SUPER_ADMIN",
                "enumitem",
                "AccountType",
                "enum"
            ])
        );
    }

    #[test]
    fn test_serialize_without_parent() {
        let item = SearchIndexItem {
            index: vec!["accounttype".to_string()],
            name: "AccountType".to_string(),
            kind: "enum".to_string(),
            parent_name: None,
            parent_kind: None,
        };

        let value = serde_json::to_value(&item).unwrap();
        assert_eq!(value, json!([["accounttype"], "AccountType", "enum"]));
    }
}
