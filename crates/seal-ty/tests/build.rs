use seal_ty::{
	checker::{Checker, parse::parse},
	context::TyContext,
};

fn run(code: &'static str) {
	let result = parse(code).unwrap();

	let ast = result.program;
	let tcx = TyContext::new();
	let checker = Checker::new(&tcx);

	checker.check(&ast).unwrap();
}

macro_rules! fail {
	($case_name:ident, $code:literal) => {
		#[should_panic]
		#[test]
		fn $case_name() {
			run($code);
		}
	};
}

fail!(
	unexpected_return_on_main_fn_1_,
	r#"
        return;
    "#
);

fail!(
	unexpected_return_on_main_fn_2_,
	r#"
        return 42;
    "#
);

fail!(
	no_function_param_type_ann_1_,
	r#"
        function f(x): number {
            return 42;
        }
    "#
);

fail!(
	no_function_param_type_ann_2_,
	r#"
        function f(x, y, z): number {
            return 42;
        }
    "#
);

fail!(
	assign_to_const_var_,
	r#"
        const x = 42;
        x = 43;
    "#
);

fail!(
	assign_to_params_,
	r#"
        function f(x: number) {
            x = 42;
        }
    "#
);
