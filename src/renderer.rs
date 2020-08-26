use super::{handlebars_helpers, schema, Result};
use chrono::NaiveDate;
use serde::Serialize;

pub struct Renderer<'a> {
    schema_name: String,
    date: NaiveDate,
    schema: &'a schema::Schema,
    handlebars: handlebars::Handlebars<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(schema_name: String, date: NaiveDate, schema: &'a schema::Schema) -> Result<Self> {
        let mut handlebars = handlebars::Handlebars::new();
        handlebars
            .register_template_string("layout", include_str!("templates/layout.handlebars"))?;
        handlebars.register_template_string("index", include_str!("templates/index.handlebars"))?;
        handlebars
            .register_template_string("object", include_str!("templates/object.handlebars"))?;
        handlebars.register_template_string(
            "input_object",
            include_str!("templates/input_object.handlebars"),
        )?;
        handlebars
            .register_template_string("scalar", include_str!("templates/scalar.handlebars"))?;
        handlebars.register_template_string("enum", include_str!("templates/enum.handlebars"))?;
        handlebars.register_template_string(
            "interface",
            include_str!("templates/interface.handlebars"),
        )?;
        handlebars.register_template_string("union", include_str!("templates/union.handlebars"))?;

        handlebars.register_template_string(
            "fields",
            include_str!("templates/partials/fields.handlebars"),
        )?;
        handlebars.register_template_string(
            "possible_types",
            include_str!("templates/partials/possible_types.handlebars"),
        )?;
        handlebars
            .register_template_string("uses", include_str!("templates/partials/uses.handlebars"))?;

        handlebars.register_helper(
            "t",
            Box::new(handlebars_helpers::TypeRefRenderer::new(&schema)),
        );
        handlebars.register_helper(
            "docblock",
            Box::new(handlebars_helpers::Docblock::default()),
        );
        handlebars.register_helper("kind", Box::new(handlebars_helpers::Kind::default()));

        Ok(Self {
            schema_name,
            date,
            schema,
            handlebars,
        })
    }
}

impl Renderer<'_> {
    pub fn render_index(&self) -> Result<String> {
        self.render(
            "index",
            &self.schema_name,
            &IndexContext::new(&self.schema_name, self.schema),
        )
    }

    pub fn render_object(&self, object: &schema::FullType) -> Result<String> {
        self.render(
            "object",
            &object.name,
            &ObjectContext::new(&self.schema_name, object, self.schema.find_uses(object)),
        )
    }

    pub fn render_input_object(&self, input_object: &schema::FullType) -> Result<String> {
        self.render(
            "input_object",
            &input_object.name,
            &InputObjectContext::new(
                &self.schema_name,
                input_object,
                self.schema.find_uses(input_object),
            ),
        )
    }

    pub fn render_scalar(&self, scalar: &schema::FullType) -> Result<String> {
        self.render(
            "scalar",
            &scalar.name,
            &ScalarContext::new(&self.schema_name, scalar, self.schema.find_uses(scalar)),
        )
    }

    pub fn render_enum(&self, enum_type: &schema::FullType) -> Result<String> {
        self.render(
            "enum",
            &enum_type.name,
            &EnumContext::new(
                &self.schema_name,
                enum_type,
                self.schema.find_uses(enum_type),
            ),
        )
    }

    pub fn render_interface(&self, interface: &schema::FullType) -> Result<String> {
        self.render(
            "interface",
            &interface.name,
            &InterfaceContext::new(
                &self.schema_name,
                interface,
                self.schema.find_uses(interface),
            ),
        )
    }

    pub fn render_union(&self, union: &schema::FullType) -> Result<String> {
        self.render(
            "union",
            &union.name,
            &UnionContext::new(&self.schema_name, union, self.schema.find_uses(union)),
        )
    }

    #[inline]
    fn render<T>(&self, template: &str, title: &str, t: &T) -> Result<String>
    where
        T: Serialize,
    {
        let rendered = self.handlebars.render(template, &t)?;
        let html = self.handlebars.render(
            "layout",
            &LayoutContext {
                title: title,
                content: &rendered,
                date_iso: self.date.format("%Y-%m-%d").to_string(),
                date_human: self.date.format("%-e %b %Y").to_string(),
            },
        )?;
        Ok(html)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutContext<'a> {
    title: &'a str,
    content: &'a str,
    date_iso: String,
    date_human: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IndexContext<'a> {
    schema_name: &'a str,
    query_type: &'a str,
    mutation_type: &'a str,
}

impl<'a> IndexContext<'a> {
    fn new(schema_name: &'a str, schema: &'a schema::Schema) -> Self {
        Self {
            schema_name,
            query_type: &schema.query_type.name,
            mutation_type: &schema.mutation_type.name,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ObjectContext<'a> {
    schema_name: &'a str,
    object: &'a schema::FullType,
    uses: Vec<schema::TypeUse<'a>>,
}

impl<'a> ObjectContext<'a> {
    fn new(
        schema_name: &'a str,
        object: &'a schema::FullType,
        uses: Vec<schema::TypeUse<'a>>,
    ) -> Self {
        Self {
            schema_name,
            object,
            uses,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InputObjectContext<'a> {
    schema_name: &'a str,
    input_object: &'a schema::FullType,
    uses: Vec<schema::TypeUse<'a>>,
}

impl<'a> InputObjectContext<'a> {
    fn new(
        schema_name: &'a str,
        input_object: &'a schema::FullType,
        uses: Vec<schema::TypeUse<'a>>,
    ) -> Self {
        Self {
            schema_name,
            input_object,
            uses,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScalarContext<'a> {
    schema_name: &'a str,
    scalar: &'a schema::FullType,
    uses: Vec<schema::TypeUse<'a>>,
}

impl<'a> ScalarContext<'a> {
    fn new(
        schema_name: &'a str,
        scalar: &'a schema::FullType,
        uses: Vec<schema::TypeUse<'a>>,
    ) -> Self {
        Self {
            schema_name,
            scalar,
            uses,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumContext<'a> {
    schema_name: &'a str,
    #[serde(rename = "enum")]
    enum_type: &'a schema::FullType,
    uses: Vec<schema::TypeUse<'a>>,
}

impl<'a> EnumContext<'a> {
    fn new(
        schema_name: &'a str,
        enum_type: &'a schema::FullType,
        uses: Vec<schema::TypeUse<'a>>,
    ) -> Self {
        Self {
            schema_name,
            enum_type,
            uses,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InterfaceContext<'a> {
    schema_name: &'a str,
    interface: &'a schema::FullType,
    uses: Vec<schema::TypeUse<'a>>,
}

impl<'a> InterfaceContext<'a> {
    fn new(
        schema_name: &'a str,
        interface: &'a schema::FullType,
        uses: Vec<schema::TypeUse<'a>>,
    ) -> Self {
        Self {
            schema_name,
            interface,
            uses,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UnionContext<'a> {
    schema_name: &'a str,
    union: &'a schema::FullType,
    uses: Vec<schema::TypeUse<'a>>,
}

impl<'a> UnionContext<'a> {
    fn new(
        schema_name: &'a str,
        union: &'a schema::FullType,
        uses: Vec<schema::TypeUse<'a>>,
    ) -> Self {
        Self {
            schema_name,
            union,
            uses,
        }
    }
}
