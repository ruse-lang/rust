use parse::{LexResult, LexError, Token};
use std::cell::Cell;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer {
    location: Cell<usize>,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer { location: Cell::new(0) }
    }

    pub fn lex<'a>(&'a self, s: &'a str) -> LexResult {
        let token_iter = TokenIterator::new(s);
        token_iter.inspect(|t: &Result<Token, LexError>| {
                      if let Ok(ref token) = *t {
                          let old_location = self.location.get();
                          self.location.set(old_location + token.span);
                      };
                  })
                  .collect::<LexResult>()
    }
}

pub struct TokenIterator<'a> {
    char_iter: Peekable<Chars<'a>>,
    location: Cell<usize>,
}

pub type TokenResult<'a> = Result<Token<'a>, LexError>;

impl<'a> TokenIterator<'a> {
    pub fn new<'b>(s: &'b str) -> TokenIterator<'b> {
        TokenIterator {
            char_iter: s.chars().peekable(),
            location: Cell::new(0),
        }
    }

    fn parse_open_paren(&self) -> Option<TokenResult<'a>> {
        Some(Ok(Token::open_paren(self.location.get())))
    }

    fn parse_close_paren(&self) -> Option<TokenResult<'a>> {
        Some(Ok(Token::close_paren(self.location.get())))
    }

    fn parse_number(&mut self, character: char) -> Option<TokenResult<'a>> {
        let mut result = Vec::new();
        let mut is_float = false;
        let mut number_length = 0;
        result.push(character);

        while let Some(&next_character) = self.char_iter.peek() {
            number_length += 1;
            match next_character {
                '0'...'9' => {
                    result.push(next_character);
                    self.char_iter.next();
                }
                '.' => {
                    result.push(next_character);
                    self.char_iter.next();
                    is_float = true;
                }
                _ => break,
            }

            let out: String = result.iter().cloned().collect();

            if is_float {
                if let Ok(val) = out.parse::<f64>() {
                    let lit = Token::float_literal(val, number_length, self.location.get());
                    return Some(Ok(lit));
                }
            } else {
                if let Ok(val) = out.parse::<i64>() {
                    let lit = Token::integer_literal(val, number_length, self.location.get());
                    return Some(Ok(lit));
                }
            }

            return Some(Err(LexError::MalformedNumber(out)));
        }

        None
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = TokenResult<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(character) = self.char_iter.next() {
            // Update location.
            let old_location = self.location.get();
            self.location.set(old_location + 1);

            // Figure out what the next type of token is.
            match character {
                '(' => return self.parse_open_paren(),
                ')' => return self.parse_close_paren(),
                '0'...'9' => return self.parse_number(character),
                // Also want to parse identifiers
                _ => {}
            }
        }

        None
    }
}
