use crate::format::text::syntax;
use crate::{err, instructions::instruction_by_name, instructions::Operands};
use crate::format::text::token::Token;
use super::Parser;
use std::io::Read;
use crate::format::text::syntax::Instruction;
use crate::error::{Result, ResultFrom};

impl<R: Read> Parser<R> {
    pub fn parse_instructions(&mut self) -> Result<Vec<Instruction>> {
        self.zero_or_more_groups(Self::try_instruction)
    }

    /// Called at a point where we expect an instruction name keyword
    fn try_plain_instruction(&mut self) -> Result<Option<Instruction>> {
        let name = match self.try_keyword()? {
            None => return Ok(None),
            Some(kw) => kw
        };

        let instruction_data = instruction_by_name(&name);
        println!("INSTRUCTION DATA {:?}", instruction_data);

        match instruction_data {
            Some(data) => {
                let operands = match data.operands {
                    Operands::None | Operands::MemoryGrow => syntax::Operands::None,
                    Operands::LocalIndex | Operands::MemIndex | Operands::FuncIndex | 
                    Operands::TableIndex | Operands::DataIndex | Operands::ElemIndex |
                    Operands::GlobalIndex => { 
                        syntax::Operands::Index(self.parse_index()?)
                    }
                    Operands::I32 | Operands::I64 | 
                    Operands::F32 | Operands::F64 => { 
                        syntax::Operands::Number(self.expect_number()?)
                    }, 
                    Operands::Memargs => { 
                        let align = self.try_align()?.unwrap_or(0);
                        let offset = self.try_offset()?.unwrap_or(0); 
                        syntax::Operands::Memargs(align, offset)
                    },
                    _ => return err!("Unimplemented operands type {:?}", data.operands)
                };
                Ok(Some(Instruction{name, operands}))
            },
            None => err!("bad instruction name {}", name)
        }
    }

    fn try_plain_instructions(&mut self) -> Result<Option<Vec<Instruction>>> {
        let instr = self.zero_or_more(Self::try_plain_instruction)?;
        if instr.is_empty() {
            Ok(None)
        } else {
            Ok(Some(instr))
        }
    }

    fn try_align(&mut self) -> Result<Option<u32>> {
        if let Some(kw)  = self.take_keyword_if(|kw| kw.starts_with("align="))? {
            if let Some(idx) = kw.find('=') {
                 let (_, valstr) = kw.split_at(idx + 1);
                 if let Ok(val) = u32::from_str_radix(valstr, 10) {
                     return Ok(Some(val))
                 }
            }
        }

        Ok(None)
    }

    fn try_offset(&mut self) -> Result<Option<u32>> {
        if let Some(kw)  = self.take_keyword_if(|kw| kw.starts_with("offset="))? {
            if let Some(idx) = kw.find('=') {
                 let (_, valstr) = kw.split_at(idx + 1);
                 if let Ok(val) = u32::from_str_radix(valstr, 10) {
                     return Ok(Some(val))
                 }
            }
        }

        Ok(None)
    }


    fn try_instruction(&mut self) -> Result<Option<Vec<Instruction>>> {
        self.first_of(&[
            Self::try_folded_instruction,
            Self::try_plain_instructions
        ])
    }

    fn try_folded_instruction(&mut self) -> Result<Option<Vec<Instruction>>> {
        if self.current.token != Token::Open {
            return Ok(None)
        }

        // TODO - handle block expr
        if self.try_expr_start("block")? {
            self.consume_expression()?;
            return Ok(Some(vec![]))
        }

        if self.try_expr_start("loop")? {
            self.consume_expression()?;
            return Ok(Some(vec![]))
        }

        if self.try_expr_start("if")? {
            self.consume_expression()?;
            return Ok(Some(vec![]))
        }

        self.advance()?;
        // TODO: block, if, loop

        // First one must be plain
        let first = match self.try_plain_instruction()? {
            Some(instr) => instr,
            None => return err!("fold must start with plain instructin")
        };

        let mut rest = self.zero_or_more_groups(Self::try_folded_instruction)?;

        self.expect_close().wrap("closing folded instruction")?;

        rest.push(first);

        Ok(Some(rest))
    }

}
