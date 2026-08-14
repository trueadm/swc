use std::{fs::read_to_string, path::PathBuf};

use pretty_assertions::assert_eq;
use swc_common::FileName;
use swc_ecma_ast::Module;
use swc_ecma_parser::{lexer::Lexer, Parser, Syntax, TsSyntax};
use testing::StdErr;

#[testing::fixture("tests/tsrx/**/*.tsrx")]
fn ast_fixture(entry: PathBuf) {
    testing::run_test(false, |cm, handler| {
        let source = read_to_string(&entry).expect("failed to read TSRX fixture");
        let file = cm.new_source_file(FileName::Real(entry.clone()).into(), source);
        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsrx: true,
                ..Default::default()
            }),
            Default::default(),
            (&*file).into(),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .map_err(|error| error.into_diagnostic(handler).emit())?;

        for error in parser.take_errors() {
            error.into_diagnostic(handler).emit();
        }

        let json = serde_json::to_string_pretty(&module).expect("failed to serialize TSRX AST");
        if StdErr::from(json.clone())
            .compare_to_file(format!("{}.json", entry.display()))
            .is_err()
        {
            panic!("TSRX AST fixture changed")
        }

        let round_trip: Module = serde_json::from_str(&json)
            .unwrap_or_else(|error| panic!("failed to deserialize TSRX AST: {error}\n{json}"));
        assert_eq!(module, round_trip, "JSON round-trip changed the TSRX AST");

        Ok(())
    })
    .unwrap();
}
