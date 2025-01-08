use swc_common::{FileName, GLOBALS, Mark, SourceMap, input::SourceFileInput};
use swc_ecma_ast::Program;
use swc_ecma_parser::{Parser, TsSyntax, error::Error};
use swc_ecma_visit::VisitMut;

pub struct ParseResult {
	pub program: Program,
	pub source_map: SourceMap,
}

pub fn parse(code: &str) -> Result<ParseResult, Error> {
	let syntax = swc_ecma_parser::Syntax::Typescript(TsSyntax {
		..Default::default()
	});
	let source_map = SourceMap::new(Default::default());
	let source_file = source_map.new_source_file(
		FileName::Custom("example.ts".to_owned()).into(),
		code.into(),
	);

	let input = SourceFileInput::from(&*source_file);

	let mut parser = Parser::new(syntax, input, None);

	let mut program = parser.parse_program()?;

	GLOBALS.set(&Default::default(), || {
		swc_ecma_transforms_base::resolver(Mark::new(), Mark::new(), true)
			.visit_mut_program(&mut program);
	});

	Ok(ParseResult {
		program,
		source_map,
	})
}

#[cfg(test)]
mod tests {
	use swc_common::SyntaxContext;
	use swc_ecma_ast::{Expr, Id};
	use swc_ecma_visit::Visit;

	use super::parse;

	#[test]
	fn parse_typescript() {
		let code = r#"
            const n: number = 1;
        "#;

		parse(code).unwrap();
	}

	#[test]
	fn parse_es2022() {
		let code = r#"
            await Promise.resolve(1)
        "#;

		parse(code).unwrap();
	}

	#[test]
	fn parse_scope() {
		let code = r#"
			let a = 1; // a#2
			let b = 1; // b#2

			a; // a#2
			b; // b#2

			{
				let a = 2; // a#3

				a; // a#3
				b; // b#2
			}
        "#;

		let program = parse(code).unwrap().program;

		struct Assertion {
			expected_var_declarators: Vec<Id>,
			expected_idents: Vec<Id>,
		}

		impl Visit for Assertion {
			fn visit_var_declarator(&mut self, n: &swc_ecma_ast::VarDeclarator) {
				assert_eq!(
					n.name.clone().expect_ident().id.to_id(),
					self.expected_var_declarators.pop().unwrap()
				);
			}

			fn visit_expr_stmt(&mut self, n: &swc_ecma_ast::ExprStmt) {
				if let Expr::Ident(ident) = &*n.expr {
					assert_eq!(ident.to_id(), self.expected_idents.pop().unwrap());
				}
			}
		}

		let mut assertion = Assertion {
			expected_var_declarators: vec![
				("a".into(), SyntaxContext::from_u32(2)),
				("b".into(), SyntaxContext::from_u32(2)),
				("a".into(), SyntaxContext::from_u32(3)),
			]
			.into_iter()
			.rev()
			.collect(),
			expected_idents: vec![
				("a".into(), SyntaxContext::from_u32(2)),
				("b".into(), SyntaxContext::from_u32(2)),
				("a".into(), SyntaxContext::from_u32(3)),
				("b".into(), SyntaxContext::from_u32(2)),
			]
			.into_iter()
			.rev()
			.collect(),
		};

		assertion.visit_program(&program);
	}
}
