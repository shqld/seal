use seal_ty::{checker::Checker, context::TyContext, parse::parse};

pub fn run(code: &'static str) -> Result<(), Vec<String>> {
	let result = parse(code).unwrap();

	let ast = result.program;
	let tcx = TyContext::new();
	let checker = Checker::new(&tcx);

	checker
		.check(&ast)
		.map_err(|errors| errors.into_iter().map(|e| format!("{}", e)).collect())
}
