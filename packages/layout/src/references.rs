use super::starlark_repr::StarlarkContainer;

pub enum LiteralOrReference {
    Literal { value: String },
    Reference { path: String },
}

impl LiteralOrReference {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<LiteralOrReference> {
        let val_type = val.value().get_type();

        match val_type {
            "string" => Ok(LiteralOrReference::Literal {
                value: val.value().to_string(),
            }),
            _ => {
                val.validate_type("ref")?;

                Ok(LiteralOrReference::Reference {
                    path: val.extract_string("literal", "path")?.to_string(),
                })
            },
        }
    }
}