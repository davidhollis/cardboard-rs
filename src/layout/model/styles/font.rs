#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Font {
    // TODO: expand available attributes
    #[knuffel(argument)]
    pub name: String,
}