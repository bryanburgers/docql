use super::schema;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
};

pub struct TypeRefRenderer<'a> {
    _schema: &'a schema::Schema,
}

impl<'a> TypeRefRenderer<'a> {
    pub fn new(schema: &'a schema::Schema) -> Self {
        Self { _schema: schema }
    }

    pub fn render_type_ref(
        &self,
        type_ref: &schema::TypeRef,
        out: &mut dyn Output,
    ) -> Result<(), std::io::Error> {
        match &type_ref.kind {
            schema::Kind::List => {
                out.write("[")?;
                if let Some(ref of_type) = type_ref.of_type {
                    self.render_type_ref(of_type, out)?;
                } else {
                    out.write("?")?;
                }
                out.write("]")?;
            }
            schema::Kind::NonNull => {
                if let Some(ref of_type) = type_ref.of_type {
                    self.render_type_ref(of_type, out)?;
                } else {
                    out.write("?")?;
                }
                out.write("!")?;
            }
            k => {
                let o = format!(
                    r#"<a class="{}" href="{}.{}.html">{}</a>"#,
                    k.prefix(),
                    k.prefix(),
                    type_ref.name.as_deref().unwrap(),
                    type_ref.name.as_deref().unwrap()
                );
                out.write(&o)?;
            }
        }

        Ok(())
    }
}

impl HelperDef for TypeRefRenderer<'_> {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).unwrap();

        let type_ref: schema::TypeRef = serde_json::from_value(param.value().clone())?;

        self.render_type_ref(&type_ref, out)?;

        Ok(())
    }
}

#[derive(Default)]
pub struct Docblock;

impl HelperDef for Docblock {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        use pulldown_cmark::{html, Options, Parser};

        let param = h.param(0).unwrap();

        let doc = param
            .value()
            .as_str()
            .ok_or_else(|| RenderError::new("Parameter to docblock was not a string"))?;

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(doc, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        out.write(&html_output)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct Kind;

impl HelperDef for Kind {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).unwrap();

        let kind: schema::Kind = serde_json::from_value(param.value().clone())?;

        out.write(&kind.prefix())?;
        Ok(())
    }
}
