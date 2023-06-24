#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Stroke {
    // TODO: expand defintion, provide custom decoder, add joint style
    #[knuffel(argument)]
    pub width: usize,
}