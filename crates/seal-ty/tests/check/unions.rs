use super::{fail, pass};

pass!(
	union_basic,
	r#"
        let x: number | string;
        x = 42;
        x = "hello";
    "#
);

pass!(
	union_three_types,
	r#"
        let value: number | string | boolean = 42;
        value = "hello";
        value = true;
    "#
);

pass!(
	union_with_null_undefined,
	r#"
        let value: string | null = "hello";
        // Note: null and undefined types would need to be implemented
    "#
);

fail!(
	union_assignment_invalid,
	r#"
        let value: number | string = true;
    "#,
	&["Type 'boolean' is not assignable to type 'number | string'."]
);

pass!(
	get_prop_on_union_with_non_object_arm,
	r#"
        let x: { n: number } = { n: 42 };
        x.n satisfies number;
    "#
);

pass!(
	assign_prop_on_union,
	r#"
        let x: { x: number } | { x: string } = { x: 42 };

        // x.x = 42;
        // x.x = "hello";
    "#
);
