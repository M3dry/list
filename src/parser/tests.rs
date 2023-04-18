#[cfg(test)]
mod tests {
    use crate::parser::{
        defun::Defun, exp::Exp, lambda::Lambda, r#enum::Enum, r#if::If, r#let::Let, r#match::Match,
        r#struct::Struct, r#use::Use,
    };

    macro_rules! snapshot {
        ($name:tt, $func:expr, $path:tt) => {
            snapshot!(
                $name,
                |parser| format!("{:#?}", $func(parser)),
                $path,
                "../../testdata/parser/"
            );
        };
        ($name:tt, $func:expr, $path:tt, rust) => {
            snapshot!(
                $name,
                |parser| match $func(parser) {
                    Ok(res) => res.to_string(),
                    Err(err) => format!("{err:#?}"),
                },
                $path,
                "../../testdata/rust/"
            );
        };
        ($name:tt, $func:expr, $path:tt, $out:literal) => {
            #[test]
            fn $name() {
                use crate::parser::Parser;

                let contents = include_str!(concat!("../../testdata/input/", $path));
                let mut settings = insta::Settings::clone_current();
                settings.set_snapshot_path($out);
                settings.bind(|| {
                    insta::assert_snapshot!(contents
                        .lines()
                        .filter_map(|line| if line != "" {
                            Some(format!(
                                "{line}\n{}",
                                $func(&mut Parser::new(line.parse().unwrap()))
                            ))
                        } else {
                            None
                        })
                        .collect::<Vec<String>>()
                        .join("\n\n"));
                });
            }
        };
    }

    snapshot!(test_calling, Exp::try_from, "calling.lt");
    snapshot!(test_calling_rust, Exp::try_from, "calling.lt", rust);
    snapshot!(test_defun, Defun::try_from, "defun.lt");
    snapshot!(test_defun_rust, Defun::try_from, "defun.lt", rust);
    snapshot!(test_if, If::try_from, "if.lt");
    snapshot!(test_if_rust, If::try_from, "if.lt", rust);
    snapshot!(test_match, Match::try_from, "match.lt");
    snapshot!(test_match_rust, Match::try_from, "match.lt", rust);
    snapshot!(test_struct, Struct::try_from, "struct.lt");
    snapshot!(test_struct_rust, Struct::try_from, "struct.lt", rust);
    snapshot!(test_enum, Enum::try_from, "enum.lt");
    snapshot!(test_enum_rust, Enum::try_from, "enum.lt", rust);
    snapshot!(test_let, Let::try_from, "let.lt");
    snapshot!(test_let_rust, Let::try_from, "let.lt", rust);
    snapshot!(test_lambda, Lambda::try_from, "lambda.lt");
    snapshot!(test_lambda_rust, Lambda::try_from, "lambda.lt", rust);
    snapshot!(test_use, Use::try_from, "use.lt");
    snapshot!(test_use_rust, Use::try_from, "use.lt", rust);
}
