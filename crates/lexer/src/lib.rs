use crate::error::SyntaxError;
use crate::error::SyntaxResult;
use crate::state::source::Source;
use crate::state::StackFrame;
use crate::state::State;
use pxp_bytestring::ByteString;

use pxp_token::DocStringIndentationKind;
use pxp_token::OpenTagKind;
use pxp_token::Token;
use pxp_token::TokenKind;

pub mod error;
pub mod macros;
pub mod state;

#[derive(Debug)]
pub struct Lexer<'a> {
    state: State,
    source: Source<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new<B: ?Sized + AsRef<[u8]>>(input: &'a B) -> Self {
        Self {
            source: Source::new(input.as_ref()),
            state: State::new(),
        }
    }

    /// Tokenize the input in immediate mode, which means that the lexer will immediately
    /// enter scripting state and start parsing PHP tokens.
    pub fn tokenize_in_immediate_mode(&'a mut self) -> SyntaxResult<Vec<Token>> {
        self.state.replace(StackFrame::Scripting);

        self.tokenize()
    }

    pub fn tokenize(&'a mut self) -> SyntaxResult<Vec<Token<'a>>> {
        let mut tokens = Vec::new();

        while !self.source.eof() {
            self.source.start_token();

            match self.state.frame()? {
                // The "Initial" state is used to parse inline HTML. It is essentially a catch-all
                // state that will build up a single token buffer until it encounters an open tag
                // of some description.
                StackFrame::Initial => self.initial(&mut tokens)?,
                // The scripting state is entered when an open tag is encountered in the source code.
                // This tells the lexer to start analysing characters at PHP tokens instead of inline HTML.
                StackFrame::Scripting => {
                    self.skip_whitespace();

                    // If we have consumed whitespace, we should restart the token's position tracking
                    // to ensure we accurately track the span of the token.
                    self.source.start_token();

                    // If we have consumed whitespace and then reached the end of the file, we should break.
                    if self.source.eof() {
                        break;
                    }

                    tokens.push(self.scripting()?);
                }
                // The "Halted" state is entered when the `__halt_compiler` token is encountered.
                // In this state, all the text that follows is no longer parsed as PHP as is collected
                // into a single "InlineHtml" token (kind of cheating, oh well).
                StackFrame::Halted => {
                    let symbol = self.source.read_remaining();

                    tokens.push(Token::new(
                        TokenKind::InlineHtml,
                        self.source.span(),
                        symbol,
                    ));
                    break;
                }
                // The double quote state is entered when inside a double-quoted string that
                // contains variables.
                StackFrame::DoubleQuote => self.double_quote(&mut tokens)?,
                // The shell exec state is entered when inside of a execution string (`).
                StackFrame::ShellExec => self.shell_exec(&mut tokens)?,
                // The doc string state is entered when tokenizing heredocs and nowdocs.
                StackFrame::DocString(kind, label) => {
                    let label = label.clone();

                    match kind {
                        TokenKind::StartHeredoc => self.heredoc(&mut tokens, label)?,
                        TokenKind::StartNowdoc => self.nowdoc(&mut tokens, label)?,
                        _ => unreachable!(),
                    }
                }
                // LookingForProperty is entered inside double quotes,
                // backticks, or a heredoc, expecting a variable name.
                // If one isn't found, it switches to scripting.
                StackFrame::LookingForVarname => {
                    if let Some(token) = self.looking_for_varname()? {
                        tokens.push(token);
                    }
                }
                // LookingForProperty is entered inside double quotes,
                // backticks, or a heredoc, expecting an arrow followed by a
                // property name.
                StackFrame::LookingForProperty => {
                    tokens.push(self.looking_for_property()?);
                }
                StackFrame::VarOffset => {
                    if self.source.eof() {
                        break;
                    }

                    tokens.push(self.var_offset()?);
                }
                // DocBlock is entered when parsing a DocBlock comment.
                // The lexer does this extra work to ensure that the comment
                // is in a usable state for the parser.
                StackFrame::DocBlock => self.docblock(&mut tokens)?,
            }
        }

        tokens.push(Token::new_without_symbol(
            TokenKind::Eof,
            self.source.span(),
        ));

        Ok(tokens)
    }

    fn skip_horizontal_whitespace(&mut self) {
        while let Some(true) = self
            .state
            .source
            .current()
            .map(|u: &u8| u == &b' ' || u == &b'\t')
        {
            self.source.next();
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(true) = self
            .state
            .source
            .current()
            .map(|u: &u8| u.is_ascii_whitespace())
        {
            self.source.next();
        }
    }

    fn read_and_skip_whitespace(&mut self) -> Vec<u8> {
        let mut buffer = Vec::new();
        while let Some(true) = self
            .state
            .source
            .current()
            .map(|u: &u8| u.is_ascii_whitespace())
        {
            buffer.push(*self.source.current().unwrap());
            self.source.next();
        }
        buffer
    }

    fn docblock_eol(&mut self) -> SyntaxResult<Token> {
        // We've already skipped the line break at this point.
        // We need to consume horizontal whitespace.
        self.skip_horizontal_whitespace();

        if matches!(self.source.current(), Some(b'*'))
            && !matches!(self.source.read(2), [b'*', b'/', ..])
        {
            self.source.next();

            // We also want to skip the next space character here.
            if let Some(b' ') = self.source.current() {
                self.source.next();
            }
        }

        let span = self.source.span();
        let symbol = self.source.span_range(span);

        Ok(Token::new_with_symbol(
            TokenKind::PhpDocEol,
            span,
            symbol.into(),
        ))
    }

    fn docblock(&mut self, tokens: &mut Vec<Token<'a>>) -> SyntaxResult<()> {
        while !self.source.eof() {
            self.source.start_token();

            if matches!(self.source.read(2), [b'\r', b'\n', ..] | [b'\n', ..]) {
                let b = self.source.current().unwrap();

                if b == &b'\r' {
                    self.source.skip(2);
                } else {
                    self.source.skip(1);
                }

                tokens.push(self.docblock_eol()?);

                continue;
            }

            match &self.source.read(2) {
                [b'@', ident_start!(), ..] => {
                    self.source.skip(2);

                    while let Some(ident_start!() | b'\\') = self.source.current() {
                        self.source.next();
                    }

                    let span = self.source.span();
                    let symbol = self.source.span_range(span);

                    tokens.push(Token::new_with_symbol(
                        TokenKind::PhpDocTag,
                        span,
                        symbol.into(),
                    ));

                    self.skip_horizontal_whitespace();
                }
                [b'$', ident_start!(), ..] => {
                    let variable = self.tokenize_variable();
                    let span = self.source.span();
                    let symbol = self.source.span_range(span);

                    tokens.push(Token::new_with_symbol(variable, span, symbol.into()));
                }
                [b'\\', ident_start!(), ..] => {
                    self.source.next();

                    let mut span = self.source.span();

                    let kind = match self.scripting()? {
                        Token {
                            kind: TokenKind::Identifier | TokenKind::QualifiedIdentifier,
                            span: ident_span,
                            ..
                        } => {
                            span.end = ident_span.end;

                            TokenKind::FullyQualifiedIdentifier
                        }
                        Token {
                            kind: TokenKind::True,
                            span: ident_span,
                            ..
                        } => {
                            span.end = ident_span.end;

                            TokenKind::FullyQualifiedIdentifier
                        }
                        Token {
                            kind: TokenKind::False,
                            span: ident_span,
                            ..
                        } => {
                            span.end = ident_span.end;

                            TokenKind::FullyQualifiedIdentifier
                        }
                        Token {
                            kind: TokenKind::Null,
                            span: ident_span,
                            ..
                        } => {
                            span.end = ident_span.end;

                            TokenKind::FullyQualifiedIdentifier
                        }
                        s => unreachable!("{:?}", s),
                    };

                    tokens.push(Token::new_with_symbol(
                        kind,
                        span,
                        self.source.span_range(span).into(),
                    ));
                }
                [b @ ident_start!(), ..] => {
                    self.source.next();
                    let mut qualified = false;
                    let mut last_was_slash = false;

                    let mut buffer = vec![*b];
                    while let Some(next @ ident!() | next @ b'\\') = self.source.current() {
                        if matches!(next, ident!()) {
                            buffer.push(*next);
                            self.source.next();
                            last_was_slash = false;
                            continue;
                        }

                        if *next == b'\\' && !last_was_slash {
                            qualified = true;
                            last_was_slash = true;
                            buffer.push(*next);
                            self.source.next();
                            continue;
                        }

                        break;
                    }

                    let kind = if qualified {
                        TokenKind::QualifiedIdentifier
                    } else {
                        identifier_to_keyword(&buffer).unwrap_or(TokenKind::Identifier)
                    };

                    let span = self.source.span();
                    let symbol = self.source.span_range(span);

                    tokens.push(Token::new_with_symbol(kind, span, symbol.into()));
                }
                [b'|', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::Pipe, span));
                }
                [b'&', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::Ampersand, span));
                }
                [b'!', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::Bang, span));
                }
                [b'?', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::Question, span));
                }
                [b'(', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::LeftParen, span));
                }
                [b')', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::RightParen, span));
                }
                [b'[', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::LeftBracket, span));
                }
                [b']', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::RightBracket, span));
                }
                [b'{', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::LeftBrace, span));
                }
                [b'}', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::RightBrace, span));
                }
                [b'<', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::LessThan, span));
                }
                [b'>', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::GreaterThan, span));
                }
                [b'.', b'.', b'.', ..] => {
                    self.source.skip(3);

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::Ellipsis, span));
                }
                [b'=', b'>', ..] => {
                    self.source.skip(2);

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::DoubleArrow, span));
                }
                [b'-', b'>', ..] => {
                    self.source.skip(2);

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::Arrow, span));
                }
                [b'=', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::Equals, span));
                }
                [b':', b':', ..] => {
                    self.source.skip(2);

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::DoubleColon, span));
                }
                [b':', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::Colon, span));
                }
                [b',', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::Comma, span));
                }
                [b'0'..=b'9', ..] => {
                    let number = self.tokenize_number()?;
                    let span = self.source.span();
                    let symbol = self.source.span_range(span);

                    tokens.push(Token::new_with_symbol(number, span, symbol.into()));
                }
                // We only need to consider these things strings if they are closed before the end of the line.
                [b'\'', ..] => {
                    // First we can grab the current offset, in case we need to backtrack.
                    let offset = self.source.offset();

                    self.source.next();

                    let is_single_quoted_string = loop {
                        let Some(c) = self.source.current() else {
                            break false;
                        };

                        // If we encounter a single quote, we can break out of the loop since we've found the end of the string.
                        if *c == b'\'' {
                            self.source.next();
                            break true;
                        }

                        // If we encounter the end of a line, we need to backtrack and treat the single quote as a single character.
                        if *c == b'\n' {
                            break false;
                        }

                        self.source.next();
                    };

                    if is_single_quoted_string {
                        let span = self.source.span();
                        let symbol = self.source.span_range(span);

                        tokens.push(Token::new_with_symbol(
                            TokenKind::LiteralSingleQuotedString,
                            span,
                            symbol.into(),
                        ));
                    } else {
                        self.source.goto(offset);
                        self.source.next();

                        let span = self.source.span();
                        let symbol = self.source.span_range(span);

                        tokens.push(Token::new_with_symbol(
                            TokenKind::PhpDocOther,
                            span,
                            symbol.into(),
                        ));
                    }
                }
                [b'"', ..] => {
                    let offset = self.source.offset();

                    self.source.next();

                    let is_single_quoted_string = loop {
                        let Some(c) = self.source.current() else {
                            break false;
                        };

                        // If we encounter a single quote, we can break out of the loop since we've found the end of the string.
                        if *c == b'"' {
                            self.source.next();
                            break true;
                        }

                        // If we encounter the end of a line, we need to backtrack and treat the single quote as a single character.
                        if *c == b'\n' {
                            break false;
                        }

                        self.source.next();
                    };

                    if is_single_quoted_string {
                        let span = self.source.span();
                        let symbol = self.source.span_range(span);

                        tokens.push(Token::new_with_symbol(
                            TokenKind::LiteralDoubleQuotedString,
                            span,
                            symbol.into(),
                        ));
                    } else {
                        self.source.goto(offset);
                        self.source.next();

                        let span = self.source.span();
                        let symbol = self.source.span_range(span);

                        tokens.push(Token::new_with_symbol(
                            TokenKind::PhpDocOther,
                            span,
                            symbol.into(),
                        ));
                    }
                }
                [b'*', b'/', ..] => {
                    self.source.skip(2);

                    tokens.push(Token::new_without_symbol(
                        TokenKind::ClosePhpDoc,
                        self.source.span(),
                    ));

                    break;
                }
                [b'*', ..] => {
                    self.source.next();

                    let span = self.source.span();

                    tokens.push(Token::new_without_symbol(TokenKind::Asterisk, span));
                }
                [b' ' | b'\t', ..] => {
                    self.skip_horizontal_whitespace();

                    let span = self.source.span();
                    let symbol = self.source.span_range(span);

                    tokens.push(Token::new_with_symbol(
                        TokenKind::PhpDocHorizontalWhitespace,
                        span,
                        symbol.into(),
                    ));
                }
                _ => {
                    self.source.next();

                    let span = self.source.span();
                    let symbol = self.source.span_range(span);

                    tokens.push(Token::new_with_symbol(
                        TokenKind::PhpDocOther,
                        span,
                        symbol.into(),
                    ));
                }
            }
        }

        self.state.exit();

        Ok(())
    }

    fn initial(&mut self, tokens: &mut Vec<Token>) -> SyntaxResult<()> {
        while self.source.current().is_some() {
            if self.source.at_case_insensitive(b"<?php", 5) {
                let inline_span = self.source.span();

                self.source.start_token();
                self.source.read_and_skip(5);
                let tag_span = self.source.span();

                self.state.replace(StackFrame::Scripting);

                if !inline_span.is_empty() {
                    tokens.push(Token::new_with_symbol(
                        TokenKind::InlineHtml,
                        inline_span,
                        ByteString::from(self.source.span_range(inline_span)),
                    ));
                }

                tokens.push(Token::new_without_symbol(
                    TokenKind::OpenTag(OpenTagKind::Full),
                    tag_span,
                ));

                return Ok(());
            } else if self.source.at_case_insensitive(b"<?=", 3) {
                let inline_span = self.source.span();

                self.source.start_token();
                self.source.skip(3);

                let tag_span = self.source.span();

                self.state.replace(StackFrame::Scripting);

                if !inline_span.is_empty() {
                    tokens.push(Token::new_with_symbol(
                        TokenKind::InlineHtml,
                        inline_span,
                        ByteString::from(self.source.span_range(inline_span)),
                    ));
                }

                tokens.push(Token::new_without_symbol(
                    TokenKind::OpenTag(OpenTagKind::Echo),
                    tag_span,
                ));

                return Ok(());
            } else if self.source.at_case_insensitive(b"<?", 2) {
                let inline_span = self.source.span();

                self.source.start_token();
                self.source.skip(2);
                let tag_span = self.source.span();

                self.state.replace(StackFrame::Scripting);

                if !inline_span.is_empty() {
                    tokens.push(Token::new_with_symbol(
                        TokenKind::InlineHtml,
                        inline_span,
                        ByteString::from(self.source.span_range(inline_span)),
                    ));
                }

                tokens.push(Token::new_without_symbol(
                    TokenKind::OpenTag(OpenTagKind::Short),
                    tag_span,
                ));

                return Ok(());
            }

            self.source.next();
        }

        let inline_span = self.source.span();

        tokens.push(Token::new_with_symbol(
            TokenKind::InlineHtml,
            inline_span,
            ByteString::from(self.source.span_range(inline_span)),
        ));

        Ok(())
    }

    fn scripting(&mut self) -> SyntaxResult<Token> {
        let (kind, with_symbol): (TokenKind, bool) = match self.source.read(3) {
            [b'!', b'=', b'='] => {
                self.source.skip(3);

                (TokenKind::BangDoubleEquals, false)
            }
            [b'?', b'?', b'='] => {
                self.source.skip(3);
                (TokenKind::DoubleQuestionEquals, false)
            }
            [b'?', b'-', b'>'] => {
                self.source.skip(3);
                (TokenKind::QuestionArrow, false)
            }
            [b'=', b'=', b'='] => {
                self.source.skip(3);
                (TokenKind::TripleEquals, false)
            }
            [b'.', b'.', b'.'] => {
                self.source.skip(3);
                (TokenKind::Ellipsis, false)
            }
            [b'`', ..] => {
                self.source.next();
                self.state.replace(StackFrame::ShellExec);
                (TokenKind::Backtick, false)
            }
            [b'@', ..] => {
                self.source.next();
                (TokenKind::At, false)
            }
            [b'!', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::BangEquals, false)
            }
            [b'!', ..] => {
                self.source.next();
                (TokenKind::Bang, false)
            }
            [b'&', b'&', ..] => {
                self.source.skip(2);
                (TokenKind::BooleanAnd, false)
            }
            [b'&', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::AmpersandEquals, false)
            }
            [b'&', ..] => {
                self.source.next();
                (TokenKind::Ampersand, false)
            }
            [b'?', b'>', ..] => {
                // This is a close tag, we can enter "Initial" mode again.
                self.source.skip(2);

                self.state.replace(StackFrame::Initial);

                (TokenKind::CloseTag, false)
            }
            [b'?', b'?', ..] => {
                self.source.skip(2);
                (TokenKind::DoubleQuestion, false)
            }
            [b'?', b':', ..] => {
                self.source.skip(2);
                (TokenKind::QuestionColon, false)
            }
            [b'?', ..] => {
                self.source.next();
                (TokenKind::Question, false)
            }
            [b'=', b'>', ..] => {
                self.source.skip(2);
                (TokenKind::DoubleArrow, false)
            }
            [b'=', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::DoubleEquals, false)
            }
            [b'=', ..] => {
                self.source.next();
                (TokenKind::Equals, false)
            }
            // Single quoted string.
            [b'\'', ..] => {
                self.source.skip(1);
                (self.tokenize_single_quote_string()?, true)
            }
            [b'b' | b'B', b'\'', ..] => {
                self.source.skip(2);
                (self.tokenize_single_quote_string()?, true)
            }
            [b'"', ..] => {
                self.source.skip(1);
                (self.tokenize_double_quote_string()?, true)
            }
            [b'b' | b'B', b'"', ..] => {
                self.source.skip(2);
                (self.tokenize_double_quote_string()?, true)
            }
            [b'$', ident_start!(), ..] => (self.tokenize_variable(), true),
            [b'$', ..] => {
                self.source.next();
                (TokenKind::Dollar, false)
            }
            [b'.', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::DotEquals, false)
            }
            [b'0'..=b'9', ..] => (self.tokenize_number()?, true),
            [b'.', b'0'..=b'9', ..] => (self.tokenize_number()?, true),
            [b'.', ..] => {
                self.source.next();
                (TokenKind::Dot, false)
            }
            [b'\\', ident_start!(), ..] => {
                self.source.next();

                let mut span = self.source.span();

                match self.scripting()? {
                    Token {
                        kind: TokenKind::Identifier | TokenKind::QualifiedIdentifier,
                        span: ident_span,
                        ..
                    } => {
                        span.end = ident_span.end;

                        (TokenKind::FullyQualifiedIdentifier, true)
                    }
                    Token {
                        kind: TokenKind::True,
                        span: ident_span,
                        ..
                    } => {
                        span.end = ident_span.end;

                        (TokenKind::FullyQualifiedIdentifier, true)
                    }
                    Token {
                        kind: TokenKind::False,
                        span: ident_span,
                        ..
                    } => {
                        span.end = ident_span.end;

                        (TokenKind::FullyQualifiedIdentifier, true)
                    }
                    Token {
                        kind: TokenKind::Null,
                        span: ident_span,
                        ..
                    } => {
                        span.end = ident_span.end;

                        (TokenKind::FullyQualifiedIdentifier, true)
                    }
                    s => unreachable!("{:?}", s),
                }
            }
            [b'\\', ..] => {
                self.source.next();
                (TokenKind::NamespaceSeparator, false)
            }
            [b'/', b'*', ..] => {
                self.source.next();

                let mut kind = TokenKind::MultiLineComment;
                #[allow(unused)]
                let mut with_symbol = true;

                loop {
                    match self.source.read(2) {
                        #[cfg(feature = "docblocks")]
                        [b'*', b'*', ..] => {
                            self.source.skip(2);

                            kind = TokenKind::OpenPhpDoc;
                            with_symbol = false;

                            self.state.enter(StackFrame::DocBlock);

                            break;
                        }
                        [b'*', b'/', ..] => {
                            self.source.skip(2);
                            break;
                        }
                        #[cfg(not(feature = "docblocks"))]
                        [b'*', b'*', ..] if kind != TokenKind::DocBlockComment => {
                            self.source.skip(2);
                            kind = TokenKind::DocBlockComment;
                        }
                        &[..] => {
                            self.source.next();
                        }
                    }
                }

                (kind, with_symbol)
            }
            [b'#', b'[', ..] => {
                self.source.skip(2);
                (TokenKind::Attribute, false)
            }
            [ch @ b'/', b'/', ..] | [ch @ b'#', ..] => {
                let kind = if *ch == b'/' {
                    self.source.skip(2);
                    TokenKind::SingleLineComment
                } else {
                    self.source.next();
                    TokenKind::HashMarkComment
                };

                while let Some(c) = self.source.current() {
                    if *c == b'\n' {
                        self.source.next();
                        break;
                    }

                    if self.source.read(2) == [b'?', b'>'] {
                        break;
                    }

                    self.source.next();
                }

                (kind, true)
            }
            [b'/', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::SlashEquals, false)
            }
            [b'/', ..] => {
                self.source.next();
                (TokenKind::Slash, false)
            }
            [b'*', b'*', b'=', ..] => {
                self.source.skip(3);
                (TokenKind::PowEquals, false)
            }
            [b'<', b'<', b'='] => {
                self.source.skip(3);

                (TokenKind::LeftShiftEquals, false)
            }
            [b'<', b'=', b'>'] => {
                self.source.skip(3);
                (TokenKind::Spaceship, false)
            }
            [b'>', b'>', b'='] => {
                self.source.skip(3);
                (TokenKind::RightShiftEquals, false)
            }
            [b'<', b'<', b'<'] => {
                self.source.skip(3);
                let mut buffer = b"<<<".to_vec();
                buffer.extend(self.read_and_skip_whitespace());

                let kind = match self.source.read(1) {
                    [b'\''] => {
                        buffer.push(b'\'');
                        self.source.next();
                        TokenKind::StartNowdoc
                    }
                    [b'"'] => {
                        buffer.push(b'"');
                        self.source.next();
                        TokenKind::StartHeredoc
                    }
                    [_, ..] => TokenKind::StartHeredoc,
                    [] => {
                        return Err(SyntaxError::UnexpectedEndOfFile(self.source.span()));
                    }
                };

                let label: ByteString = match self.peek_identifier() {
                    Some(_) => self.consume_identifier().into(),
                    None => {
                        return match self.source.current() {
                            Some(c) => Err(SyntaxError::UnexpectedCharacter(
                                *c,
                                self.source.span(),
                            )),
                            None => Err(SyntaxError::UnexpectedEndOfFile(self.source.span())),
                        }
                    }
                };

                buffer.extend_from_slice(&label);

                if kind == TokenKind::StartNowdoc {
                    match self.source.current() {
                        Some(b'\'') => {
                            buffer.push(b'\'');
                            self.source.next();
                        }
                        _ => {
                            // FIXME: this is most likely a bug, what if current is none?
                            return Err(SyntaxError::UnexpectedCharacter(
                                *self.source.current().unwrap(),
                                self.source.span(),
                            ));
                        }
                    };
                } else if let Some(b'"') = self.source.current() {
                    buffer.push(b'"');
                    self.source.next();
                }

                if !matches!(self.source.current(), Some(b'\n')) {
                    return Err(SyntaxError::UnexpectedCharacter(
                        *self.source.current().unwrap(),
                        self.source.span(),
                    ));
                }

                self.source.next();
                self.state.replace(StackFrame::DocString(kind, label.clone()));

                (kind, true)
            }
            [b'*', b'*', ..] => {
                self.source.skip(2);
                (TokenKind::Pow, false)
            }
            [b'*', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::AsteriskEquals, false)
            }
            [b'*', ..] => {
                self.source.next();
                (TokenKind::Asterisk, false)
            }
            [b'|', b'|', ..] => {
                self.source.skip(2);
                (TokenKind::BooleanOr, false)
            }
            [b'|', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::PipeEquals, false)
            }
            [b'|', ..] => {
                self.source.next();
                (TokenKind::Pipe, false)
            }
            [b'^', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::CaretEquals, false)
            }
            [b'^', ..] => {
                self.source.next();
                (TokenKind::Caret, false)
            }
            [b'{', ..] => {
                self.source.next();
                self.state.enter(StackFrame::Scripting);
                (TokenKind::LeftBrace, false)
            }
            [b'}', ..] => {
                self.source.next();
                self.state.exit();
                (TokenKind::RightBrace, false)
            }
            [b'(', ..] => {
                self.source.next();

                // Inlined so we can add whitespace to the buffer.
                while let Some(true) = self
                    .state
                    .source
                    .current()
                    .map(|u: &u8| u.is_ascii_whitespace())
                {
                    self.source.next();
                }

                if self.source.at_case_insensitive(b"int", 3) {
                    if self.source.at_case_insensitive(b"integer", 7)
                        && self.source.peek_ignoring_whitespace(7, 1) == [b')']
                    {
                        self.source.read_and_skip(7);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::IntegerCast, true)
                    } else if self.source.peek_ignoring_whitespace(3, 1) == [b')'] {
                        self.source.read_and_skip(3);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::IntCast, true)
                    } else {
                        (TokenKind::LeftParen, false)
                    }
                } else if self.source.at_case_insensitive(b"bool", 4) {
                    if self.source.at_case_insensitive(b"boolean", 7)
                        && self.source.peek_ignoring_whitespace(7, 1) == [b')']
                    {
                        self.source.read_and_skip(7);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::BooleanCast, true)
                    } else if self.source.peek_ignoring_whitespace(4, 1) == [b')'] {
                        self.source.read_and_skip(4);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::BoolCast, true)
                    } else {
                        (TokenKind::LeftParen, true)
                    }
                } else if self.source.at_case_insensitive(b"float", 5) {
                    if self.source.peek_ignoring_whitespace(5, 1) == [b')'] {
                        self.source.read_and_skip(5);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::FloatCast, true)
                    } else {
                        (TokenKind::LeftParen, true)
                    }
                } else if self.source.at_case_insensitive(b"double", 6) {
                    if self.source.peek_ignoring_whitespace(6, 1) == [b')'] {
                        self.source.read_and_skip(6);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::DoubleCast, true)
                    } else {
                        (TokenKind::LeftParen, true)
                    }
                } else if self.source.at_case_insensitive(b"real", 4) {
                    if self.source.peek_ignoring_whitespace(4, 1) == [b')'] {
                        self.source.read_and_skip(4);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::RealCast, true)
                    } else {
                        (TokenKind::LeftParen, true)
                    }
                } else if self.source.at_case_insensitive(b"string", 6) {
                    if self.source.peek_ignoring_whitespace(6, 1) == [b')'] {
                        self.source.read_and_skip(6);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::StringCast, true)
                    } else {
                        (TokenKind::LeftParen, true)
                    }
                } else if self.source.at_case_insensitive(b"binary", 6) {
                    if self.source.peek_ignoring_whitespace(6, 1) == [b')'] {
                        self.source.read_and_skip(6);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::BinaryCast, true)
                    } else {
                        (TokenKind::LeftParen, true)
                    }
                } else if self.source.at_case_insensitive(b"array", 5) {
                    if self.source.peek_ignoring_whitespace(5, 1) == [b')'] {
                        self.source.read_and_skip(5);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::ArrayCast, true)
                    } else {
                        (TokenKind::LeftParen, true)
                    }
                } else if self.source.at_case_insensitive(b"object", 6) {
                    if self.source.peek_ignoring_whitespace(6, 1) == [b')'] {
                        self.source.read_and_skip(6);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::ObjectCast, true)
                    } else {
                        (TokenKind::LeftParen, true)
                    }
                } else if self.source.at_case_insensitive(b"unset", 5) {
                    if self.source.peek_ignoring_whitespace(5, 1) == [b')'] {
                        self.source.read_and_skip(5);
                        self.read_and_skip_whitespace();
                        self.source.read_and_skip(1);

                        (TokenKind::UnsetCast, true)
                    } else {
                        (TokenKind::LeftParen, true)
                    }
                } else {
                    (TokenKind::LeftParen, false)
                }
            }
            [b')', ..] => {
                self.source.next();
                (TokenKind::RightParen, false)
            }
            [b';', ..] => {
                self.source.next();
                (TokenKind::SemiColon, false)
            }
            [b'+', b'+', ..] => {
                self.source.skip(2);
                (TokenKind::Increment, false)
            }
            [b'+', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::PlusEquals, false)
            }
            [b'+', ..] => {
                self.source.next();
                (TokenKind::Plus, false)
            }
            [b'%', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::PercentEquals, false)
            }
            [b'%', ..] => {
                self.source.next();
                (TokenKind::Percent, false)
            }
            [b'-', b'-', ..] => {
                self.source.skip(2);
                (TokenKind::Decrement, false)
            }
            [b'-', b'>', ..] => {
                self.source.skip(2);
                (TokenKind::Arrow, false)
            }
            [b'-', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::MinusEquals, false)
            }
            [b'-', ..] => {
                self.source.next();
                (TokenKind::Minus, false)
            }
            [b'<', b'<', ..] => {
                self.source.skip(2);
                (TokenKind::LeftShift, false)
            }
            [b'<', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::LessThanEquals, false)
            }
            [b'<', b'>', ..] => {
                self.source.skip(2);
                (TokenKind::AngledLeftRight, false)
            }
            [b'<', ..] => {
                self.source.next();
                (TokenKind::LessThan, false)
            }
            [b'>', b'>', ..] => {
                self.source.skip(2);
                (TokenKind::RightShift, false)
            }
            [b'>', b'=', ..] => {
                self.source.skip(2);
                (TokenKind::GreaterThanEquals, false)
            }
            [b'>', ..] => {
                self.source.next();
                (TokenKind::GreaterThan, false)
            }
            [b',', ..] => {
                self.source.next();
                (TokenKind::Comma, false)
            }
            [b'[', ..] => {
                self.source.next();
                (TokenKind::LeftBracket, false)
            }
            [b']', ..] => {
                self.source.next();
                (TokenKind::RightBracket, false)
            }
            [b':', b':', ..] => {
                self.source.skip(2);
                (TokenKind::DoubleColon, false)
            }
            [b':', ..] => {
                self.source.next();
                (TokenKind::Colon, false)
            }
            [b'~', ..] => {
                self.source.next();
                (TokenKind::BitwiseNot, false)
            }
            [b @ ident_start!(), ..] => {
                self.source.next();
                let mut qualified = false;
                let mut last_was_slash = false;

                let mut buffer = vec![*b];
                while let Some(next @ ident!() | next @ b'\\') = self.source.current() {
                    if matches!(next, ident!()) {
                        buffer.push(*next);
                        self.source.next();
                        last_was_slash = false;
                        continue;
                    }

                    if *next == b'\\' && !last_was_slash {
                        qualified = true;
                        last_was_slash = true;
                        buffer.push(*next);
                        self.source.next();
                        continue;
                    }

                    break;
                }

                if qualified {
                    (TokenKind::QualifiedIdentifier, true)
                } else {
                    let kind = identifier_to_keyword(&buffer).unwrap_or(TokenKind::Identifier);

                    if kind == TokenKind::HaltCompiler {
                        match self.source.read(3) {
                            [b'(', b')', b';'] => {
                                self.source.skip(3);
                                self.state.replace(StackFrame::Halted);
                            }
                            _ => {
                                return Err(SyntaxError::InvalidHaltCompiler(
                                    self.source.span(),
                                ))
                            }
                        }
                    }

                    (kind, true)
                }
            }
            [b, ..] => unimplemented!(
                "<scripting> {} at offset: {}",
                *b as char,
                self.source.offset(),
            ),
            // We should never reach this point since we have the empty checks surrounding
            // the call to this function, but it's better to be safe than sorry.
            [] => return Err(SyntaxError::UnexpectedEndOfFile(self.source.span())),
        };

        let mut span = self.source.span();

        // FIXME: This is a bit hacky, but it works for now.
        //        We're doing this so that the closing double quote isn't included in the span.
        if kind == TokenKind::LiteralDoubleQuotedString {
            span.end -= 1;
        }

        Ok(Token::new(
            kind,
            span,
            match with_symbol {
                true => Some(ByteString::from(self.source.span_range(span))),
                false => None,
            },
        ))
    }

    fn double_quote(&mut self, tokens: &mut Vec<Token>) -> SyntaxResult<()> {
        #[allow(unused_assignments)]
        let mut buffer_span = None;

        let (kind, with_symbol, span) = loop {
            match self.source.read(3) {
                [b'$', b'{', ..] => {
                    buffer_span = Some(self.source.span());
                    self.source.start_token();
                    self.source.skip(2);
                    self.state.enter(StackFrame::LookingForVarname);
                    break (TokenKind::DollarLeftBrace, false, self.source.span());
                }
                [b'{', b'$', ..] => {
                    buffer_span = Some(self.source.span());
                    self.source.start_token();
                    // Intentionally only consume the left brace.
                    self.source.next();
                    self.state.enter(StackFrame::Scripting);
                    break (TokenKind::LeftBrace, false, self.source.span());
                }
                [b'"', ..] => {
                    buffer_span = Some(self.source.span());
                    self.source.start_token();
                    self.source.next();
                    self.state.replace(StackFrame::Scripting);
                    break (TokenKind::DoubleQuote, false, self.source.span());
                }
                &[b'\\', b'"' | b'\\' | b'$', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'n', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'r', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b't', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'v', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'e', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'f', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'x', b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')] => {
                    self.source.skip(3);

                    let mut hex = String::from(b as char);
                    if let Some(b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) =
                        self.source.current()
                    {
                        self.source.next();
                        hex.push(*b as char);
                    }
                }
                &[b'\\', b'u', b'{'] => {
                    self.source.skip(3);

                    let mut code_point = String::new();
                    while let Some(b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) =
                        self.source.current()
                    {
                        self.source.next();
                        code_point.push(*b as char);
                    }

                    if code_point.is_empty() || self.source.current() != Some(&b'}') {
                        return Err(SyntaxError::InvalidUnicodeEscape(self.source.span()));
                    }
                    self.source.next();

                    let c = if let Ok(c) = u32::from_str_radix(&code_point, 16) {
                        c
                    } else {
                        return Err(SyntaxError::InvalidUnicodeEscape(self.source.span()));
                    };

                    if char::from_u32(c).is_none() {
                        return Err(SyntaxError::InvalidUnicodeEscape(self.source.span()));
                    }
                }
                &[b'\\', b @ b'0'..=b'7', ..] => {
                    self.source.skip(2);

                    let mut octal = String::from(b as char);
                    if let Some(b @ b'0'..=b'7') = self.source.current() {
                        self.source.next();
                        octal.push(*b as char);
                    }
                    if let Some(b @ b'0'..=b'7') = self.source.current() {
                        self.source.next();
                        octal.push(*b as char);
                    }

                    if u8::from_str_radix(&octal, 8).is_err() {
                        return Err(SyntaxError::InvalidOctalEscape(self.source.span()));
                    }
                }
                [b'$', ident_start!(), ..] => {
                    buffer_span = Some(self.source.span());
                    self.source.start_token();
                    let mut var = self.source.read_and_skip(1).to_vec();
                    var.extend(self.consume_identifier());

                    match self.source.read(4) {
                        [b'[', ..] => self.state.enter(StackFrame::VarOffset),
                        [b'-', b'>', ident_start!(), ..] | [b'?', b'-', b'>', ident_start!()] => {
                            self.state.enter(StackFrame::LookingForProperty)
                        }
                        _ => {}
                    }

                    break (TokenKind::Variable, true, self.source.span());
                }
                &[_, ..] => {
                    self.source.next();
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(self.source.span())),
            }
        };

        let buffer_span = match buffer_span {
            Some(span) => span,
            None => self.source.span(),
        };

        if !buffer_span.is_empty() {
            tokens.push(Token::new_with_symbol(
                TokenKind::StringPart,
                buffer_span,
                ByteString::from(self.source.span_range(buffer_span)),
            ));
        }

        tokens.push(Token::new(
            kind,
            span,
            match with_symbol {
                true => Some(ByteString::from(self.source.span_range(span))),
                false => None,
            },
        ));

        Ok(())
    }

    fn shell_exec(&mut self, tokens: &mut Vec<Token>) -> SyntaxResult<()> {
        let mut buffer_span = None;

        let (kind, with_symbol) = loop {
            match self.source.read(2) {
                [b'$', b'{'] => {
                    buffer_span = Some(self.source.span());
                    self.source.start_token();
                    self.source.skip(2);
                    self.state.enter(StackFrame::LookingForVarname);
                    break (TokenKind::DollarLeftBrace, false);
                }
                [b'{', b'$'] => {
                    buffer_span = Some(self.source.span());
                    self.source.start_token();
                    // Intentionally only consume the left brace.
                    self.source.next();
                    self.state.enter(StackFrame::Scripting);
                    break (TokenKind::LeftBrace, false);
                }
                [b'`', ..] => {
                    self.source.next();
                    self.state.replace(StackFrame::Scripting);
                    break (TokenKind::Backtick, false);
                }
                [b'$', ident_start!()] => {
                    let mut var = self.source.read_and_skip(1).to_vec();
                    var.extend(self.consume_identifier());

                    match self.source.read(4) {
                        [b'[', ..] => self.state.enter(StackFrame::VarOffset),
                        [b'-', b'>', ident_start!(), ..] | [b'?', b'-', b'>', ident_start!()] => {
                            self.state.enter(StackFrame::LookingForProperty)
                        }
                        _ => {}
                    }

                    break (TokenKind::Variable, true);
                }
                &[_, ..] => {
                    self.source.next();
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(self.source.span())),
            }
        };

        let buffer_span = match buffer_span {
            Some(span) => span,
            None => self.source.span(),
        };

        if !buffer_span.is_empty() {
            tokens.push(Token::new_with_symbol(
                TokenKind::StringPart,
                buffer_span,
                ByteString::from(self.source.span_range(buffer_span)),
            ))
        }

        let span = self.source.span();
        tokens.push(Token::new(
            kind,
            span,
            match with_symbol {
                true => Some(ByteString::from(self.source.span_range(span))),
                false => None,
            },
        ));

        Ok(())
    }

    fn heredoc(&mut self, tokens: &mut Vec<Token>, label: ByteString) -> SyntaxResult<()> {
        #[allow(unused_assignments)]
        let mut buffer_span = None;
        let mut last_was_newline = false;

        let (kind, with_symbol) = loop {
            match self.source.read(3) {
                [b'\\', b'"' | b'\\' | b'$', ..] => {
                    self.source.skip(2);
                }
                [b'$', b'{', ..] => {
                    buffer_span = Some(self.source.span());
                    self.source.start_token();
                    self.source.skip(2);
                    self.state.enter(StackFrame::LookingForVarname);
                    break (TokenKind::DollarLeftBrace, false);
                }
                [b'{', b'$', ..] => {
                    buffer_span = Some(self.source.span());
                    self.source.start_token();
                    // Intentionally only consume the left brace.
                    self.source.next();
                    self.state.enter(StackFrame::Scripting);
                    break (TokenKind::LeftBrace, false);
                }
                [b'$', ident_start!(), ..] => {
                    buffer_span = Some(self.source.span());
                    self.source.start_token();
                    let mut var = self.source.read_and_skip(1).to_vec();
                    var.extend(self.consume_identifier());

                    match self.source.read(4) {
                        [b'[', ..] => self.state.enter(StackFrame::VarOffset),
                        [b'-', b'>', ident_start!(), ..] | [b'?', b'-', b'>', ident_start!()] => {
                            self.state.enter(StackFrame::LookingForProperty)
                        }
                        _ => {}
                    }

                    break (TokenKind::Variable, true);
                }
                // If we find a new-line, we can start to check if we can see the EndHeredoc token.
                [b'\n', ..] => {
                    last_was_newline = true;
                    self.source.next();

                    // Check if we can see the closing label right here.
                    if self.source.at(&label, label.len()) {
                        buffer_span = Some(self.source.span());
                        self.source.start_token();
                        self.source.skip(label.len());
                        self.state.replace(StackFrame::Scripting);
                        break (TokenKind::EndHeredoc, true);
                    }

                    self.skip_horizontal_whitespace();

                    // We've consumed all leading whitespace on this line now,
                    // so let's try to read the label again.
                    if self.source.at(&label, label.len()) {
                        buffer_span = Some(self.source.span());

                        self.source.start_token();
                        self.source.skip(label.len());
                        self.state.replace(StackFrame::Scripting);

                        break (TokenKind::EndHeredoc, true);
                    }
                }
                &[b, ..] => {
                    self.source.next();
                    last_was_newline = b == b'\n';
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(self.source.span())),
            }
        };

        let mut buffer_span = match buffer_span {
            Some(span) => span,
            None => self.source.span(),
        };

        // Any trailing line breaks should be removed from the final heredoc.
        if last_was_newline {
            buffer_span.end -= 1;
        }

        if !buffer_span.is_empty() {
            tokens.push(Token::new_with_symbol(
                TokenKind::StringPart,
                buffer_span,
                ByteString::from(self.source.span_range(buffer_span)),
            ));
        }

        let span = self.source.span();
        tokens.push(Token::new(
            kind,
            span,
            match with_symbol {
                true => Some(ByteString::from(self.source.span_range(span))),
                false => None,
            },
        ));

        Ok(())
    }

    fn nowdoc(&mut self, tokens: &mut Vec<Token>, label: ByteString) -> SyntaxResult<()> {
        #[allow(unused_assignments)]
        let mut buffer_span = None;
        let mut last_was_newline = false;

        let (kind, with_symbol) = loop {
            match self.source.read(3) {
                // If we find a new-line, we can start to check if we can see the EndHeredoc token.
                [b'\n', ..] => {
                    self.source.next();

                    // Check if we can see the closing label right here.
                    if self.source.at(&label, label.len()) {
                        buffer_span = Some(self.source.span());
                        self.source.start_token();
                        self.source.skip(label.len());
                        self.state.replace(StackFrame::Scripting);
                        last_was_newline = true;
                        break (TokenKind::EndNowdoc, true);
                    }

                    self.skip_horizontal_whitespace();

                    // We've consumed all leading whitespace on this line now,
                    // so let's try to read the label again.
                    if self.source.at(&label, label.len()) {
                        buffer_span = Some(self.source.span());
                        self.source.start_token();

                        // If we get here, only 1 type of indentation was found. We can move
                        // the process along by reading over the label and breaking out
                        // with the EndHeredoc token, storing the kind and amount of whitespace.
                        self.source.skip(label.len());
                        self.state.replace(StackFrame::Scripting);
                        break (TokenKind::EndNowdoc, true);
                    }
                }
                &[b, ..] => {
                    self.source.next();
                    last_was_newline = b == b'\n';
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(self.source.span())),
            }
        };

        let mut buffer_span = match buffer_span {
            Some(span) => span,
            None => self.source.span(),
        };

        // Any trailing line breaks should be removed from the final heredoc.
        if last_was_newline {
            buffer_span.end -= 1;
        }

        tokens.push(Token::new_with_symbol(
            TokenKind::StringPart,
            buffer_span,
            ByteString::from(self.source.span_range(buffer_span)),
        ));

        let span = self.source.span();

        tokens.push(Token::new(
            kind,
            span,
            match with_symbol {
                true => Some(ByteString::from(self.source.span_range(span))),
                false => None,
            },
        ));

        Ok(())
    }

    fn looking_for_varname(&mut self) -> SyntaxResult<Option<Token>> {
        let identifier = self.peek_identifier();

        if let Some(ident) = identifier {
            if let [b'[' | b'}'] = self.source.peek(ident.len(), 1) {
                self.source.skip(ident.len());
                let span = self.source.span();
                self.state.replace(StackFrame::Scripting);
                return Ok(Some(Token::new_with_symbol(
                    TokenKind::Identifier,
                    span,
                    ByteString::from(self.source.span_range(span)),
                )));
            }
        }

        self.state.replace(StackFrame::Scripting);

        Ok(None)
    }

    fn looking_for_property(&mut self) -> SyntaxResult<Token> {
        let (kind, with_symbol) = match self.source.read(3) {
            [b'?', b'-', b'>'] => {
                self.source.skip(3);
                (TokenKind::QuestionArrow, false)
            }
            [b'-', b'>', ..] => {
                self.source.skip(2);
                (TokenKind::Arrow, false)
            }
            &[ident_start!(), ..] => {
                self.consume_identifier();
                self.state.exit();
                (TokenKind::Identifier, true)
            }
            // Should be impossible as we already looked ahead this far inside double_quote.
            _ => unreachable!(),
        };

        let span = self.source.span();

        Ok(Token::new(
            kind,
            span,
            match with_symbol {
                true => Some(ByteString::from(self.source.span_range(span))),
                false => None,
            },
        ))
    }

    fn var_offset(&mut self) -> SyntaxResult<Token> {
        let (kind, with_symbol) = match self.source.read(2) {
            [b'$', ident_start!()] => (self.tokenize_variable(), true),
            [b'0'..=b'9', ..] => {
                // FIXME: all integer literals are allowed, but only decimal integers with no underscores
                // are actually treated as numbers. Others are treated as strings.
                // Float literals are not allowed, but that could be handled in the parser.
                (self.tokenize_number()?, true)
            }
            [b'[', ..] => {
                self.source.next();
                (TokenKind::LeftBracket, false)
            }
            [b'-', ..] => {
                self.source.next();
                (TokenKind::Minus, false)
            }
            [b']', ..] => {
                self.source.next();
                self.state.exit();
                (TokenKind::RightBracket, false)
            }
            &[ident_start!(), ..] => {
                self.consume_identifier();
                (TokenKind::Identifier, true)
            }
            &[b, ..] => return Err(SyntaxError::UnrecognisedToken(b, self.source.span())),
            [] => return Err(SyntaxError::UnexpectedEndOfFile(self.source.span())),
        };

        let span = self.source.span();

        Ok(Token::new(
            kind,
            span,
            match with_symbol {
                true => Some(ByteString::from(self.source.span_range(span))),
                false => None,
            },
        ))
    }

    fn tokenize_single_quote_string(&mut self) -> SyntaxResult<TokenKind> {
        loop {
            match self.source.read(2) {
                [b'\'', ..] => {
                    self.source.next();
                    break;
                }
                &[b'\\', b'\'' | b'\\'] => {
                    self.source.skip(2);
                }
                &[_, ..] => {
                    self.source.next();
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(self.source.span())),
            }
        }

        Ok(TokenKind::LiteralSingleQuotedString)
    }

    fn tokenize_double_quote_string(&mut self) -> SyntaxResult<TokenKind> {
        self.source.start_token();

        let constant = loop {
            match self.source.read(3) {
                [b'"', ..] => {
                    self.source.next();
                    break true;
                }
                &[b'\\', b'"' | b'\\' | b'$', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'n', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'r', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b't', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'v', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'e', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'f', ..] => {
                    self.source.skip(2);
                }
                &[b'\\', b'x', b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')] => {
                    self.source.skip(3);

                    let mut hex = String::from(b as char);
                    if let Some(b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) =
                        self.source.current()
                    {
                        self.source.next();
                        hex.push(*b as char);
                    }
                }
                &[b'\\', b'u', b'{'] => {
                    self.source.skip(3);

                    let mut code_point = String::new();
                    while let Some(b @ (b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) =
                        self.source.current()
                    {
                        self.source.next();
                        code_point.push(*b as char);
                    }

                    if code_point.is_empty() || self.source.current() != Some(&b'}') {
                        return Err(SyntaxError::InvalidUnicodeEscape(self.source.span()));
                    }
                    self.source.next();

                    let c = if let Ok(c) = u32::from_str_radix(&code_point, 16) {
                        c
                    } else {
                        return Err(SyntaxError::InvalidUnicodeEscape(self.source.span()));
                    };

                    if char::from_u32(c).is_none() {
                        return Err(SyntaxError::InvalidUnicodeEscape(self.source.span()));
                    }
                }
                &[b'\\', b @ b'0'..=b'7', ..] => {
                    self.source.skip(2);

                    let mut octal = String::from(b as char);
                    if let Some(b @ b'0'..=b'7') = self.source.current() {
                        self.source.next();
                        octal.push(*b as char);
                    }

                    if let Some(b @ b'0'..=b'7') = self.source.current() {
                        self.source.next();
                        octal.push(*b as char);
                    }

                    if u8::from_str_radix(&octal, 8).is_err() {
                        return Err(SyntaxError::InvalidOctalEscape(self.source.span()));
                    }
                }
                [b'$', ident_start!(), ..] | [b'{', b'$', ..] | [b'$', b'{', ..] => {
                    break false;
                }
                &[_, ..] => {
                    self.source.next();
                }
                [] => return Err(SyntaxError::UnexpectedEndOfFile(self.source.span())),
            }
        };

        Ok(if constant {
            TokenKind::LiteralDoubleQuotedString
        } else {
            self.state.replace(StackFrame::DoubleQuote);
            TokenKind::StringPart
        })
    }

    fn peek_identifier(&self) -> Option<&[u8]> {
        let mut size = 0;

        if let [ident_start!()] = self.source.read(1) {
            size += 1;
            while let [ident!()] = self.source.peek(size, 1) {
                size += 1;
            }

            Some(self.source.read(size))
        } else {
            None
        }
    }

    fn consume_identifier(&mut self) -> Vec<u8> {
        let ident = self.peek_identifier().unwrap().to_vec();
        self.source.skip(ident.len());

        ident
    }

    fn tokenize_variable(&mut self) -> TokenKind {
        self.source.skip(1);
        self.consume_identifier();

        TokenKind::Variable
    }

    fn tokenize_number(&mut self) -> SyntaxResult<TokenKind> {
        let (base, kind) = match self.source.read(2) {
            [b'0', b'B' | b'b'] => {
                self.source.skip(2);
                (2, NumberKind::Int)
            }
            [b'0', b'O' | b'o'] => {
                self.source.skip(2);
                (8, NumberKind::Int)
            }
            [b'0', b'X' | b'x'] => {
                self.source.skip(2);
                (16, NumberKind::Int)
            }
            [b'0', ..] => (10, NumberKind::OctalOrFloat),
            [b'.', ..] => (10, NumberKind::Float),
            _ => (10, NumberKind::IntOrFloat),
        };

        if kind != NumberKind::Float {
            self.read_digits(base);
            if kind == NumberKind::Int {
                return Ok(TokenKind::LiteralInteger);
            }
        }

        // Remaining cases: decimal integer, legacy octal integer, or float.
        let is_float = matches!(
            self.source.read(3),
            [b'.', ..] | [b'e' | b'E', b'-' | b'+', b'0'..=b'9'] | [b'e' | b'E', b'0'..=b'9', ..]
        );

        if !is_float {
            return Ok(TokenKind::LiteralInteger);
        }

        if let Some(b'.') = self.source.current() {
            self.source.next();
            self.read_digits(10);
        }

        if let Some(b'e' | b'E') = self.source.current() {
            self.source.next();

            if let Some(b'-' | b'+') = self.source.current() {
                self.source.next();
            }

            self.read_digits(10);
        }

        Ok(TokenKind::LiteralFloat)
    }

    fn read_digits(&mut self, base: usize) {
        if base == 16 {
            self.read_digits_fn(u8::is_ascii_hexdigit);
        } else {
            let max = b'0' + base as u8;
            self.read_digits_fn(|b| (b'0'..max).contains(b));
        };
    }

    fn read_digits_fn<F: Fn(&u8) -> bool>(&mut self, is_digit: F) {
        if let Some(b) = self.source.current() {
            if is_digit(b) {
                self.source.next();
            } else {
                return;
            }
        }

        loop {
            match self.source.read(2) {
                [b, ..] if is_digit(b) => {
                    self.source.next();
                }
                [b'_', b] if is_digit(b) => {
                    self.source.next();
                    self.source.next();
                }
                _ => {
                    break;
                }
            }
        }
    }
}

