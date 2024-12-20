pub mod check;
pub mod context;
pub mod parse;

use context::TyContext;

pub struct Checker<'tcx> {
	tcx: TyContext<'tcx>,
}

impl<'tcx> Checker<'tcx> {
	pub fn new(tcx: TyContext<'tcx>) -> Checker<'tcx> {
		Checker {
			tcx,
			function_scopes: RefCell::new(vec![]),
		}
	}
}
