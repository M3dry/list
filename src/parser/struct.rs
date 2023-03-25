use crate::tokenizer::{Keywords, Token};

use super::{error, r#type::Type, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub struct Struct {
    name: String,
    generics: Vec<String>,
    fields: StructFields,
}

impl TryFrom<&mut Parser> for Struct {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Struct", "Expected more tokens")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Struct",
                format!("Expected ParenOpen, got {next:#?}")
            ));
        }

        let next = value.pop_front_err("Struct", "Expected more tokens")?;
        if next != Token::Keyword(Keywords::Struct) {
            return Err(error!(
                "Struct",
                format!("Expected Struct keyword, got {next:#?}")
            ));
        }

        let next = value.pop_front_err("Struct", "Expected more tokens")?;
        let name = if let Token::Identifier(iden) = next {
            iden
        } else {
            return Err(error!(
                "Struct",
                format!("Expected identifier, got {next:#?}")
            ));
        };

        let mut generics = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("Struct", format!("Expected more tokens")))?;

            if let Token::Generic(_) = peek {
                generics.push(
                    if let Token::Generic(generic) = value.pop_front().unwrap() {
                        generic
                    } else {
                        panic!("wtf")
                    },
                )
            } else {
                break;
            }
        }

        let fields = error!(StructFields::try_from(&mut *value), "Struct")?;

        let next = value.pop_front_err("Struct", "Expected more tokens")?;
        if next != Token::ParenClose {
            return Err(error!(
                "Struct",
                format!("Expected ParenClose, got {next:#?}")
            ));
        }

        Ok(Struct {
            name,
            generics,
            fields,
        })
    }
}

impl ToString for Struct {
    fn to_string(&self) -> String {
        format!(
            "struct {}{} {}",
            self.name,
            if !self.generics.is_empty() {
                format!(
                    "<{}>",
                    &self.generics.iter().fold(String::new(), |str, generic| {
                        format!("{str}, {generic}")
                    })[2..]
                )
            } else {
                format!("")
            },
            self.fields.to_string()
        )
    }
}

#[derive(Debug)]
pub(crate) struct StructFields(Vec<StructField>);

impl TryFrom<&mut Parser> for StructFields {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("StructFields", "Expected more tokens")?;
        if next != Token::CurlyOpen {
            return Err(error!(
                "StructFields",
                format!("Expected CurlyOpen, got {next:#?}")
            ));
        }

        let mut fields = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("StructFields", format!("Expected more tokens")))?;
            if peek == &Token::CurlyClose {
                value.pop_front();
                break;
            }

            fields.push(error!(StructField::try_from(&mut *value), "StructFields")?);
        }

        Ok(StructFields(fields))
    }
}

impl ToString for StructFields {
    fn to_string(&self) -> String {
        format!(
            "{{{}}}",
            if !self.0.is_empty() {
                (&self.0.iter().fold(String::new(), |str, field| {
                    format!("{str}, {}", field.to_string())
                })[2..])
                    .to_string()
            } else {
                format!("")
            }
        )
    }
}

#[derive(Debug)]
struct StructField {
    name: String,
    r#type: Type,
}

impl TryFrom<&mut Parser> for StructField {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("StructField", "Expected more tokens")?;
        if let Token::Identifier(iden) = next {
            let name = iden;

            let next = value.pop_front_err("StructField", "Expected more tokens")?;
            if next != Token::Keyword(Keywords::Arrow) {
                return Err(error!(
                    "StructField",
                    format!("Expected arror keyword, got {next:#?}")
                ))?;
            }

            let r#type = error!(Type::try_from(&mut *value), "StructField")?;

            Ok(StructField { name, r#type })
        } else {
            Err(error!(
                "StructField",
                format!("Expected identifier, got {next:#?}")
            ))
        }
    }
}

impl ToString for StructField {
    fn to_string(&self) -> String {
        format!("{}: {}", self.name, self.r#type.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::snapshot;

    snapshot!(test_struct, Struct::try_from, "struct.lt");
    snapshot!(test_struct_rust, Struct::try_from, "struct.lt", rust);
}
