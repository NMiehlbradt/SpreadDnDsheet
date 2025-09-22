use std::iter::Peekable;

use plex::lexer;

use crate::language::ast::Value;
use crate::language::ast::AST;

use super::ast::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Whitespace,
    Error,

    IntLit,
    StringLit,
    Name,

    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,

    Comma,
    Dot,
    Colon,

    Plus,
    Minus,
    Star,
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    token_type: TokenType,
    text: &'a str,
}

lexer! {
    fn next_token(_text: 'a) -> TokenType;

    r#"[ \t\n\r]+"# => TokenType::Whitespace,

    r#"\("# => TokenType::LParen,
    r#"\)"# => TokenType::RParen,
    
    r#"\["# => TokenType::LBrack,
    r#"\]"# => TokenType::RBrack,

    r#"{"# => TokenType::LBrace,
    r#"}"# => TokenType::RBrace,

    r#","# => TokenType::Comma,
    r#"\."# => TokenType::Dot,
    r#":"# => TokenType::Colon,

    r#"\+"# => TokenType::Plus,
    r#"-"# => TokenType::Minus,
    r#"\*"# => TokenType::Star,

    r#"[0-9]+"# => TokenType::IntLit,
    r#""[^"]*""# => TokenType::StringLit, //TODO escape chars

    r#"[a-zA-Z_][a-zA-Z0-9_]*"# => TokenType::Name,

    r#"."# => TokenType::Error,
}

struct Lexer<'a> {
    current: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(text: &'a str) -> Lexer<'a> {
        Lexer { current: text }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        loop {
            let token = next_token(self.current).map(|(t, rest)| {
                let token = Token {
                    token_type: t,
                    text: &self.current[0..self.current.len() - rest.len()],
                };
                self.current = rest;
                token
            });
            if let Some(Token {
                token_type: TokenType::Whitespace,
                ..
            }) = token
            {
                continue;
            } else {
                return token;
            }
        }
    }
}

struct Parser<'a> {
    tokens: Peekable<Lexer<'a>>,
}

impl Error {
    fn parse_error(message: impl Into<String>) -> Self {
        Error::with_message(format!("Parse Error: {}", message.into()))
    }
}

