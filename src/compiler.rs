use std::{cell::RefCell, mem, rc::Rc};

use crate::{
    bytecode::{self, FunctionObj, OpCode, Precedence, Value},
    scanner::Scanner,
    token::{self, Token, TokenKind},
    Error, Result,
};

/*
statement      → exprStmt
               | forStmt
               | ifStmt
               | printStmt
               | returnStmt
               | whileStmt
               | block ;

declaration    → classDecl
               | funDecl
               | varDecl
               | statement ;

block          → "{" declaration* "}" ;

*/

struct Local<'a> {
    name: &'a str,
    depth: Option<u32>,
}

impl<'a> Local<'a> {
    fn new(name: &'a str, depth: Option<u32>) -> Self {
        Self { name, depth }
    }
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Token<'a>,
    previous: Token<'a>,
}

impl<'a> Parser<'a> {
    fn new(scanner: Scanner<'a>, current: Token<'a>, previous: Token<'a>) -> Self {
        Self {
            scanner,
            current,
            previous,
        }
    }

    pub fn with_source(source: &'a str) -> Self {
        let scanner = Scanner::new(source);
        Self::new(scanner, Token::none(), Token::none())
    }

    fn current(&self) -> &Token<'a> {
        &self.current
    }

    fn previous(&self) -> &Token<'a> {
        &self.previous
    }

    fn update_tokens(&mut self, new: Token<'a>) {
        self.previous = mem::replace(&mut self.current, new);
    }
}
pub struct Compiler<'a> {
    parser: &'a RefCell<Parser<'a>>,
    error_count: usize,

    fun: FunctionObj,

    functions: Option<Vec<FunctionObj>>,

    locals: Vec<Local<'a>>,
    scope_depth: u32,
}

impl<'a> Compiler<'a> {
    // pub fn with_source(source: &'a str) -> Self {
    //     Self::new(Rc::new(Scanner::new(source)), FunctionObj::new_main())
    // }

    fn new(
        parser: &'a RefCell<Parser<'a>>,
        fun: FunctionObj,
        functions: Option<Vec<FunctionObj>>,
    ) -> Self {
        let mut locals = Vec::with_capacity(256);
        locals.push(Local::new("", Some(0)));
        Self {
            parser,
            error_count: 0,
            fun,
            functions,
            locals,
            scope_depth: 0,
        }
    }

    pub fn with_fun(parser: &'a RefCell<Parser<'a>>, fun: FunctionObj) -> Self {
        Self::new(parser, fun, None)
    }

    pub fn main_compiler(parser: &'a RefCell<Parser<'a>>) -> Self {
        Self::new(parser, FunctionObj::new_main(), Some(Vec::new()))
    }

