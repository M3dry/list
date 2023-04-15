use crate::tokenizer::{Keywords, Token};

use super::{error, exp::Exp, Parser, ParserError, ParserErrorStack};
#[derive(Debug)]
pub struct If {
    condition: Exp,
    true_branch: Exp,
    elif_branch: Vec<(Exp, Exp)>,
    false_branch: Exp,
}

impl TryFrom<&mut Parser> for If {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("If")?;
        if next != Token::ParenOpen {
            return Err(error!("If", format!("Expected ParenOpen, got {next:#?}"),));
        }

        let next = value.pop_front_err("If")?;
        if next != Token::Keyword(Keywords::If) {
            return Err(error!("If", format!("Expected If keyword, got {next:#?}"),));
        }

        let condition = error!(Exp::try_from(&mut *value), "If")?;
        let true_branch = error!(Exp::try_from(&mut *value), "If")?;
        let mut elif_branch = vec![];

        loop {
            let peek = value.first();

            if peek == Some(&Token::Keyword(Keywords::Elif)) {
                value.pop_front();
                elif_branch.push((error!(Exp::try_from(&mut *value), "If")?, error!(Exp::try_from(&mut *value), "If")?))
            }

            break
        }

        let next = value.pop_front_err("If")?;
        if next != Token::Keyword(Keywords::Else) {
            return Err(error!("If", format!("Expected else, got {next:#?}")))
        }

        let false_branch = error!(Exp::try_from(&mut *value), "If")?;

        Ok(Self {
            condition,
            true_branch,
            elif_branch,
            false_branch,
        })
    }
}

impl ToString for If {
    fn to_string(&self) -> String {
        format!(
            "if {} {{{}}}{} else {{{}}}",
            self.condition.to_string(),
            self.true_branch.to_string(),
            self.elif_branch.iter().fold(String::new(), |str, elif| format!("{str} else if {} {{{}}}", elif.0.to_string(), elif.1.to_string())),
            self.false_branch.to_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::snapshot;

    snapshot!(test_if, If::try_from, "if.lt");
    snapshot!(test_if_rust, If::try_from, "if.lt", rust);
}
