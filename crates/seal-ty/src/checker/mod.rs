pub mod check;
pub mod context;
pub mod parse;

use context::TyContext;
use swc_ecma_ast::Program;

pub struct Checker<'tcx> {
	tcx: TyContext<'tcx>,
}

impl<'tcx> Checker<'tcx> {
	pub fn new(ast: Program) -> Checker<'tcx> {
		let tcx = TyContext::new(ast);
		Checker { tcx }
	}
}
