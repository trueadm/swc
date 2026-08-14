use swc_common::{BytePos, Span, Spanned};
use swc_ecma_ast::*;

use super::{input::Tokens, stmt::TempForHead, Parser};
use crate::{lexer::Token, Context, PResult};

impl<I: Tokens> Parser<I> {
    /// Returns whether the current `@` begins a TSRX expression rather than a
    /// legacy decorator.
    pub(super) fn is_tsrx_expr_start(&mut self) -> bool {
        self.input().syntax().tsrx()
            && self.input().is(Token::At)
            && matches!(
                peek!(self),
                Some(Token::LBrace | Token::If | Token::For | Token::Switch | Token::Try)
            )
    }

    /// Parses a TSRX expression beginning at `@`.
    pub(super) fn parse_tsrx_expr(&mut self) -> PResult<Box<Expr>> {
        let start = self.cur_pos();
        self.assert_and_bump(Token::At);

        let expr = match self.input().cur() {
            Token::LBrace => TsrxExpr::CodeBlock(self.parse_tsrx_code_block(false)?),
            Token::If => TsrxExpr::If(Box::new(self.parse_tsrx_if(start)?)),
            Token::For => TsrxExpr::For(Box::new(self.parse_tsrx_for(start)?)),
            Token::Switch => TsrxExpr::Switch(Box::new(self.parse_tsrx_switch(start)?)),
            Token::Try => TsrxExpr::Try(Box::new(self.parse_tsrx_try(start)?)),
            _ => unexpected!(self, "a TSRX statement container or template directive"),
        };

        Ok(Box::new(Expr::Tsrx(Box::new(expr))))
    }

    /// Parse the braced portion of a statement container or directive.
    fn parse_tsrx_code_block(&mut self, is_function_body: bool) -> PResult<JSXCodeBlock> {
        let block = self.parse_block(false)?;
        Ok(Self::tsrx_code_block_from_block(block, is_function_body))
    }

    fn tsrx_code_block_from_block(mut block: BlockStmt, is_function_body: bool) -> JSXCodeBlock {
        let render = match block.stmts.last() {
            Some(Stmt::Expr(ExprStmt { expr, .. })) if Self::is_tsrx_render_expr(expr) => {
                let Stmt::Expr(stmt) = block.stmts.pop().expect("last statement was checked")
                else {
                    unreachable!()
                };
                Some(stmt.expr)
            }
            _ => None,
        };

        JSXCodeBlock {
            span: block.span,
            body: block.stmts,
            render,
            is_function_body,
        }
    }

