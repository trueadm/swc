use swc_ecma_ast::*;
use swc_ecma_codegen_macros::node_impl;

#[cfg(swc_ast_unknown)]
use crate::unknown_error;

#[node_impl]
impl MacroNode for TsrxExpr {
    fn emit(&mut self, emitter: &mut Macro) -> Result {
        match self {
            TsrxExpr::CodeBlock(block) => {
                punct!(emitter, "@");
                emit!(block);
            }
            TsrxExpr::StyleElement(style) => emit!(style),
            TsrxExpr::If(if_expr) => emit!(if_expr),
            TsrxExpr::For(for_expr) => emit!(for_expr),
            TsrxExpr::Switch(switch_expr) => emit!(switch_expr),
            TsrxExpr::Try(try_expr) => emit!(try_expr),
            #[cfg(swc_ast_unknown)]
            _ => return Err(unknown_error()),
        }
        Ok(())
    }
}

#[node_impl]
impl MacroNode for JSXCodeBlock {
    fn emit(&mut self, emitter: &mut Macro) -> Result {
        punct!(emitter, "{");
        if !self.body.is_empty() {
            emitter.emit_list(
                self.span,
                Some(&self.body),
                ListFormat::MultiLineBlockStatements,
            )?;
        }
        if let Some(render) = &self.render {
            emitter.wr.commit_pending_semi()?;
            emit!(render);
        }
        punct!(emitter, "}");
        Ok(())
    }
}

#[node_impl]
impl MacroNode for JSXStyleElement {
    fn emit(&mut self, emitter: &mut Macro) -> Result {
        emit!(self.opening);
        emitter.wr.write_str(&self.css)?;
        emit!(self.closing);
        Ok(())
    }
}

#[node_impl]
impl MacroNode for JSXIfExpr {
    fn emit(&mut self, emitter: &mut Macro) -> Result {
        punct!(emitter, "@");
        keyword!(emitter, "if");
        formatting_space!(emitter);
        punct!(emitter, "(");
        emit!(self.test);
        punct!(emitter, ")");
        formatting_space!(emitter);
        emit!(self.consequent);
        if let Some(alternate) = &self.alternate {
            formatting_space!(emitter);
            punct!(emitter, "@");
            keyword!(emitter, "else");
            formatting_space!(emitter);
            emit!(alternate);
        }
        Ok(())
    }
}

#[node_impl]
impl MacroNode for JSXIfAlternate {
    fn emit(&mut self, emitter: &mut Macro) -> Result {
        match self {
            JSXIfAlternate::CodeBlock(block) => emit!(block),
            JSXIfAlternate::If(if_expr) => {
                keyword!(emitter, "if");
                formatting_space!(emitter);
                punct!(emitter, "(");
                emit!(if_expr.test);
                punct!(emitter, ")");
                formatting_space!(emitter);
                emit!(if_expr.consequent);
                if let Some(alternate) = &if_expr.alternate {
                    formatting_space!(emitter);
                    punct!(emitter, "@");
                    keyword!(emitter, "else");
                    formatting_space!(emitter);
                    emit!(alternate);
                }
            }
            #[cfg(swc_ast_unknown)]
            _ => return Err(unknown_error()),
        }
        Ok(())
    }
}

#[node_impl]
impl MacroNode for JSXForExpr {
    fn emit(&mut self, emitter: &mut Macro) -> Result {
        punct!(emitter, "@");
        keyword!(emitter, "for");
        if self.is_await {
            space!(emitter);
            keyword!(emitter, "await");
        }
        formatting_space!(emitter);
        punct!(emitter, "(");
        match self.kind {
            JSXForKind::ForStatement => {
                opt!(emitter, self.init);
                punct!(emitter, ";");
                opt_leading_space!(emitter, self.test);
                punct!(emitter, ";");
                opt_leading_space!(emitter, self.update);
            }
            JSXForKind::ForInStatement | JSXForKind::ForOfStatement => {
                opt!(emitter, self.left);
                space!(emitter);
                if self.kind == JSXForKind::ForInStatement {
                    keyword!(emitter, "in");
                } else {
                    keyword!(emitter, "of");
                }
                space!(emitter);
                opt!(emitter, self.right);
            }
        }
        if let Some(index) = &self.index {
            punct!(emitter, ";");
            space!(emitter);
            keyword!(emitter, "index");
            space!(emitter);
            emit!(index);
        }
        if let Some(key) = &self.key {
            punct!(emitter, ";");
            space!(emitter);
            keyword!(emitter, "key");
            space!(emitter);
            emit!(key);
        }
        punct!(emitter, ")");
        formatting_space!(emitter);
        emit!(self.body);
        if let Some(empty) = &self.empty {
            formatting_space!(emitter);
            punct!(emitter, "@");
            keyword!(emitter, "empty");
            formatting_space!(emitter);
            emit!(empty);
        }
        Ok(())
    }
}

#[node_impl]
impl MacroNode for JSXSwitchExpr {
    fn emit(&mut self, emitter: &mut Macro) -> Result {
        punct!(emitter, "@");
        keyword!(emitter, "switch");
        formatting_space!(emitter);
        punct!(emitter, "(");
        emit!(self.discriminant);
        punct!(emitter, ")");
        formatting_space!(emitter);
        punct!(emitter, "{");
        for case in &self.cases {
            emit!(case);
        }
        punct!(emitter, "}");
        Ok(())
    }
}

#[node_impl]
impl MacroNode for JSXSwitchCase {
    fn emit(&mut self, emitter: &mut Macro) -> Result {
        punct!(emitter, "@");
        if let Some(test) = &self.test {
            keyword!(emitter, "case");
            space!(emitter);
            emit!(test);
        } else {
            keyword!(emitter, "default");
        }
        punct!(emitter, ":");
        formatting_space!(emitter);
        emit!(self.consequent);
        Ok(())
    }
}

#[node_impl]
impl MacroNode for JSXTryExpr {
    fn emit(&mut self, emitter: &mut Macro) -> Result {
        punct!(emitter, "@");
        keyword!(emitter, "try");
        formatting_space!(emitter);
        emit!(self.block);
        if let Some(pending) = &self.pending {
            formatting_space!(emitter);
            punct!(emitter, "@");
            keyword!(emitter, "pending");
            formatting_space!(emitter);
            emit!(pending);
        }
        if let Some(handler) = &self.handler {
            formatting_space!(emitter);
            emit!(handler);
        }
        Ok(())
    }
}

#[node_impl]
impl MacroNode for JSXCatchClause {
    fn emit(&mut self, emitter: &mut Macro) -> Result {
        punct!(emitter, "@");
        keyword!(emitter, "catch");
        formatting_space!(emitter);
        punct!(emitter, "(");
        opt!(emitter, self.param);
        if let Some(reset) = &self.reset {
            punct!(emitter, ",");
            formatting_space!(emitter);
            emit!(reset);
        }
        punct!(emitter, ")");
        formatting_space!(emitter);
        emit!(self.body);
        Ok(())
    }
}
