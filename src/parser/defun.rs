use crate::tokenizer::{Keywords, Token};

use super::{
    args::ArgsTyped, error, exp::Exp, r#type::Type, Parser, ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub struct Defun {
    scope: Scope,
    name: String,
    args: ArgsTyped,
    return_type: Type,
    body: Exp,
}

impl TryFrom<&mut Parser> for Defun {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Defun")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Defun",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let scope = Scope::try_from(&mut *value).unwrap();

        let next = value.pop_front_err("Defun")?;
        if next != Token::Keyword(Keywords::Defun) {
            return Err(error!(
                "Defun",
                format!("Expected Defun keyword, got {next:#?}"),
            ));
        }

        let next = value.pop_front_err("Defun")?;
        let name = if let Token::Identifier(iden) = next {
            iden
        } else {
            return Err(error!(
                "Defun name",
                format!("Expected Identifier, got {next:#?}"),
            ));
        };

        let args = error!(ArgsTyped::try_from(&mut *value), "Defun")?;

        let next = value.pop_front_err("Defun")?;
        if next != Token::Keyword(Keywords::Arrow) {
            return Err(error!(
                "Defun arrow",
                format!("Expected Arrow keyword, got {next:#?}"),
            ));
        }

        let return_type = error!(Type::try_from(&mut *value), "Defun")?;
        let body = error!(Exp::try_from(&mut *value), "Defun")?;

        Ok(Defun {
            scope,
            name,
            args,
            return_type,
            body,
        })
    }
}

impl ToString for Defun {
    fn to_string(&self) -> String {
        format!(
            "{}fn {}{} -> {} {{{}}}",
            self.scope.to_string(),
            self.name,
            self.args.to_string(),
            self.return_type.to_string(),
            self.body.to_string()
        )
    }
}

#[derive(Debug)]
enum Scope {
    File,
    Crate,
    Full,
}

impl TryFrom<&mut Parser> for Scope {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .first()
            .ok_or(error!("Scope", format!("Expected more tokens")))?;
        match next {
            Token::Identifier(iden) => match &iden[..] {
                "crate" => {
                    value.pop_front();
                    Ok(Scope::Crate)
                }
                "pub" => {
                    value.pop_front();
                    Ok(Scope::Full)
                }
                iden => Err(error!("Scope iden", format!("Expected pub, got {iden:#?}"),)),
            },
            _ => Ok(Scope::File),
        }
    }
}

impl ToString for Scope {
    fn to_string(&self) -> String {
        match self {
            Scope::File => "",
            Scope::Crate => "pub(crate) ",
            Scope::Full => "pub ",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::snapshot;

    snapshot!(test_defun, Defun::try_from, "defun.lt");
    snapshot!(test_defun_rust, Defun::try_from, "defun.lt", rust);
}