    fn is_tsrx_render_expr(expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::JSXElement(_) | Expr::JSXFragment(_) | Expr::Tsrx(_)
        )
    }

    fn parse_tsrx_condition(&mut self) -> PResult<Box<Expr>> {
        expect!(self, Token::LParen);
        let test = self.parse_expr()?;
        expect!(self, Token::RParen);
        Ok(test)
    }

    fn parse_tsrx_if(&mut self, start: BytePos) -> PResult<JSXIfExpr> {
        self.assert_and_bump(Token::If);
        let test = self.parse_tsrx_condition()?;
        let consequent = self.parse_tsrx_code_block(false)?;

        let alternate = if self.input().is(Token::At)
            && peek!(self).is_some_and(|token| token == Token::Else)
        {
            self.assert_and_bump(Token::At);
            self.assert_and_bump(Token::Else);
            if self.input().is(Token::If) {
                let nested_start = self.cur_pos();
                Some(JSXIfAlternate::If(Box::new(
                    self.parse_tsrx_if(nested_start)?,
                )))
            } else {
                Some(JSXIfAlternate::CodeBlock(
                    self.parse_tsrx_code_block(false)?,
                ))
            }
        } else {
            None
        };

        Ok(JSXIfExpr {
            span: self.span(start),
            test,
            consequent,
            alternate,
        })
    }

    fn parse_tsrx_for(&mut self, start: BytePos) -> PResult<JSXForExpr> {
        self.assert_and_bump(Token::For);
        let is_await = self.input_mut().eat(Token::Await);
        expect!(self, Token::LParen);

        let head = self.do_inside_of_context(Context::ForLoopInit, |p| {
            if is_await {
                p.do_inside_of_context(Context::ForAwaitLoopInit, Self::parse_for_head)
            } else {
                p.do_outside_of_context(Context::ForAwaitLoopInit, Self::parse_for_head)
            }
        })?;

        let mut index = None;
        let mut key = None;
        while self.input_mut().eat(Token::Semi) {
            if self.is_contextual_word("index") {
                self.bump();
                index = Some(self.parse_binding_ident(false)?.id);
            } else if self.is_contextual_word("key") {
                self.bump();
                key = Some(self.allow_in_expr(Self::parse_assignment_expr)?);
            } else {
                unexpected!(self, "`index` or `key` in a TSRX @for header")
            }
        }
        expect!(self, Token::RParen);

        let body = self.parse_tsrx_code_block(false)?;
        let empty = self.parse_tsrx_named_block("empty")?;

        let (kind, init, test, update, left, right) = match head {
            TempForHead::For { init, test, update } => {
                (JSXForKind::ForStatement, init, test, update, None, None)
            }
            TempForHead::ForIn { left, right } => (
                JSXForKind::ForInStatement,
                None,
                None,
                None,
                Some(left),
                Some(right),
            ),
            TempForHead::ForOf { left, right } => (
                JSXForKind::ForOfStatement,
                None,
                None,
                None,
                Some(left),
                Some(right),
            ),
        };

        Ok(JSXForExpr {
            span: self.span(start),
            kind,
            init,
            test,
            update,
            left,
            right,
            is_await,
            body,
            index,
            key,
            empty,
        })
    }

    fn parse_tsrx_switch(&mut self, start: BytePos) -> PResult<JSXSwitchExpr> {
        self.assert_and_bump(Token::Switch);
        let discriminant = self.parse_tsrx_condition()?;
        expect!(self, Token::LBrace);

        let mut cases = Vec::new();
        while !self.input().is(Token::RBrace) {
            let case_start = self.cur_pos();
            expect!(self, Token::At);
            let test = if self.input_mut().eat(Token::Case) {
                Some(self.parse_expr()?)
            } else if self.input_mut().eat(Token::Default) {
                None
            } else {
                unexpected!(self, "`@case` or `@default` in a TSRX @switch")
            };
            expect!(self, Token::Colon);
            let consequent = self.parse_tsrx_code_block(false)?;
            cases.push(JSXSwitchCase {
                span: self.span(case_start),
                test,
                consequent,
            });
        }
        expect!(self, Token::RBrace);

        Ok(JSXSwitchExpr {
            span: self.span(start),
            discriminant,
            cases,
        })
    }

    fn parse_tsrx_try(&mut self, start: BytePos) -> PResult<JSXTryExpr> {
        self.assert_and_bump(Token::Try);
        let block = self.parse_tsrx_code_block(false)?;
        let pending = self.parse_tsrx_named_block("pending")?;

        let handler = if self.input().is(Token::At)
            && peek!(self).is_some_and(|token| token == Token::Catch)
        {
            let catch_start = self.cur_pos();
            self.assert_and_bump(Token::At);
            self.assert_and_bump(Token::Catch);
            expect!(self, Token::LParen);
            let param = if self.input().is(Token::RParen) {
                None
            } else {
                Some(Pat::Ident(self.parse_binding_ident(false)?))
            };
            let reset = if self.input_mut().eat(Token::Comma) {
                Some(self.parse_binding_ident(false)?.id)
            } else {
                None
            };
            expect!(self, Token::RParen);
            let body = self.parse_tsrx_code_block(false)?;
            Some(JSXCatchClause {
                span: self.span(catch_start),
                param,
                reset,
                body,
            })
        } else {
            None
        };

        Ok(JSXTryExpr {
            span: self.span(start),
            block,
            pending,
            handler,
            finalizer: None,
        })
    }

    fn parse_tsrx_named_block(&mut self, name: &str) -> PResult<Option<JSXCodeBlock>> {
        if !self.input().is(Token::At) {
            return Ok(None);
        }

        let next_token = self.input_mut().peek();
        let is_match = next_token.is_some_and(Token::is_word)
            && self.input().next().is_some_and(|next| match &next.value {
                Some(crate::lexer::TokenValue::Word(word)) => word == name,
                None => self.input().iter.read_string(next.span()) == name,
                _ => false,
            });
        if !is_match {
            return Ok(None);
        }
        self.assert_and_bump(Token::At);
        self.bump();
        self.parse_tsrx_code_block(false).map(Some)
    }

    fn is_contextual_word(&self, expected: &str) -> bool {
        self.input().cur().is_word() && self.input().cur().take_word(&self.input) == expected
    }

    /// Wrap a direct `function f() @{ ... }` body in the existing public
    /// `FunctionBody` shape without changing every SWC consumer at once.
    pub(super) fn parse_tsrx_function_body(&mut self) -> PResult<FunctionBody> {
        let start = self.cur_pos();
        let mut expr = self.parse_tsrx_expr()?;
        let span = Span::new_with_checked(start, expr.span_hi());

        if let Expr::Tsrx(tsrx) = &mut *expr {
            if let TsrxExpr::CodeBlock(block) = &mut **tsrx {
                block.is_function_body = true;
            }
        }

        Ok(FunctionBody {
            span,
            stmts: vec![Stmt::Expr(ExprStmt { span, expr })],
        })
    }
}

