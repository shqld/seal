use super::{fail, pass};

pass!(
	template_literal_simple,
	r#"
        let str = `hello world`;
        str satisfies string;
    "#
);

pass!(
	template_literal_with_interpolation,
	r#"
        let name = "Alice";
        let age = 30;
        let message = `Hello, ${name}! You are ${age} years old.`;
        message satisfies string;
    "#
);

pass!(
	template_literal_nested,
	r#"
        let a = "world";
        let b = `hello ${a}`;
        let c = `${b}!`;
        c satisfies string;
    "#
);

pass!(
	template_literal_multiline,
	r#"
        let html = `
            <div>
                <h1>Title</h1>
                <p>Content</p>
            </div>
        `;
        html satisfies string;
    "#
);

pass!(
	template_literal_nested_expressions,
	r#"
        let x = 5;
        let y = 10;
        let result = `The sum of ${x} and ${y} is ${x + y}`;
        result satisfies string;
    "#
);

pass!(
	regex_literal_simple,
	r#"
        let re = /hello/;
        re satisfies RegExp;
    "#
);

pass!(
	regex_literal_with_flags,
	r#"
        let re = /hello/gi;
        re satisfies RegExp;
    "#
);

pass!(
	regex_literal_properties,
	r#"
        let re = /hello/gi;
        re.source satisfies string;
        re.global satisfies boolean;
        re.ignoreCase satisfies boolean;
        re.multiline satisfies boolean;
    "#
);

pass!(
	regex_complex_patterns,
	r#"
        let emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        let phonePattern = /^\+?[\d\s\-\(\)]+$/;
        let urlPattern = /https?:\/\/[^\s]+/gi;
        
        emailPattern satisfies RegExp;
        phonePattern satisfies RegExp;
        urlPattern satisfies RegExp;
    "#
);

pass!(
	string_concatenation,
	r#"
        let greeting = "Hello";
        let space = " ";
        let name = "World";
        let message = greeting + space;
        let fullMessage = message + name;
        fullMessage satisfies string;
    "#
);

pass!(
	string_template_complex,
	r#"
        let user = "Alice";
        let age = 30;
        let active = true;
        let profile = `User: ${user}, Age: ${age}, Active: ${active}`;
        profile satisfies string;
    "#
);

pass!(
	number_proto,
	r#"
        let n = 42;

        n.toExponential satisfies (fractionDigits?: number) => string;
        n.toFixed satisfies (fractionDigits?: number) => string;
        n.toLocaleString satisfies () => string;
        n.toPrecision satisfies (precision?: number) => string;
    "#
);

fail!(
	number_proto_non_existent_method,
	r#"
        let n = 42;
        n.foo;
    "#,
	&["Property 'foo' does not exist on type 'number'."]
);

pass!(
	string_proto,
	r#"
        let s = "hello";

        s.length satisfies number;
        s.at satisfies (index: number) => string;
        s.charAt satisfies (index: number) => string;
        s.charCodeAt satisfies (index: number) => number;
        s.codePointAt satisfies (index: number) => number;
        s.concat satisfies (strings: string) => string;
        s.endsWith satisfies (searchString: string) => boolean;
        s.includes satisfies (searchString: string) => boolean;
        s.indexOf satisfies (searchString: string) => number;
        s.isWellFormed satisfies () => boolean;
        s.lastIndexOf satisfies (searchString: string) => number;
        s.localeCompare satisfies (compareString: string) => number;
        // TODO: object
        // s.match satisfies (regexp: string) => object;
        // TODO: object
        // s.matchAll satisfies (regexp: string) => object;
        s.normalize satisfies (form: string) => string;
        s.padEnd satisfies (targetLength: number, padString: string) => string;
        s.padStart satisfies (targetLength: number, padString: string) => string;
        s.repeat satisfies (count: number) => string;
        s.replace satisfies (searchValue: string, replaceValue: string) => string;
        s.replaceAll satisfies (searchValue: string, replaceValue: string) => string;
        s.search satisfies (regexp: string) => number;
        s.slice satisfies (start: number, end: number) => string;
        // TODO: object
        // s.split satisfies (separator: string, limit: number) => object;
        s.startsWith satisfies (searchString: string, position: number) => boolean;
        s.substr satisfies (start: number, length: number) => string;
        s.substring satisfies (start: number, end: number) => string;
        s.toLocaleLowerCase satisfies () => string;
        s.toLocaleUpperCase satisfies () => string;
        s.toLowerCase satisfies () => string;
        s.toUpperCase satisfies () => string;
        s.toWellFormed satisfies () => string;
        s.trim satisfies () => string;
        s.trimEnd satisfies () => string;
        s.trimLeft satisfies () => string;
        s.trimRight satisfies () => string;
        s.trimStart satisfies () => string;
    "#
);

fail!(
	string_proto_non_existent_method,
	r#"
        let s = "hello";
        s.foo;
    "#,
	&["Property 'foo' does not exist on type 'string'."]
);
