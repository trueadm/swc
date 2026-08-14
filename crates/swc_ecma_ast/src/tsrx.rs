use string_enum::StringEnum;
use swc_atoms::Atom;
use swc_common::{ast_node, EqIgnoreSpan, Span};

use crate::{
    BlockStmt, Expr, ForHead, Ident, JSXClosingElement, JSXOpeningElement, Pat, Stmt, VarDeclOrExpr,
};

/// An expression introduced by the TSRX grammar.
///
/// The inner nodes retain their own ESTree-compatible `type` tag when this
/// enum is embedded in [`Expr`](crate::Expr) or a JSX child.
#[ast_node]
#[derive(Eq, Hash, EqIgnoreSpan)]
#[allow(variant_size_differences)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
pub enum TsrxExpr {
    #[tag("JSXCodeBlock")]
    CodeBlock(JSXCodeBlock),

    #[tag("JSXStyleElement")]
    StyleElement(Box<JSXStyleElement>),

    #[tag("JSXIfExpression")]
    If(Box<JSXIfExpr>),

    #[tag("JSXForExpression")]
    For(Box<JSXForExpr>),

    #[tag("JSXSwitchExpression")]
    Switch(Box<JSXSwitchExpr>),

    #[tag("JSXTryExpression")]
    Try(Box<JSXTryExpr>),
}

/// A TSRX statement container (`@{ ... }`).
///
/// Setup statements are stored in `body`; the final template expression is
/// stored separately in `render` so consumers do not need to infer which
/// expression statement is the rendered value.
#[ast_node("JSXCodeBlock")]
#[derive(Eq, Hash, EqIgnoreSpan)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
pub struct JSXCodeBlock {
    pub span: Span,

    #[cfg_attr(feature = "serde-impl", serde(default))]
    pub body: Vec<Stmt>,

    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub render: Option<Box<Expr>>,

    /// True when this code block was written directly as a function body.
    #[cfg_attr(feature = "serde-impl", serde(default, rename = "isFunctionBody"))]
    pub is_function_body: bool,
}

/// A `<style>` element whose body is captured as raw CSS source.
///
/// CSS parsing belongs to a later transform; retaining the source here keeps
/// the ECMAScript AST crate independent from `swc_css_ast`.
#[ast_node("JSXStyleElement")]
#[derive(Eq, Hash, EqIgnoreSpan)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
pub struct JSXStyleElement {
    pub span: Span,
    pub opening: JSXOpeningElement,
    pub css: Atom,
    pub closing: JSXClosingElement,
}

/// A TSRX `@if` expression.
#[ast_node("JSXIfExpression")]
#[derive(Eq, Hash, EqIgnoreSpan)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
pub struct JSXIfExpr {
    pub span: Span,
    pub test: Box<Expr>,
    pub consequent: JSXCodeBlock,

    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub alternate: Option<JSXIfAlternate>,
}

#[ast_node]
#[derive(Eq, Hash, EqIgnoreSpan)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
pub enum JSXIfAlternate {
    #[tag("JSXCodeBlock")]
    CodeBlock(JSXCodeBlock),
    #[tag("JSXIfExpression")]
    If(Box<JSXIfExpr>),
}

/// The JavaScript loop form used by a TSRX `@for` expression.
#[derive(StringEnum, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EqIgnoreSpan, Default)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
#[cfg_attr(
    feature = "encoding-impl",
    derive(::swc_common::Encode, ::swc_common::Decode)
)]
pub enum JSXForKind {
    /// `ForStatement`
    #[default]
    ForStatement,
    /// `ForInStatement`
    ForInStatement,
    /// `ForOfStatement`
    ForOfStatement,
}

/// A TSRX `@for` expression.
///
/// The fields mirror JavaScript's three loop forms. Only fields belonging to
/// `kind` are populated.
#[ast_node("JSXForExpression")]
#[derive(Eq, Hash, EqIgnoreSpan)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
pub struct JSXForExpr {
    pub span: Span,
    pub kind: JSXForKind,

    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub init: Option<VarDeclOrExpr>,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub test: Option<Box<Expr>>,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub update: Option<Box<Expr>>,

    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub left: Option<ForHead>,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub right: Option<Box<Expr>>,
    pub is_await: bool,

    pub body: JSXCodeBlock,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub index: Option<Ident>,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub key: Option<Box<Expr>>,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub empty: Option<JSXCodeBlock>,
}

/// A case in a TSRX `@switch` expression.
#[ast_node("JSXSwitchCase")]
#[derive(Eq, Hash, EqIgnoreSpan)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
pub struct JSXSwitchCase {
    pub span: Span,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub test: Option<Box<Expr>>,
    pub consequent: JSXCodeBlock,
}

/// A TSRX `@switch` expression.
#[ast_node("JSXSwitchExpression")]
#[derive(Eq, Hash, EqIgnoreSpan)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
pub struct JSXSwitchExpr {
    pub span: Span,
    pub discriminant: Box<Expr>,
    pub cases: Vec<JSXSwitchCase>,
}

/// The catch clause of a TSRX `@try` expression.
#[ast_node("JSXCatchClause")]
#[derive(Eq, Hash, EqIgnoreSpan)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
pub struct JSXCatchClause {
    pub span: Span,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub param: Option<Pat>,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub reset: Option<Ident>,
    pub body: JSXCodeBlock,
}

/// A TSRX `@try` expression.
#[ast_node("JSXTryExpression")]
#[derive(Eq, Hash, EqIgnoreSpan)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "shrink-to-fit", derive(shrink_to_fit::ShrinkToFit))]
pub struct JSXTryExpr {
    pub span: Span,
    pub block: JSXCodeBlock,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub pending: Option<JSXCodeBlock>,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub handler: Option<JSXCatchClause>,
    #[cfg_attr(
        feature = "encoding-impl",
        encoding(with = "cbor4ii::core::types::Maybe")
    )]
    pub finalizer: Option<BlockStmt>,
}