    fn scan_token(&mut self) -> Result<Token<'a>> {
        self.parser.borrow_mut().scanner.scan_token()
    }

    fn current_kind(&self) -> TokenKind<'a> {
        self.parser.borrow().current().kind()
    }

    fn previous_kind(&self) -> TokenKind<'a> {
        self.parser.borrow().previous().kind()
    }

    fn curr_chunk(&mut self) -> &mut bytecode::Chunk {
        self.fun.chunk_mut()
    }

    fn emit_ins(&mut self, ins: bytecode::OpCode) {
        let line = self.parser.borrow().previous().line();
        self.curr_chunk().write_ins(ins, line);
    }

    fn add_const(&mut self, val: Value) -> u16 {
        self.curr_chunk().add_const(val)
    }

    fn emit_const_ins(&mut self, value: Value) {
        let line = self.parser.borrow().previous().line();
        self.curr_chunk().add_const_ins(value, line);
    }

    pub fn compile(mut self) -> Result<(FunctionObj, Vec<FunctionObj>)> {
        self.advance()?;
        while !self.is_at_end() {
            self.declaration();
        }
        if self.error_count != 0 {
            return Err(Error::from(format!(
                "\nAborting compilation due to {} errors",
                self.error_count
            )));
        }

        #[cfg(feature = "print_code")]
        self.fun.disassemble();

        Ok((self.fun, self.functions.unwrap()))
    }

    fn synchronize(&mut self) {
        let curr = loop {
            if let Ok(token) = self.scan_token() {
                match token.kind() {
                    TokenKind::Semicolon => match self.scan_token() {
                        Ok(next) => {
                            self.parser.borrow_mut().previous = token;
                            break next;
                        }

                        Err(error) => {
                            self.report_error(error);
                            continue;
                        }
                    },
                    TokenKind::Eof
                    | TokenKind::Class
                    | TokenKind::Fun
                    | TokenKind::Var
                    | TokenKind::For
                    | TokenKind::If
                    | TokenKind::While
                    | TokenKind::Print
                    | TokenKind::Return => break token,
                    _ => {}
                }
            };
        };
        self.parser.borrow_mut().update_tokens(curr);
    }

    fn write_ident_constant(&mut self, ident: &'a str) -> u16 {
        let ident = Value::String(Rc::new(ident.to_string()));
        self.add_const(ident)
    }

    fn declaration(&mut self) -> bool {
        // true means it was successful
        let result = match self.current_kind() {
            TokenKind::Var => self.var_decl(),
            TokenKind::Fun => self.fun_decl(),
            _ => self.statement(),
        };
        if let Err(error) = result {
            self.synchronize();
            self.report_error(error);
            return false;
        }
        true
    }

    fn fun_decl(&mut self) -> Result<()> {
        self.advance()?;

        let (id, name) = self.declare_variable()?;
        let name = name.to_string();

        self.mark_initialized();

        let fun_compiler = Compiler::with_fun(self.parser, FunctionObj::new(name, 0));

        let fun = fun_compiler.compile_fun()?;

        self.emit_const_ins(Value::Function(self.functions.as_ref().unwrap().len()));
        self.functions.as_mut().unwrap().push(fun);

        self.define_variable(id);
        Ok(())
    }

    // fn func_scope(&mut self, name: String) -> Result<()> {
    //     let old_scope = std::mem::replace(&mut self.scope_depth, 1);
    //     let old_start = std::mem::replace(
    //         &mut self.fun_local_start,
    //         self.locals.len().saturating_sub(1),
    //     );
    //     let new_func = FunctionObj::new(name, 0);

    //     let mut func = std::mem::replace(&mut self.curr_func, Some(new_func));

    //     let result = self.parse_fun();

    //     self.end_scope();

    //     self.scope_depth = old_scope;
    //     self.fun_local_start = old_start;
    //     std::mem::swap(&mut self.curr_func, &mut func);

    //     let func = Rc::new(func.unwrap());
    //     self.emit_const_ins(Value::Function(func));

    //     result
    // }

    fn parse_fun(&mut self) -> Result<()> {
        self.scope_depth += 1;
        self.consume(TokenKind::LeftParen, "Expect '(' after function name.")?;
        if !self.check_curr(TokenKind::RightParen) {
            loop {
                if self.fun.arity() == u8::MAX {
                    self.error_at_current("Cannot have more than 255 parameters.");
                }
                *self.fun.arity_mut() += 1;
                let (id, _) = self.declare_variable()?;
                self.define_variable(id);
                if !self.match_curr(TokenKind::Comma)? {
                    break;
                }
            }
        }
        self.consume(TokenKind::RightParen, "Expect ')' after parameters")?;
        if !self.check_curr(TokenKind::LeftBrace) {
            return Err(self.error_at_current("Expect '{' before function body."));
        }
        // self.consume(TokenKind::LeftBrace, "Expect '{' before function body.")?;

        self.block()?;

        Ok(())
    }

    fn compile_fun(mut self) -> Result<FunctionObj> {
        let result = self.parse_fun();

        #[cfg(feature = "print_code")]
        self.fun.disassemble();

        result?;

        Ok(self.fun)
    }

    fn statement(&mut self) -> Result<()> {
        match self.current_kind() {
            TokenKind::Print => self.print_stmt(),
            TokenKind::LeftBrace => self.block(),
            TokenKind::If => self.if_stmt(),
            TokenKind::While => self.while_stmt(),
            TokenKind::For => self.for_stmt(),
            TokenKind::Return => self.return_stmt(),
            _ => self.expression_stmt(),
        }
    }

    fn return_stmt(&mut self) -> Result<()> {
        self.advance()?;
        if self.check_curr(TokenKind::Semicolon) {
            self.emit_ins(OpCode::Nil);
            self.consume(TokenKind::Semicolon, "Expect ';' after return.")?;
        } else {
            if self.fun.is_main() {
                return Err(self.error_at_current("Cannot return value from top-level code."));
            }
            self.expression()?;
            self.consume(TokenKind::Semicolon, "Expect ';' after return value.")?;
        }
        self.emit_ins(OpCode::Return);
        Ok(())
    }

    fn for_stmt(&mut self) -> Result<()> {
        self.advance()?;
        self.scope_depth += 1;

        let result = self.parse_for();

        self.end_scope();
        result
    }

    fn parse_for(&mut self) -> Result<()> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'.")?;
        match self.current_kind() {
            TokenKind::Semicolon => {}
            TokenKind::Var => {
                self.var_decl()?;
            }
            _ => {
                self.expression_stmt()?;
            }
        }

        let mut loop_start = self.curr_chunk().len();
        let mut exit_jump = None;

        if !self.match_curr(TokenKind::Semicolon)? {
            self.expression()?;
            self.consume(TokenKind::Semicolon, "Expect ';' after loop condition.")?;

            // Jump out of the loop if the condition is false.
            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse(None)));
            self.emit_ins(OpCode::Pop);
        }

        if !self.match_curr(TokenKind::RightParen)? {
            let body_jump = self.emit_jump(OpCode::Jump(None));

            let increment_start = self.curr_chunk().len();
            self.expression()?;
            self.emit_ins(OpCode::Pop);

            self.consume(TokenKind::RightParen, "Expect ')' after for clauses.")?;

            self.emit_loop(loop_start)?;
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement()?;

        self.emit_loop(loop_start)?;

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.emit_ins(OpCode::Pop);
        }
        Ok(())
    }

    fn while_stmt(&mut self) -> Result<()> {
        self.advance()?;

        let loop_start = self.curr_chunk().len();

        self.consume(TokenKind::LeftParen, "Expect '(' after 'while'.")?;
        self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after condition.")?;

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse(None));
        self.emit_ins(OpCode::Pop);

        self.statement()?;
        self.emit_loop(loop_start)?;

        self.patch_jump(exit_jump);
        self.emit_ins(OpCode::Pop);
        Ok(())
    }

    fn if_stmt(&mut self) -> Result<()> {
        self.advance()?;

        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.")?;
        self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after condition.")?;

        let then_jump = self.emit_jump(OpCode::JumpIfFalse(None));
        self.emit_ins(OpCode::Pop);
        self.statement()?;

        let else_jump = self.emit_jump(OpCode::Jump(None));

        self.patch_jump(then_jump);
        self.emit_ins(OpCode::Pop);

        if self.match_curr(TokenKind::Else)? {
            self.statement()?;
        }
        self.patch_jump(else_jump);
        Ok(())
    }

    fn emit_loop(&mut self, loop_start: usize) -> Result<()> {
        let offset = self.curr_chunk().len() - loop_start + 1;
        if offset > u16::MAX as usize {
            return Err(self.error_at_previous("Loop body too large."));
        }
        self.emit_ins(OpCode::Loop(offset as u16));
        Ok(())
    }

    fn emit_jump(&mut self, ins: OpCode) -> usize {
        self.emit_ins(ins);
        self.curr_chunk().len() - 1
    }

    fn patch_jump(&mut self, index: usize) {
        let jump_offset = (self.curr_chunk().len() - index - 1) as u16;

        let code = self.curr_chunk().code_mut();
        match code[index] {
            OpCode::JumpIfFalse(None) => code[index] = OpCode::JumpIfFalse(Some(jump_offset)),
            OpCode::Jump(None) => code[index] = OpCode::Jump(Some(jump_offset)),
            _ => unreachable!("Internal error: Tried to patch non jump insruction"),
        }
    }

    fn block(&mut self) -> Result<()> {
        self.advance()?;
        self.scope_depth += 1;

        while !self.check_curr(TokenKind::RightBrace) && !self.is_at_end() {
            if !self.declaration() {
                self.end_scope();
                return Ok(());
            }
        }

        self.end_scope();
        self.consume(TokenKind::RightBrace, "Expect '}' after block.")?;
        Ok(())
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        // discard all locals that are in this scope
        while !self.locals.is_empty()
            && self.locals.last().unwrap().depth.unwrap() > self.scope_depth
        {
            self.emit_ins(OpCode::Pop);
            self.locals.pop();
        }
    }

    fn var_decl(&mut self) -> Result<()> {
        self.advance()?;
        let (id, _) = self.declare_variable()?;

        if self.match_curr(TokenKind::Equal)? {
            self.expression()?;
        } else {
            self.emit_ins(OpCode::Nil);
        }

        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        self.define_variable(id);
        Ok(())
    }

    fn declare_local(&mut self, name: &'a str) -> Result<()> {
        for i in (0..self.locals.len()).rev() {
            let local = &self.locals[i];
            if local.depth.is_some() && local.depth.unwrap() < self.scope_depth {
                break;
            }
            if local.name == name {
                return Err(self
                    .error_at_previous("Variable with this name already declared in this scope."));
            }
        }

        self.locals.push(Local::new(name, None));
        Ok(())
    }

    fn declare_variable(&mut self) -> Result<(u16, &str)> {
        let msg = match self.previous_kind() {
            TokenKind::Fun => "Expect function name.",
            _ => "Expect variable name.",
        };
        let name = self.consume_ident(msg)?;
        if self.scope_depth == 0 {
            // a global
            Ok((self.write_ident_constant(name), name))
        } else {
            self.declare_local(name)?;
            Ok((0, name))
        }
    }

    fn define_variable(&mut self, id: u16) {
        if self.scope_depth == 0 {
            self.emit_ins(OpCode::DefineGlobal(id));
        } else {
            self.mark_initialized();
        }
    }

    fn mark_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        self.locals.last_mut().unwrap().depth = Some(self.scope_depth);
    }

    fn variable(&mut self, ident: &'a str, can_assign: bool) -> Result<()> {
        let (is_local, arg) = if let Some(offset) = self.resolve_local(ident) {
            (true, offset)
        } else {
            (false, self.write_ident_constant(ident))
        };

        if can_assign && self.match_curr(TokenKind::Equal)? {
            self.expression()?;
            if is_local {
                self.emit_ins(OpCode::SetLocal(arg));
            } else {
                self.emit_ins(OpCode::SetGlobal(arg));
            }
        } else if is_local {
            self.emit_ins(OpCode::GetLocal(arg));
        } else {
            self.emit_ins(OpCode::GetGlobal(arg));
        }

        Ok(())
    }

    fn resolve_local(&mut self, name: &'a str) -> Option<u16> {
        for (i, local) in self.locals.iter().rev().enumerate() {
            if local.name == name {
                if local.depth.is_none() {
                    self.error_at_previous("Cannot read local variable in its own initializer.");
                }
                return Some((self.locals.len() - 1 - i) as u16);
            }
        }

        None
    }

    fn print_stmt(&mut self) -> Result<()> {
        self.advance()?;
        self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after value.")?;
        self.emit_ins(OpCode::Print);
        Ok(())
    }

    fn expression_stmt(&mut self) -> Result<()> {
        self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.")?;
        self.emit_ins(OpCode::Pop);
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.check_curr(TokenKind::Eof)
    }

    fn match_curr(&mut self, kind: TokenKind) -> Result<bool> {
        if self.check_curr(kind) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn check_curr(&self, kind: TokenKind) -> bool {
        self.current_kind() == kind
    }

    fn advance(&mut self) -> Result<()> {
        let token = self.scan_token();
        match token {
            Ok(token) => {
                self.parser.borrow_mut().update_tokens(token);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn report_error(&mut self, error: Error) {
        self.error_count += 1;
        eprintln!("Parsing error: {}", error);
    }

    fn error_at(&self, token: &Token<'a>, msg: &str) -> Error {
        Error::from(format!(
            "{} at line {}, at token '{}'",
            msg,
            token.line(),
            token.kind()
        ))
    }

    fn error_at_previous(&mut self, msg: &str) -> Error {
        self.error_at(self.parser.borrow().previous(), msg)
    }

    fn error_at_current(&mut self, msg: &str) -> Error {
        self.error_at(self.parser.borrow().current(), msg)
    }

    fn consume(&mut self, expected: TokenKind, msg: &str) -> Result<()> {
        if self.check_curr(expected) {
            self.advance()?;
            Ok(())
        } else {
            Err(self.error_at_current(msg))
        }
    }

    fn consume_ident(&mut self, msg: &str) -> Result<&'a str> {
        if let TokenKind::Identifier(ident) = self.current_kind() {
            self.advance()?;
            Ok(ident)
        } else {
            Err(self.error_at_current(msg))
        }
    }

    fn expression(&mut self) -> Result<()> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn grouping(&mut self) -> Result<()> {
        self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after expression")
    }

    fn unary(&mut self) -> Result<()> {
        let op = self.previous_kind();
        self.parse_precedence(Precedence::Unary)?;
        self.emit_ins(match op {
            TokenKind::Bang => OpCode::Not,
            TokenKind::Minus => OpCode::Negate,
            _ => unreachable!(),
        });
        Ok(())
    }

    // parse any expression at given precendece level or higher
    fn parse_precedence(&mut self, precedence: Precedence) -> Result<()> {
        self.advance()?;
        let can_assign = precedence <= Precedence::Assignment;
        self.prefix(self.previous_kind(), can_assign)?;

        while precedence <= self.current_kind().precedence() {
            self.advance()?;
            self.infix(self.previous_kind())?;
        }
        if can_assign && self.match_curr(TokenKind::Equal)? {
            return Err(self.error_at_previous("Invalid assignment target."));
        }
        Ok(())
    }

    fn binary(&mut self) -> Result<()> {
        let operator = self.previous_kind();
        let precedence = operator.precedence();
        self.parse_precedence(precedence.higher())?;

        match operator {
            TokenKind::Plus => self.emit_ins(OpCode::Add),
            TokenKind::Minus => self.emit_ins(OpCode::Subtract),
            TokenKind::Star => self.emit_ins(OpCode::Multiply),
            TokenKind::Slash => self.emit_ins(OpCode::Divide),
            TokenKind::BangEqual => {
                self.emit_ins(OpCode::Equal);
                self.emit_ins(OpCode::Not);
            }
            TokenKind::EqualEqual => self.emit_ins(OpCode::Equal),
            TokenKind::Less => self.emit_ins(OpCode::Less),
            TokenKind::LessEqual => {
                self.emit_ins(OpCode::Greater);
                self.emit_ins(OpCode::Not);
            }

            TokenKind::Greater => self.emit_ins(OpCode::Greater),
            TokenKind::GreaterEqual => {
                self.emit_ins(OpCode::Less);
                self.emit_ins(OpCode::Not);
            }

            _ => unreachable!(),
        }
        Ok(())
    }

    fn or(&mut self) -> Result<()> {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse(None));
        let end_jump = self.emit_jump(OpCode::Jump(None));

        self.patch_jump(else_jump);
        self.emit_ins(OpCode::Pop);

        self.parse_precedence(Precedence::Or)?;
        self.patch_jump(end_jump);

        Ok(())
    }

    fn and(&mut self) -> Result<()> {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse(None));

        self.emit_ins(OpCode::Pop);
        self.parse_precedence(Precedence::And)?;

        self.patch_jump(end_jump);
        Ok(())
    }

    fn literal(&mut self) {
        match self.previous_kind() {
            TokenKind::True => self.emit_ins(OpCode::True),
            TokenKind::False => self.emit_ins(OpCode::False),
            TokenKind::Nil => self.emit_ins(OpCode::Nil),
            _ => unreachable!(),
        }
    }

    fn argument_list(&mut self) -> Result<u8> {
        let mut arg_count = 0;
        if !self.check_curr(TokenKind::RightParen) {
            loop {
                self.expression()?;
                if arg_count == 255 {
                    return Err(self.error_at_previous("Cannot have more than 255 arguments."));
                }
                arg_count += 1;
                if !self.match_curr(TokenKind::Comma)? {
                    break;
                }
            }
        }
        self.consume(TokenKind::RightParen, "Expect ')' after arguments.")?;
        Ok(arg_count)
    }

    fn call(&mut self) -> Result<()> {
        let arg_count = self.argument_list()?;
        self.emit_ins(OpCode::Call(arg_count));
        Ok(())
    }

    fn _trace(&mut self, token: &token::Token, prev_line: usize) {
        if token.line() != prev_line {
            println!("{:04}", token.line());
        } else {
            print!("   | ");
        }
        print!("{:?} ", token.kind());
    }

    fn prefix(&mut self, kind: TokenKind<'a>, can_assign: bool) -> Result<()> {
        match kind {
            TokenKind::LeftParen => self.grouping(),
            TokenKind::Number(val) => {
                self.emit_const_ins(Value::Number(val));
                Ok(())
            }
            TokenKind::Minus | TokenKind::Bang => self.unary(),
            TokenKind::True | TokenKind::False | TokenKind::Nil => {
                self.literal();
                Ok(())
            }
            TokenKind::String(s) => {
                self.emit_const_ins(Value::String(Rc::new(s.to_string())));
                Ok(())
            }
            TokenKind::Identifier(ident) => self.variable(ident, can_assign),

            _ => Err(self.error_at_previous(&format!("Unexpected token '{:?}'", kind))),
        }
    }

    fn infix(&mut self, kind: TokenKind) -> Result<()> {
        match kind {
            TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::Slash
            | TokenKind::Star
            | TokenKind::EqualEqual
            | TokenKind::Bang
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Less
            | TokenKind::LessEqual => self.binary(),

            TokenKind::Or => self.or(),
            TokenKind::And => self.and(),

            TokenKind::LeftParen => self.call(),

            _ => Ok(()),
        }
    }
}