pub fn parse(text: &str) -> Result<AST, Error> {
    let mut parser = Parser::new(text);
    let expr = parser.parse_expr(0);
    match parser.next() {
        None => expr,
        Some(t) => Err(Error::parse_error(format!("Unexpected token: {}", t.text))),
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            tokens: Lexer::new(text).peekable(),
        }
    }

    fn peek(&mut self) -> Option<&Token<'a>> {
        self.tokens.peek()
    }

    fn expect_token(&mut self, token_type: TokenType) -> Result<Token<'a>, Error> {
        match self.next() {
            Some(t) if t.token_type == token_type => Ok(t),
            _ => Err(Error::parse_error("Unexpected token")),
        }
    }

    fn next_if_eq(&mut self, token_type: TokenType) -> Option<Token<'a>> {
        self.tokens.next_if(|t| t.token_type == token_type)
    }
    fn parse_expr(&mut self, min_bp: u8) -> Result<AST, Error> {
        macro_rules! token_type {
            ($token_type:ident) => {
                Some(Token {
                    token_type: TokenType::$token_type,
                    ..
                })
            };

            ($token_type:ident, $text:pat) => {
                Some(Token {
                    token_type: TokenType::$token_type,
                    text: $text,
                })
            };
        }

        macro_rules! comma_seperated {
            ($close:ident, $collect:ident, $parse:expr) => {
                let mut $collect = vec![];
                if self.next_if_eq(TokenType::$close).is_none() {
                    $collect.push($parse);
                    while self.next_if_eq(TokenType::Comma).is_some() {
                        $collect.push($parse);
                    }
                    self.expect_token(TokenType::$close)?;
                }
            }
        }

        let mut lhs = match self.next() {
            // Integer Literals
            token_type!(IntLit, text) => AST::Literal(Value::Integer(
                text.parse()
                    .map_err(|_| Error::parse_error("Invalid int"))?,
            )),
            // String Literals
            token_type!(StringLit, text) => AST::Literal(Value::String(text.to_string())), //TODO escape chars
            // List Literals
            token_type!(LBrack) => {
                comma_seperated!(RBrack, elements, self.parse_expr(0)?);
                AST::Literal(Value::List(elements))
            }
            // Record Literals
            token_type!(LBrace) => {
                comma_seperated!(RBrace, elements, {
                    let name = self.expect_token(TokenType::Name)?.text.to_string();
                    self.expect_token(TokenType::Colon)?;
                    let value = self.parse_expr(0)?;
                    (name, value)
                });
                AST::Literal(Value::Record(elements.into_iter().collect()))
            }

            // Names
            token_type!(Name, text) => match self.peek() {
                // Name followed by brackets is a function call
                // TODO: This will probably eventually be moved as a postfix operator once user defined functions are supported
                token_type!(LParen) => {
                    self.next();
                    comma_seperated!(RParen, args, self.parse_expr(0)?);
                    AST::function(text.to_string(), args)
                }
                _ => AST::Var(text.to_string()),
            },

            // Prefix operators
            token_type!(Minus) => {
                let (_, right_bp) = prefix(5);
                let rhs = self.parse_expr(right_bp)?;
                AST::function("negate", vec![rhs])
            }

            // Brackets
            token_type!(LParen) => {
                let expr = self.parse_expr(0)?;
                self.expect_token(TokenType::RParen)?;
                expr
            }
            _ => return Err(Error::parse_error("Expected name or lit int")),
        };

        macro_rules! infix_op {
            ($prec:expr, $func:literal) => {{
                let (left_bp, right_bp) = $prec;
                if left_bp < min_bp {
                    break;
                }
                self.tokens.next();
                let rhs = self.parse_expr(right_bp)?;
                lhs = AST::function($func, vec![lhs, rhs]);
            }};
        }

        loop {
            match self.peek().copied() {
                // Infix operators
                token_type!(Plus) => infix_op!(assoc_left(1), "+"),
                token_type!(Minus) => infix_op!(assoc_left(1), "-"),
                token_type!(Star) => infix_op!(assoc_left(2), "*"),

                // Postfix operators
                token_type!(Dot) => {
                    let (left_bp, _) = postfix(10);
                    if left_bp < min_bp {
                        break;
                    }
                    self.tokens.next();
                    let field = self.expect_token(TokenType::Name)?.text.to_string();
                    let rhs = AST::Literal(Value::String(field));
                    lhs = AST::function("dot", vec![lhs, rhs]);
                }
                _ => break,
            };
        }

        Ok(lhs)
    }
}

fn assoc_left(bp: u8) -> (u8, u8) {
    (bp * 2 - 1, bp * 2)
}

// fn assoc_right(bp: u8) -> (u8, u8) {
//     (bp * 2, bp * 2 - 1)
// }

fn prefix(bp: u8) -> ((), u8) {
    ((), bp * 2 - 1)
}

fn postfix(bp: u8) -> (u8, ()) {
    (bp * 2 - 1, ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ast::s_exprs::ToSExpr;

    macro_rules! test_parse_success {
        ($test_name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $test_name() {
                assert_eq!(parse($input).unwrap().to_s_expr(), $expected);
            }
        };
    }

    test_parse_success!(test_int, "5", "5");
    test_parse_success!(test_int2, "0", "0");
    test_parse_success!(test_string, "\"string\"", "\"string\"");
    test_parse_success!(test_list_lit, "[1, 2, 3]", "[1, 2, 3]");
    test_parse_success!(test_record_lit, "{b: 2, a: 1}", "{a: 1, b: 2}");
    test_parse_success!(test_plus, "1 + 2", "(+ 1 2)");
    test_parse_success!(test_minus, "1 - 2", "(- 1 2)");
    test_parse_success!(test_multiply, "1 * 2", "(* 1 2)");
    test_parse_success!(test_negate, "-1", "(negate 1)");
    test_parse_success!(test_negate2, "--1", "(negate (negate 1))");
    test_parse_success!(test_prec_left, "1 * 2 + 3", "(+ (* 1 2) 3)");
    test_parse_success!(test_prec_right, "1 + 2 * 3", "(+ 1 (* 2 3))");
    test_parse_success!(test_dot, "a.b", "(dot a b)");
    test_parse_success!(test_dot2, "a.b.c", "(dot (dot a b) c)");
    test_parse_success!(test_dot_prec_left, "a.b + c", "(+ (dot a b) c)");
    test_parse_success!(test_dot_prec_right, "a + b.c", "(+ a (dot b c))");
    
}
