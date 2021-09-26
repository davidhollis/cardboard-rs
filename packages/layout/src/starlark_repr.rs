use starlark::values::{ Heap, Value };
use std::convert::TryFrom;

pub struct StarlarkContainer<'v>(Value<'v>, &'v Heap);

impl<'v> StarlarkContainer<'v> {
    pub fn new(v: Value<'v>, h: &'v Heap) -> StarlarkContainer<'v> {
        StarlarkContainer(v, h)
    }

    pub fn value(&self) -> Value<'v> { self.0 }

    pub fn validate_type(&self, type_name: &str) -> anyhow::Result<()> {
        let (_, val_type_attr) = self.0.get_attr("type", self.1)
            .ok_or(anyhow!("Malformed {}: no type attribute.", type_name))?;
        let val_type = val_type_attr.unpack_str()
            .ok_or(anyhow!("Malformed {}: type is not a string.", type_name))?;
        
        if val_type == type_name {
            Ok(())
        } else {
            Err(anyhow!(
                "Malformed {}: incorrect type (expected: '{}'; got: '{}').",
                type_name, type_name, val_type
            ))
        }
    }

    pub fn extract_value(&self, type_name: &str, field: &str) -> anyhow::Result<StarlarkContainer<'v>> {
        let (_, attr) = self.0.get_attr(field, self.1)
            .ok_or(anyhow!("Malformed {}: no {} attribute.", type_name, field))?;
        Ok(StarlarkContainer(attr, self.1))
    }

    pub fn extract_string(&self, type_name: &str, field: &str) -> anyhow::Result<&'v str> {
        let (_, attr) = self.0.get_attr(field, self.1)
            .ok_or(anyhow!("Malformed {}: no {} attribute.", type_name, field))?;
        attr.unpack_str()
            .ok_or(anyhow!("Malformed {}: {} is not a string.", type_name, field))
    }

    pub fn extract_i16(&self, type_name: &str, field: &str) -> anyhow::Result<i16> {
        let (_, attr) = self.0.get_attr(field, self.1)
            .ok_or(anyhow!("Malformed {}: no {} attribute.", type_name, field))?;
        attr.unpack_int()
            .and_then(|i| i16::try_from(i).ok())
            .ok_or(anyhow!("Malformed {}: {} is not an i16.", type_name, field))
    }

    pub fn extract_u16(&self, type_name: &str, field: &str) -> anyhow::Result<u16> {
        let (_, attr) = self.0.get_attr(field, self.1)
            .ok_or(anyhow!("Malformed {}: no {} attribute.", type_name, field))?;
        attr.unpack_int()
            .and_then(|i| u16::try_from(i).ok())
            .ok_or(anyhow!("Malformed {}: {} is not a u16.", type_name, field))
    }
}