#[cfg(test)]
mod tests {
    use swc_ecma_ast::*;

    use super::super::test_parser;
    use crate::{Syntax, TsSyntax};

    fn syntax() -> Syntax {
        Syntax::Typescript(TsSyntax {
            tsrx: true,
            ..Default::default()
        })
    }

    fn expr(source: &'static str) -> Box<Expr> {
        test_parser(source, syntax(), |parser| parser.parse_expr())
    }

    #[test]
    fn parses_statement_container_ast() {
        let parsed = expr("@{ const greeting = 'hello'; <h1>{greeting}</h1> }");
        let Expr::Tsrx(tsrx) = *parsed else {
            panic!("expected a TSRX expression")
        };
        let TsrxExpr::CodeBlock(block) = *tsrx else {
            panic!("expected a JSXCodeBlock")
        };

        assert_eq!(block.body.len(), 1);
        assert!(matches!(block.body[0], Stmt::Decl(Decl::Var(_))));
        assert!(matches!(block.render.as_deref(), Some(Expr::JSXElement(_))));
        assert!(!block.is_function_body);
    }

    #[test]
    fn marks_direct_function_body() {
        let module = test_parser("function View() @{ <main /> }", syntax(), |parser| {
            parser.parse_module()
        });
        let ModuleItem::Stmt(Stmt::Decl(Decl::Fn(function))) = &module.body[0] else {
            panic!("expected a function declaration")
        };
        let body = function.function.body.as_ref().expect("function body");
        let Stmt::Expr(ExprStmt { expr, .. }) = &body.stmts[0] else {
            panic!("expected the TSRX body wrapper")
        };
        let Expr::Tsrx(tsrx) = &**expr else {
            panic!("expected a TSRX expression")
        };
        let TsrxExpr::CodeBlock(block) = &**tsrx else {
            panic!("expected a JSXCodeBlock")
        };

        assert!(block.is_function_body);
        assert!(matches!(block.render.as_deref(), Some(Expr::JSXElement(_))));
    }

    #[test]
    fn parses_if_expression_ast() {
        let parsed = expr(
            "@if (visible) { <Shown /> } @else if (pending) { <Loading /> } @else { <Hidden /> }",
        );
        let Expr::Tsrx(tsrx) = *parsed else {
            panic!("expected a TSRX expression")
        };
        let TsrxExpr::If(if_expr) = *tsrx else {
            panic!("expected a JSXIfExpression")
        };

        assert!(matches!(*if_expr.test, Expr::Ident(_)));
        assert!(matches!(
            if_expr.consequent.render.as_deref(),
            Some(Expr::JSXElement(_))
        ));
        assert!(matches!(if_expr.alternate, Some(JSXIfAlternate::If(_))));
    }

