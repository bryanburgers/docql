use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphQLResponse {
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "__schema")]
    pub schema: Schema,
}

#[derive(Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    pub query_type: RootTypeRef,
    pub mutation_type: RootTypeRef,
    pub types: Vec<FullType>,
}

impl Schema {
    pub fn _find_type(&self, type_ref: &TypeRef) -> Option<&FullType> {
        let type_ref_name = type_ref.name.as_ref()?;

        for typ in &self.types {
            if &typ.name == type_ref_name {
                return Some(typ);
            }
        }

        None
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct RootTypeRef {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct TypeRef {
    pub kind: Kind,
    pub name: Option<String>,
    pub of_type: Option<Box<TypeRef>>,
}

#[derive(Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct FullType {
    pub kind: Kind,
    pub name: String,
    pub description: Option<String>,
    pub fields: Option<Vec<Field>>,
    pub input_fields: Option<Vec<InputValue>>,
    pub interfaces: Option<Vec<TypeRef>>,
    pub enum_values: Option<Vec<EnumValue>>,
    pub possible_types: Option<Vec<TypeRef>>,
}

#[derive(Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Kind {
    NonNull,
    List,
    Object,
    InputObject,
    Union,
    Enum,
    Scalar,
    Interface,
}

impl Kind {
    pub fn prefix(&self) -> &str {
        match self {
            Self::NonNull => "non_null",
            Self::List => "list",
            Self::Object => "object",
            Self::InputObject => "input_object",
            Self::Union => "union",
            Self::Enum => "enum",
            Self::Scalar => "scalar",
            Self::Interface => "interface",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub args: Vec<InputValue>,
    #[serde(rename = "type")]
    pub typ: TypeRef,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct InputValue {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub typ: TypeRef,
    pub default_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue {
    pub name: String,
    pub description: Option<String>,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<String>,
}

impl Schema {
    pub fn find_uses(&self, full_type: &FullType) -> Vec<TypeUse> {
        let mut uses = Vec::new();

        for typ in &self.types {
            if let Some(ref fields) = typ.fields {
                for field in fields {
                    if self.is_use(full_type, &field.typ) {
                        uses.push(TypeUse::Field { typ, field: &field });
                    } else {
                        for arg in &field.args {
                            if self.is_use(full_type, &arg.typ) {
                                uses.push(TypeUse::Field { typ, field });
                                break;
                            }
                        }
                    }
                }
            }
            if let Some(ref input_fields) = typ.input_fields {
                for input_field in input_fields {
                    if self.is_use(full_type, &input_field.typ) {
                        uses.push(TypeUse::InputField {
                            typ,
                            input_field: &input_field,
                        });
                    }
                }
            }
            if let Some(ref possible_types) = typ.possible_types {
                for possible_type in possible_types {
                    if self.is_use(full_type, &possible_type) {
                        uses.push(TypeUse::PossibleType { typ });
                    }
                }
            }
        }

        uses.sort();
        uses
    }

    fn is_use(&self, full_type: &FullType, type_ref: &TypeRef) -> bool {
        if Some(&full_type.name) == type_ref.name.as_ref() {
            return true;
        }

        if let Some(ref of_type) = type_ref.of_type {
            match type_ref.kind {
                Kind::NonNull | Kind::List => self.is_use(full_type, of_type),
                _ => false,
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(tag = "use_type")]
pub enum TypeUse<'a> {
    /// The type is used as an input or an output in a field
    Field {
        #[serde(rename = "type")]
        typ: &'a FullType,
        field: &'a Field,
    },
    /// The type is used as an input field in another input object
    InputField {
        #[serde(rename = "type")]
        typ: &'a FullType,
        input_field: &'a InputValue,
    },
    /// The type is used as a possible type on an interface or an enumeration
    PossibleType {
        #[serde(rename = "type")]
        typ: &'a FullType,
    },
}
