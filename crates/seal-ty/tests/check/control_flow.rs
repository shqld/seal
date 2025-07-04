use super::{fail, pass};

pass!(
	narrow_union_with_multiple_branches,
	r#"
        let x: boolean | number | string = 42;

        if (typeof x === 'number') {
            x satisfies number;
        } else if (typeof x === 'string') {
            x satisfies string;
        } else {
            x satisfies boolean;
        }
    "#
);

pass!(
	narrow_union_through_nested_scope,
	r#"
        let x: number | string = 42;

        if (typeof x === 'number') {
            x satisfies number;

            if (true) {
                // TODO: inner scope should inherit the scoped types in the outer scope
                // x satisfies number;
            }

            x satisfies number;
        }
    "#
);

fail!(
	narrowed_union_outside_scope,
	r#"
        let x: number | string = 42;

        if (typeof x === 'number') {
            x satisfies number;
        }

        // x: number | string
        x satisfies number;
    "#,
	&["Type 'number | string' is not assignable to type 'number'."]
);

pass!(
	assign_to_narrowed_union,
	r#"
        let x: number | string = 42;

        if (typeof x === 'number') {
            x = 42;
            // TODO:
            // x = "hello";
        } else {
            // TODO:
            // x = 42;
            x = "hello";
        }
    "#
);

pass!(
	nested_if_statements,
	r#"
        let x: number | string | boolean = 42;
        
        if (typeof x === 'number') {
            if (x > 0) {
                x satisfies number;
            } else {
                x satisfies number;
            }
        } else if (typeof x === 'string') {
            x satisfies string;
        } else {
            x satisfies boolean;
        }
    "#
);

pass!(
	complex_boolean_expressions,
	r#"
        let a = true;
        let b = false;
        let c = true;
        
        if (a && b || c) {
            let result = "condition met";
        }
        
        if ((a || b) && c) {
            let result = "complex condition";
        }
    "#
);

pass!(
	type_narrowing_multiple_guards,
	r#"
        let value: number | string | boolean | null = "hello";
        
        if (typeof value === 'string') {
            value satisfies string;
        } else if (typeof value === 'number') {
            value satisfies number;
        } else if (typeof value === 'boolean') {
            value satisfies boolean;
        } else {
            // value should be null here
        }
    "#
);

pass!(
	type_narrowing_property_access,
	r#"
        let obj = { type: "user", name: "Alice" };
        
        if (obj.type === "user") {
            obj.name satisfies string;
        }
    "#
);

pass!(
	boolean_type_guards_complex,
	r#"
        let value: number | string = "test";
        
        if (typeof value === 'string') {
            value satisfies string;
        }
        
        if (typeof value === 'number') {
            value satisfies number;
        }
    "#
);

fail!(
	eq_operands_have_no_overlap,
	r#"
        let x: { n: number } = { n: 42 };
        x.n === "hello";
    "#,
	&[
		"This comparison appears to be unintentional because the types 'number' and '\"hello\"' have no overlap."
	]
);