    #[test]
    fn parses_for_options_and_empty_block() {
        let parsed = expr(
            "@for (const item of items; index index; key item.id) { <Row item={item} /> } @empty \
             { <Empty /> }",
        );
        let Expr::Tsrx(tsrx) = *parsed else {
            panic!("expected a TSRX expression")
        };
        let TsrxExpr::For(for_expr) = *tsrx else {
            panic!("expected a JSXForExpression")
        };

        assert_eq!(for_expr.kind, JSXForKind::ForOfStatement);
        assert_eq!(
            for_expr.index.as_ref().map(|ident| ident.sym.as_ref()),
            Some("index")
        );
        assert!(for_expr.key.is_some());
        assert!(for_expr.empty.is_some());
    }

    #[test]
    fn parses_switch_and_try_expressions() {
        let switch =
            expr("@switch (state) { @case 'ready': { <Ready /> } @default: { <Idle /> } }");
        let Expr::Tsrx(switch) = *switch else {
            panic!("expected a TSRX expression")
        };
        let TsrxExpr::Switch(switch) = *switch else {
            panic!("expected a JSXSwitchExpression")
        };
        assert_eq!(switch.cases.len(), 2);
        assert!(switch.cases[0].test.is_some());
        assert!(switch.cases[1].test.is_none());

        let try_expr = expr(
            "@try { <Content /> } @pending { <Loading /> } @catch (error, reset) { <ErrorView /> }",
        );
        let Expr::Tsrx(try_expr) = *try_expr else {
            panic!("expected a TSRX expression")
        };
        let TsrxExpr::Try(try_expr) = *try_expr else {
            panic!("expected a JSXTryExpression")
        };
        assert!(try_expr.pending.is_some());
        assert_eq!(
            try_expr
                .handler
                .as_ref()
                .and_then(|handler| handler.reset.as_ref())
                .map(|ident| ident.sym.as_ref()),
            Some("reset")
        );
    }

    #[test]
    fn parses_directives_as_jsx_children() {
        let parsed = expr(
            "<main>before @if (visible) { <Shown /> } after @for (const item of items) { <Row /> \
             }</main>",
        );
        let Expr::JSXElement(element) = *parsed else {
            panic!("expected a JSX element")
        };

        assert_eq!(element.children.len(), 4);
        assert!(matches!(element.children[0], JSXElementChild::JSXText(_)));
        assert!(matches!(
            element.children[1],
            JSXElementChild::Tsrx(ref expr) if matches!(**expr, TsrxExpr::If(_))
        ));
        assert!(matches!(element.children[2], JSXElementChild::JSXText(_)));
        assert!(matches!(
            element.children[3],
            JSXElementChild::Tsrx(ref expr) if matches!(**expr, TsrxExpr::For(_))
        ));
    }

    #[test]
    fn parses_dynamic_tags_and_shorthand_attributes() {
        let parsed = expr("<{Tag} {value}></{Tag}>");
        let Expr::JSXElement(element) = *parsed else {
            panic!("expected a JSX element")
        };

        assert!(matches!(
            element.opening.name,
            JSXElementName::JSXExprContainer(_)
        ));
        assert!(matches!(
            element.opening.attrs[0],
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                shorthand: true,
                ..
            })
        ));
        assert!(matches!(
            element.closing.as_ref().map(|closing| &closing.name),
            Some(JSXElementName::JSXExprContainer(_))
        ));
    }

    #[test]
    fn tsx_mode_does_not_enable_tsrx() {
        let syntax = Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: true,
            ..Default::default()
        });
        test_parser("@sealed class Example {}", syntax, |parser| {
            parser.parse_module()
        });
    }
}
