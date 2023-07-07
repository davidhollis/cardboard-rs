#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub struct Fill {
    // TODO: expand definition to support different fill types, and provide
    //       custom knuffel::Decode to flatten it
    #[knuffel(child, unwrap(argument))]
    pub r#type: String,
}
