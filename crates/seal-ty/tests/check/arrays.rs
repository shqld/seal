use super::{fail, pass};

pass!(
	array_literal_empty,
	r#"
        let arr = [];
        arr satisfies unknown[]; // NOTE: unknown instead of any or never
    "#
);

pass!(
	array_literal_homogeneous,
	r#"
        let arr = [1, 2, 3];
        arr satisfies number[];
    "#
);

pass!(
	array_literal_heterogeneous,
	r#"
        let arr = [1, "hello", true];
        arr satisfies (number | string | boolean)[];
    "#
);

pass!(
	array_literal_nested,
	r#"
        let arr = [[1, 2], [3, 4]];
        arr satisfies number[][];
    "#
);

pass!(
	array_type_annotation,
	r#"
        let arr: number[] = [1, 2, 3];
        arr satisfies number[];
    "#
);

pass!(
	array_empty_with_annotation,
	r#"
        let arr: string[] = [];
        arr satisfies string[];
    "#
);

fail!(
	array_type_mismatch,
	r#"
        let arr: number[] = ["hello", "world"];
    "#,
	&["Type 'string[]' is not assignable to type 'number[]'."]
);

pass!(
	array_mixed_compatible_union,
	r#"
        let arr: (number | string)[] = [1, "hello", 2, "world"];
        arr satisfies (number | string)[];
    "#
);

fail!(
	array_element_access_bounds,
	r#"
        let arr = [1, 2, 3];
        let invalid: string = arr[0];
    "#,
	&["Type 'number' is not assignable to type 'string'."]
);

pass!(
	nested_array_2d,
	r#"
        let matrix: number[][] = [[1, 2], [3, 4], [5, 6]];
        matrix satisfies number[][];
    "#
);

pass!(
	nested_array_3d,
	r#"
        let cube: number[][][] = [[[1, 2]], [[3, 4]]];
        cube satisfies number[][][];
    "#
);

pass!(
	member_assignment,
	r#"
        let arr: number[] = [1, 2, 3];
        arr[0] = 10;
        arr[4] = 4;
        arr[100] = 100;
        arr[-1] = 1;
    "#
);

fail!(
	element_assignment_type_mismatch,
	r#"
        let arr: number[] = [1, 2, 3];
        arr[0] = "wrong";
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

fail!(
	array_with_no_annotation,
	r#"
        let arr = [];
        arr satisfies unknown[];
        arr satisfies number[]; // ERROR
        arr[0] satisfies unknown;
        arr[0] satisfies number; // ERROR
    "#,
	&[
		"Type 'unknown[]' is not assignable to type 'number[]'.",
		"Type 'unknown' is not assignable to type 'number'.",
	]
);

pass!(
	array_with_no_annotation_inference,
	r#"
        let arr = [];
        arr satisfies unknown[];
        arr[0] = 42;
        arr satisfies number[];
    "#
);