const KEYWORDS: [(&[u8], TokenKind); 85] = [
    (b"eval", TokenKind::Eval),
    (b"die", TokenKind::Die),
    (b"empty", TokenKind::Empty),
    (b"isset", TokenKind::Isset),
    (b"unset", TokenKind::Unset),
    (b"exit", TokenKind::Exit),
    (b"enddeclare", TokenKind::EndDeclare),
    (b"endswitch", TokenKind::EndSwitch),
    (b"endfor", TokenKind::EndFor),
    (b"endwhile", TokenKind::EndWhile),
    (b"endforeach", TokenKind::EndForeach),
    (b"endif", TokenKind::EndIf),
    (b"from", TokenKind::From),
    (b"and", TokenKind::LogicalAnd),
    (b"or", TokenKind::LogicalOr),
    (b"xor", TokenKind::LogicalXor),
    (b"print", TokenKind::Print),
    (b"__halt_compiler", TokenKind::HaltCompiler),
    (b"readonly", TokenKind::Readonly),
    (b"global", TokenKind::Global),
    (b"match", TokenKind::Match),
    (b"abstract", TokenKind::Abstract),
    (b"array", TokenKind::Array),
    (b"as", TokenKind::As),
    (b"break", TokenKind::Break),
    (b"case", TokenKind::Case),
    (b"catch", TokenKind::Catch),
    (b"class", TokenKind::Class),
    (b"clone", TokenKind::Clone),
    (b"continue", TokenKind::Continue),
    (b"const", TokenKind::Const),
    (b"declare", TokenKind::Declare),
    (b"default", TokenKind::Default),
    (b"do", TokenKind::Do),
    (b"echo", TokenKind::Echo),
    (b"else", TokenKind::Else),
    (b"elseif", TokenKind::ElseIf),
    (b"enum", TokenKind::Enum),
    (b"extends", TokenKind::Extends),
    (b"false", TokenKind::False),
    (b"final", TokenKind::Final),
    (b"finally", TokenKind::Finally),
    (b"fn", TokenKind::Fn),
    (b"for", TokenKind::For),
    (b"foreach", TokenKind::Foreach),
    (b"function", TokenKind::Function),
    (b"goto", TokenKind::Goto),
    (b"if", TokenKind::If),
    (b"include", TokenKind::Include),
    (b"include_once", TokenKind::IncludeOnce),
    (b"implements", TokenKind::Implements),
    (b"interface", TokenKind::Interface),
    (b"instanceof", TokenKind::Instanceof),
    (b"namespace", TokenKind::Namespace),
    (b"new", TokenKind::New),
    (b"null", TokenKind::Null),
    (b"private", TokenKind::Private),
    (b"protected", TokenKind::Protected),
    (b"public", TokenKind::Public),
    (b"require", TokenKind::Require),
    (b"require_once", TokenKind::RequireOnce),
    (b"return", TokenKind::Return),
    (b"static", TokenKind::Static),
    (b"switch", TokenKind::Switch),
    (b"throw", TokenKind::Throw),
    (b"trait", TokenKind::Trait),
    (b"true", TokenKind::True),
    (b"try", TokenKind::Try),
    (b"use", TokenKind::Use),
    (b"var", TokenKind::Var),
    (b"yield", TokenKind::Yield),
    (b"__dir__", TokenKind::DirConstant),
    (b"__file__", TokenKind::FileConstant),
    (b"__line__", TokenKind::LineConstant),
    (b"__function__", TokenKind::FunctionConstant),
    (b"__class__", TokenKind::ClassConstant),
    (b"__method__", TokenKind::MethodConstant),
    (b"__trait__", TokenKind::TraitConstant),
    (b"__namespace__", TokenKind::NamespaceConstant),
    (
        b"__compiler_halt_offset__",
        TokenKind::CompilerHaltOffsetConstant,
    ),
    (b"while", TokenKind::While),
    (b"insteadof", TokenKind::Insteadof),
    (b"list", TokenKind::List),
    (b"self", TokenKind::Self_),
    (b"parent", TokenKind::Parent),
];

