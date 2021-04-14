mod lex;
mod token;

use crate::err;
use crate::error::{Result, ResultFrom};
use crate::module::section::Section;
use lex::Tokenizer;
use std::io::Read;
use token::Token;

pub struct Parser<R: Read> {
    tokenizer: Tokenizer<R>,
    current: Token,
}

impl<R: Read> Parser<R> {
    pub fn parse(r: R) -> Result<Vec<Section>> {
        let mut parser = Parser::new(r)?;
        parser.advance()?;
        parser.parse_module()
    }

    fn new(r: R) -> Result<Parser<R>> {
        Ok(Parser {
            tokenizer: Tokenizer::new(r)?,
            current: Token::Start,
        })
    }

    fn next(&mut self) -> Result<()> {
        if self.current == Token::Eof {
            panic!("EOF")
        }
        match self.tokenizer.next() {
            None => self.current = Token::Eof,
            Some(Ok(t)) => self.current = t,
            Some(Err(e)) => return Err(e),
        }
        Ok(())
    }

    // Advance to the next token, skipping all whitespace and comments.
    fn advance(&mut self) -> Result<()> {
        self.next()?;
        while self.current.ignorable() {
            self.next()?;
        }
        Ok(())
    }

    fn parse_module(&mut self) -> Result<Vec<Section>> {
        if self.current != Token::Open {
            return err!("Invalid start token {:?}", self.current);
        }
        self.advance()?;

        // Modules usually start with "(module". However, this is optional, and a module file can
        // be a list of top-levelo sections.
        if self.current.is_keyword("module") {
            self.advance()?;
        }

        if self.current != Token::Open {
            return err!("Invalid start token {:?}", self.current);
        }
        self.advance()?;

        // section*
        let mut result: Vec<Section> = vec![];
        while let Some(s) = self.parse_section()? {
            result.push(s);

            self.advance()?;
            match self.current {
                Token::Open => (),
                Token::Close => break,
                _ => return err!("Invalid start token {:?}", self.current),
            }
            self.advance()?;
        }

        Ok(result)
    }

    fn consume_expression(&mut self) -> Result<()> {
     let mut depth = 1;
        while depth > 0 {
            self.advance()?;
            match self.current {
                Token::Open => depth += 1,
                Token::Close => depth -= 1,
                _ => (),
            };
            if depth == 0 {
                break;
            }
        }
        Ok(())
    }

    // Parser should be located at the token immediately following a '('
    fn parse_section(&mut self) -> Result<Option<Section>> {
        let name = self.current.expect_keyword()
            .wrap("parsing section name")?
            .clone();

        match name.as_ref() {
            "type" => self.parse_type_section(),
            "func" => self.parse_func_section(),
            "table" => self.parse_table_section(),
            "memory" => self.parse_memory_section(),
            "import" => self.parse_import_section(),
            "export" => self.parse_export_section(),
            "global" => self.parse_global_section(),
            _ => err!("unknown section name {}", name)
        }
    }

    fn parse_type_section(&mut self) -> Result<Option<Section>> {
        self.consume_expression()?; 
        Ok(Some(Section::Types(Box::new([]))))
    }

    fn parse_func_section(&mut self) -> Result<Option<Section>> {
        self.consume_expression()?; 
        Ok(Some(Section::Funcs(Box::new([]))))
    }   

    fn parse_table_section(&mut self) -> Result<Option<Section>> {
        self.consume_expression()?; 
        Ok(Some(Section::Tables(Box::new([]))))
    }

    fn parse_memory_section(&mut self) -> Result<Option<Section>> {
        self.consume_expression()?; 
        Ok(Some(Section::Mems(Box::new([]))))
    }

    fn parse_import_section(&mut self) -> Result<Option<Section>> {
        self.consume_expression()?; 
        Ok(Some(Section::Imports(Box::new([]))))
    }

    fn parse_export_section(&mut self) -> Result<Option<Section>> {
        self.consume_expression()?; 
        Ok(Some(Section::Exports(Box::new([]))))
    }
    
    fn parse_global_section(&mut self) -> Result<Option<Section>> {
        self.consume_expression()?; 
        Ok(Some(Section::Globals(Box::new([]))))
    }
}
