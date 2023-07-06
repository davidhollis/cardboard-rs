use knuffel::{ast::{Value, Literal}, decode::Kind, span::Spanned};

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Geometry {
    #[knuffel(child, unwrap(argument))]
    pub width: usize,
    #[knuffel(child, unwrap(argument))]
    pub height: usize,
    #[knuffel(child)]
    pub cut: Insets,
    #[knuffel(child)]
    pub safe: Insets,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Insets {
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub left: usize,
}

impl<S> knuffel::Decode<S> for Insets where S: knuffel::traits::ErrorSpan {
    fn decode_node(node: &knuffel::ast::SpannedNode<S>, _ctx: &mut knuffel::decode::Context<S>)
        -> Result<Self, knuffel::errors::DecodeError<S>> {
        match node.arguments.as_slice() {
            [Value { literal, .. }] => Ok(Insets::uniform(extract_integer(literal)?)),
            [
                Value { literal: top, .. },
                Value { literal: right, .. },
                Value { literal: bottom, .. },
                Value { literal: left, .. },
            ] => Ok(Insets {
                top: extract_integer(top)?,
                right: extract_integer(right)?,
                bottom: extract_integer(bottom)?,
                left: extract_integer(left)?,
            }),
            _ => Err(knuffel::errors::DecodeError::conversion(node, "Invalid number of arguments for insets. Expected either 1 or 4."))
        }
    }
}

impl Insets {
    pub fn uniform(size: usize) -> Insets {
        Insets { top: size, right: size, bottom: size, left: size }
    }
}

fn extract_integer<S>(literal: &Spanned<Literal, S>) -> Result<usize, knuffel::errors::DecodeError<S>>
    where S: knuffel::traits::ErrorSpan {
    match **literal {
        Literal::Int(ref raw_integer) =>
            Ok(
                raw_integer
                .try_into()
                .map_err(|err| knuffel::errors::DecodeError::conversion(&literal, err))?
            ),
        _ => Err(knuffel::errors::DecodeError::scalar_kind(Kind::Int, &literal))
    }
}