#[inline(always)]
fn identifier_to_keyword(ident: &[u8]) -> Option<TokenKind> {
    KEYWORDS.iter().find_map(|(keyword, kind)| {
        if ident.eq_ignore_ascii_case(keyword) {
            Some(*kind)
        } else {
            None
        }
    })
}

#[derive(Debug, Eq, PartialEq)]
enum NumberKind {
    Int,
    Float,
    IntOrFloat,
    OctalOrFloat,
}

#[cfg(test)]
mod tests {
    use super::Lexer;

    use pxp_token::{OpenTagKind, Token, TokenKind};

    #[test]
    fn it_can_tokenize_keywords() {
        use TokenKind::*;

        let tokens = tokenise("<?php die self parent from print readonly global abstract as break case catch class clone const continue declare default do echo else elseif empty enddeclare endfor endforeach endif endswitch endwhile enum extends false final finally fn for foreach function goto if implements include include_once instanceof insteadof eval exit unset isset list interface match namespace new null private protected public require require_once return static switch throw trait true try use var yield while and or xor").iter().map(|t| t.kind).collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                OpenTag(OpenTagKind::Full),
                Die,
                Self_,
                Parent,
                From,
                Print,
                Readonly,
                Global,
                Abstract,
                As,
                Break,
                Case,
                Catch,
                Class,
                Clone,
                Const,
                Continue,
                Declare,
                Default,
                Do,
                Echo,
                Else,
                ElseIf,
                Empty,
                EndDeclare,
                EndFor,
                EndForeach,
                EndIf,
                EndSwitch,
                EndWhile,
                Enum,
                Extends,
                False,
                Final,
                Finally,
                Fn,
                For,
                Foreach,
                Function,
                Goto,
                If,
                Implements,
                Include,
                IncludeOnce,
                Instanceof,
                Insteadof,
                Eval,
                Exit,
                Unset,
                Isset,
                List,
                Interface,
                Match,
                Namespace,
                New,
                Null,
                Private,
                Protected,
                Public,
                Require,
                RequireOnce,
                Return,
                Static,
                Switch,
                Throw,
                Trait,
                True,
                Try,
                Use,
                Var,
                Yield,
                While,
                LogicalAnd,
                LogicalOr,
                LogicalXor,
                Eof,
            ]
        );
    }

    #[test]
    fn it_can_tokenize_casts() {
        use TokenKind::*;

        let tokens = tokenise("<?php (int) (integer) (bool) (boolean) (float) (double) (real) (string) (array) (object) (unset)").iter().map(|t| t.kind).collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                OpenTag(OpenTagKind::Full),
                IntCast,
                IntegerCast,
                BoolCast,
                BooleanCast,
                FloatCast,
                DoubleCast,
                RealCast,
                StringCast,
                ArrayCast,
                ObjectCast,
                UnsetCast,
                Eof,
            ]
        );
    }

    #[test]
    fn it_can_tokenize_casts_with_excess_whitespace() {
        use TokenKind::*;

        let tokens = tokenise("<?php (int    ) (integer  ) (bool  ) (boolean) (float ) (double   ) (real    ) (string ) (array   ) (object   ) (  unset  )").iter().map(|t| t.kind).collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                OpenTag(OpenTagKind::Full),
                IntCast,
                IntegerCast,
                BoolCast,
                BooleanCast,
                FloatCast,
                DoubleCast,
                RealCast,
                StringCast,
                ArrayCast,
                ObjectCast,
                UnsetCast,
                Eof,
            ]
        );
    }

    #[test]
    fn it_can_tokenize_operators() {
        use TokenKind::*;

        let tokens = tokenise("<?php + - * / % ** = += -= *= /= .= %= **= &= |= ^= <<= >>= <=> == === != <> !== > < >= <= <=> ?? ! && || ??= and or xor . -> :: ++ -- ?? ! and or xor").iter().map(|t| t.kind).collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                OpenTag(OpenTagKind::Full),
                Plus,
                Minus,
                Asterisk,
                Slash,
                Percent,
                Pow,
                Equals,
                PlusEquals,
                MinusEquals,
                AsteriskEquals,
                SlashEquals,
                DotEquals,
                PercentEquals,
                PowEquals,
                AmpersandEquals,
                PipeEquals,
                CaretEquals,
                LeftShiftEquals,
                RightShiftEquals,
                Spaceship,
                DoubleEquals,
                TripleEquals,
                BangEquals,
                AngledLeftRight,
                BangDoubleEquals,
                GreaterThan,
                LessThan,
                GreaterThanEquals,
                LessThanEquals,
                Spaceship,
                DoubleQuestion,
                Bang,
                BooleanAnd,
                BooleanOr,
                DoubleQuestionEquals,
                LogicalAnd,
                LogicalOr,
                LogicalXor,
                Dot,
                Arrow,
                DoubleColon,
                Increment,
                Decrement,
                DoubleQuestion,
                Bang,
                LogicalAnd,
                LogicalOr,
                LogicalXor,
                Eof,
            ]
        );
    }

    #[test]
    fn it_can_tokenize_single_quoted_strings() {
        let tokens = tokenise("<?php 'foo' 'foo\\'bar'")
            .iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                TokenKind::OpenTag(OpenTagKind::Full),
                TokenKind::LiteralSingleQuotedString,
                TokenKind::LiteralSingleQuotedString,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn it_can_tokenize_double_quoted_strings() {
        let tokens = tokenise("<?php \"foo\" \"foo\\\"bar\"")
            .iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                TokenKind::OpenTag(OpenTagKind::Full),
                TokenKind::LiteralDoubleQuotedString,
                TokenKind::LiteralDoubleQuotedString,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn it_can_tokenize_heredocs() {
        let tokens = tokenise("<?php <<<EOD\n    foo\n    EOD")
            .iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                TokenKind::OpenTag(OpenTagKind::Full),
                TokenKind::StartHeredoc,
                TokenKind::StringPart,
                TokenKind::EndHeredoc,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn it_can_tokenize_nowdocs() {
        let tokens = tokenise("<?php <<<'EOD'\n    foo\n    EOD")
            .iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                TokenKind::OpenTag(OpenTagKind::Full),
                TokenKind::StartNowdoc,
                TokenKind::StringPart,
                TokenKind::EndNowdoc,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn it_can_tokenize_integers() {
        let tokens = tokenise("<?php 100 0123 0o123 0x1A 0b11111111 1_234_567")
            .iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                TokenKind::OpenTag(OpenTagKind::Full),
                TokenKind::LiteralInteger,
                TokenKind::LiteralInteger,
                TokenKind::LiteralInteger,
                TokenKind::LiteralInteger,
                TokenKind::LiteralInteger,
                TokenKind::LiteralInteger,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn it_can_tokenize_floats() {
        let tokens = tokenise("<?php 1.234 1.2e3 7E-10 1_234.567")
            .iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                TokenKind::OpenTag(OpenTagKind::Full),
                TokenKind::LiteralFloat,
                TokenKind::LiteralFloat,
                TokenKind::LiteralFloat,
                TokenKind::LiteralFloat,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn it_can_tokenize_identifiers() {
        let tokens = tokenise("<?php hello \\hello hello\\world")
            .iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                TokenKind::OpenTag(OpenTagKind::Full),
                TokenKind::Identifier,
                TokenKind::FullyQualifiedIdentifier,
                TokenKind::QualifiedIdentifier,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn it_can_tokenize_heredocs_with_escapes() {
        let tokens = tokenise("<?php <<<EOD\n\\$foo\nEOD;")
            .iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            &tokens,
            &[
                TokenKind::OpenTag(OpenTagKind::Full),
                TokenKind::StartHeredoc,
                TokenKind::StringPart,
                TokenKind::EndHeredoc,
                TokenKind::SemiColon,
                TokenKind::Eof,
            ]
        );
    }

    fn tokenise(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);

        lexer.tokenize().unwrap()
    }
}
