use super::{fail, pass};

pass!(
	let_declaration,
	r#"
        let a = 1;
        a satisfies number;
    "#
);

pass!(
	assign_to_uninitialized_var,
	r#"
        let a;
        a = 1;
        a satisfies number;
    "#
);

fail!(
	assign_type_mismatched_value_to_initialized_var,
	r#"
        let a = 1;
        a = "hello";
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

fail!(
	assign_to_initialized_var_that_was_originally_uninitialized,
	r#"
        let a;
        a = 1;
        a = "hello";
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

pass!(
	const_binding,
	r#"
        const x = "42";
        x satisfies "42";
        x satisfies string;
    "#
);

fail!(
	let_binding_literal_type,
	r#"
        let x = "42";
        x satisfies "42";
    "#,
	&["Type 'string' is not assignable to type '\"42\"'."]
);

pass!(
	assign_to_annotated_var,
	r#"
        let n: number;
        n = 1;

        n satisfies number;
    "#
);

fail!(
	assign_to_annotated_var_type_mismatch,
	r#"
        let n: number;
        n = "hello";
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);
