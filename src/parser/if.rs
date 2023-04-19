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
        let _ = error!("If", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!("If", value.pop_front(), [Token::Keyword(Keywords::If)])?;
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

        let _ = error!("If", value.pop_front(), [Token::Keyword(Keywords::Else)])?;
        let false_branch = error!(Exp::try_from(&mut *value), "If")?;
        let _ = error!("If", value.pop_front(), [Token::ParenClose])?;

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
