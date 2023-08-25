use knuffel::{span::Spanned, ast::Literal};

pub fn extract_number_as_float<S>(literal: &Spanned<Literal, S>) -> Result<f32, knuffel::errors::DecodeError<S>>
where S: knuffel::traits::ErrorSpan {
    match **literal {
        Literal::Int(ref raw_integer) => {
            let integer_value: usize = raw_integer.try_into().map_err(|err| knuffel::errors::DecodeError::conversion(&literal, err))?;
            Ok(integer_value as f32)
        },
        Literal::Decimal(ref raw_decimal) => {
            Ok(
                raw_decimal
                .try_into()
                .map_err(|err| knuffel::errors::DecodeError::conversion(&literal, err))?
            )
        },
        _ => Err(knuffel::errors::DecodeError::scalar_kind(knuffel::decode::Kind::Decimal, &literal))
    }
}
