use crate::format::text::{parse::error::ParseError, syntax::{self, Expr, Index, Unresolved}};
use crate::{instructions::instruction_by_name, instructions::Operands};
use crate::format::text::token::Token;
use super::Parser;
use std::io::Read;
use crate::format::text::syntax::Instruction;
use super::Result;



impl<R: Read> Parser<R> {
    pub fn parse_instructions(&mut self) -> Result<Vec<Instruction<Unresolved>>> {
        self.zero_or_more_groups(Self::try_instruction)
    }

    /// Called at a point where we expect an instruction name keyword
    fn try_plain_instruction(&mut self) -> Result<Option<Instruction<Unresolved>>> {
        let name = match self.peek_keyword()? {
            None => return Ok(None),
            Some(kw) => kw
        };

        // These mark the end of a block, so we should return none as a signal to the
        // caller.
        if matches!(name, "else" | "end") {
            return Ok(None)
        }

        let instruction_data = instruction_by_name(&name);

        match instruction_data {
            Some(data) => {
                let name = self.try_keyword()?.unwrap();
                let operands = match data.operands {
                    Operands::None | Operands::MemoryGrow => syntax::Operands::None,
                    Operands::FuncIndex => syntax::Operands::FuncIndex(self.expect_index()?),
                    Operands::TableIndex => syntax::Operands::TableIndex(self.expect_index()?),
                    Operands::GlobalIndex => syntax::Operands::GlobalIndex(self.expect_index()?),
                    Operands::ElemIndex => syntax::Operands::ElemIndex(self.expect_index()?),
                    Operands::DataIndex => syntax::Operands::DataIndex(self.expect_index()?),
                    Operands::LocalIndex => syntax::Operands::LocalIndex(self.expect_index()?),
                    Operands::Br => syntax::Operands::LabelIndex(self.expect_index()?),
                    Operands::BrTable => {
                        let idxs = self.zero_or_more(Self::try_index)?;
                        syntax::Operands::BrTable(idxs)
                    }
                    Operands::CallIndirect => {
                        let idx = self.try_index()?.unwrap_or_else(|| Index::unnamed(0));
                        let tu = self.parse_type_use()?;
                        syntax::Operands::CallIndirect(idx, tu)
                    }
                    Operands::I32 => syntax::Operands::I32(self.expect_number()? as u32),
                    Operands::I64 => syntax::Operands::I64(self.expect_number()? as u64),
                    Operands::F32 => syntax::Operands::F32(self.expect_number()? as f32),
                    Operands::F64 => syntax::Operands::F64(self.expect_number()? as f64),
                    Operands::Memargs => { 
                        let align = self.try_align()?.unwrap_or(0);
                        let offset = self.try_offset()?.unwrap_or(0); 
                        syntax::Operands::Memargs(align, offset)
                    },
                    Operands::Block => self.parse_plain_block()?,
                    Operands::If => self.parse_plain_if_operands()?,
                    _ => panic!("Unimplemented operands type {:?}", data.operands)
                };
                Ok(Some(Instruction{name, opcode: data.opcode, operands}))
            },
            None => Ok(None)
        }
    }

    fn parse_plain_block(&mut self) -> Result<syntax::Operands<Unresolved>> {
        println!("PARSING PLAIN BLOCK");
        let label = self.try_id()?;
        let blocktype = self.try_function_type()?;
        let instr = self.parse_instructions()?;
        if self.take_keyword_if(|kw| kw == "end")?.is_none() {
            return Err(ParseError::unexpected("block end"))
        }
        Ok(syntax::Operands::Block(label, blocktype, Expr{instr}))
    }

    fn parse_plain_if_operands(&mut self) -> Result<syntax::Operands<Unresolved>> {
        let label = self.try_id()?;

        let blocktype = self.try_function_type()?;
                        
        let thengroup = self.parse_instructions()?;

        let elsegroup = match self.take_keyword_if(|kw| kw == "else")? {
            Some(_) => {
                self.parse_instructions()?
            }
            _ => vec![]
        };

        if self.take_keyword_if(|kw| kw == "end")?.is_none() {
            return Err(ParseError::unexpected("end"))
        }
                    
        Ok(syntax::Operands::If(label, blocktype, Expr{instr: thengroup}, Expr{instr: elsegroup}))
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

    fn try_plain_instruction_as_single(&mut self) -> Result<Option<Vec<Instruction<Unresolved>>>> {
        self.try_plain_instruction().map(|i| i.map(|i| vec![i]))
    }

    fn try_instruction(&mut self) -> Result<Option<Vec<Instruction<Unresolved>>>> {
        self.first_of(&[
            Self::try_folded_instruction,
            Self::try_plain_instruction_as_single,
        ])
    }

    fn parse_folded_block(&mut self, name: &str, opcode: u8) -> Result<Instruction<Unresolved>> {
        let label = self.try_id()?;
        let blocktype = self.try_function_type()?;
        let instr = self.parse_instructions()?;
        self.expect_close()?;
        let operands = syntax::Operands::Block(label, blocktype, Expr{instr});
        Ok(Instruction{name: name.into(), opcode, operands})
    }

    fn parse_folded_if(&mut self) -> Result<Vec<Instruction<Unresolved>>> {
        let label = self.try_id()?;
        let blocktype = self.try_function_type()?;
        let condition = self.zero_or_more_groups(Self::try_folded_instruction)?;
        let mut unfolded = condition;
        let thexpr = if self.try_expr_start("then")? {
            let instr = self.zero_or_more_groups(Self::try_instruction)?;
            self.expect_close()?;
            Expr{instr}
        } else { 
            return Err(ParseError::unexpected("then"))
        };
        let elexpr = if self.try_expr_start("else")? {
            let instr = self.zero_or_more_groups(Self::try_instruction)?;
            self.expect_close()?;
            Expr{instr}
        } else { Expr::default() };

        self.expect_close()?;
        let operands = syntax::Operands::If(label, blocktype, thexpr, elexpr);

        unfolded.push(Instruction{name: "if".into(), opcode: 0x02, operands});

        Ok(unfolded)
    }

    // (block <label> <bt <instr>*)
    // block <label> <bt> <instr>* end
    // (loop <label> <bt> <instr>*) 
    // loop <label> <bt> <instr>* end
    // <folded>* if <label> <bt> <instr>* <else <instr*>>? end
    // (if <label> <bt> <folded>* (then <instr>*) (else <instr>*)?)
    fn try_folded_instruction(&mut self) -> Result<Option<Vec<Instruction<Unresolved>>>> {
        if self.current.token != Token::Open {
            return Ok(None)
        }

        if matches!(self.peek_next_keyword()?, Some("then") | Some("else")) {
            return Ok(None)
        }

        if self.try_expr_start("block")? {
            return Ok(Some(vec![self.parse_folded_block("block", 0x02)?]))
        }
        
        if self.try_expr_start("loop")? {
            return Ok(Some(vec![self.parse_folded_block("loop", 0x03)?]))
        }

        if self.try_expr_start("if")? {
            return Ok(Some(self.parse_folded_if()?))
        }
        
        self.advance()?;

         // First one must be plain
        let first = match self.try_plain_instruction()? {
            Some(instr) => instr,
            None => return Ok(None)
        };
        
        let mut rest = self.zero_or_more_groups(Self::try_folded_instruction)?;


        rest.push(first);
        self.expect_close()?;

        Ok(Some(rest))
    }
}
