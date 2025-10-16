//! Documentation tests for compound literal transformations from C to Rust
//!
//! This test suite documents how C99 compound literals transform to Rust
//! struct/array literals and temporary values.
//!
//! ## C99 Standard References
//! - ISO C99 §6.5.2.5: Compound literals (new in C99)
//! - K&R C 2nd Ed: Does NOT cover compound literals (pre-C99)
//!
//! ## Key Transformations
//!
//! ### Struct Compound Literals
//! ```c
//! draw((struct Point){ .x = 10, .y = 20 });  // C99
//! ```
//! ```rust
//! draw(Point { x: 10, y: 20 });               // Rust
//! ```
//!
//! ### Array Compound Literals
//! ```c
//! int* p = (int[]){ 1, 2, 3, 4, 5 };         // C99
//! ```
//! ```rust
//! let arr = [1, 2, 3, 4, 5];                  // Rust
//! let p = &arr;
//! ```
//!
//! ### Pointer to Compound Literal
//! ```c
//! struct Point* p = &(struct Point){ .x = 1, .y = 2 };  // C99
//! ```
//! ```rust
//! let temp = Point { x: 1, y: 2 };                       // Rust
//! let p = &temp;
//! ```
//!
//! ## Safety Considerations
//!
//! - **Lifetime**: C99 compound literals have automatic storage duration
//! - **Scope**: Valid until end of enclosing block (like Rust temporaries)
//! - **Modifiable**: C99 compound literals can be modified (lvalue)
//! - **Type-safe**: Rust prevents type mismatches at compile time
//! - **No dangling**: Rust borrow checker prevents dangling references
//!
//! ## Common Patterns
//!
//! 1. **Function arguments**: Pass temporary structs without named variables
//! 2. **Array initialization**: Create arrays inline
//! 3. **Designated initializers**: Combine with designated initializers
//! 4. **Return values**: Return temporary structures
//!
//! ## Differences from C
//!
//! - Rust doesn't need cast syntax `(Type){ ... }`
//! - Rust struct literals are always compound literals
//! - Rust temporaries have clear lifetime rules (borrow checker)
//! - Rust array literals don't need pointer to use as slice

#[cfg(test)]
mod compound_literals_documentation_tests {
    /// Document transformation of simple struct compound literal
    ///
    /// C99: `(struct Point){ .x = 10, .y = 20 }`
    /// Rust: `Point { x: 10, y: 20 }`
    ///
    /// Reference: ISO C99 §6.5.2.5
    #[test]
    fn test_simple_struct_compound_literal() {
        let c_code = "(struct Point){ .x = 10, .y = 20 }";
        let rust_equivalent = "Point { x: 10, y: 20 }";

        #[derive(Debug, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        // Rust doesn't need cast syntax - struct literals are always compound literals
        let p = Point { x: 10, y: 20 };
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);

        assert!(c_code.contains("(struct Point)"));
        assert!(rust_equivalent.contains("Point {"));
    }

    /// Document compound literal as function argument
    ///
    /// C99: `draw((struct Point){ .x = 10, .y = 20 });`
    /// Rust: `draw(Point { x: 10, y: 20 });`
    ///
    /// Reference: ISO C99 §6.5.2.5 (compound literals as arguments)
    #[test]
    fn test_compound_literal_as_argument() {
        let c_code = "draw((struct Point){ .x = 10, .y = 20 });";
        let rust_equivalent = "draw(Point { x: 10, y: 20 });";

        #[derive(Debug, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        fn draw(p: Point) -> i32 {
            p.x + p.y
        }

        // Pass temporary struct directly to function
        let result = draw(Point { x: 10, y: 20 });
        assert_eq!(result, 30);

        assert!(c_code.contains("draw((struct Point)"));
        assert!(rust_equivalent.contains("draw(Point {"));
    }

    /// Document array compound literal
    ///
    /// C99: `(int[]){ 1, 2, 3, 4, 5 }`
    /// Rust: `[1, 2, 3, 4, 5]`
    ///
    /// Reference: ISO C99 §6.5.2.5
    #[test]
    fn test_array_compound_literal() {
        let c_code = "int* p = (int[]){ 1, 2, 3, 4, 5 };";
        let rust_equivalent = "let p = &[1, 2, 3, 4, 5];";

        // Rust array literal (no cast needed)
        let arr = [1, 2, 3, 4, 5];
        let p: &[i32] = &arr; // Reference to array (slice)

        assert_eq!(p[0], 1);
        assert_eq!(p[4], 5);
        assert_eq!(p.len(), 5);

        assert!(c_code.contains("(int[])"));
        assert!(rust_equivalent.contains("[1, 2, 3, 4, 5]"));
    }

    /// Document pointer to compound literal
    ///
    /// C99: `struct Point* p = &(struct Point){ .x = 1, .y = 2 };`
    /// Rust: `let temp = Point { x: 1, y: 2 }; let p = &temp;`
    ///
    /// Reference: ISO C99 §6.5.2.5
    #[test]
    fn test_pointer_to_compound_literal() {
        let c_code = "struct Point* p = &(struct Point){ .x = 1, .y = 2 };";
        let rust_equivalent = "let temp = Point { x: 1, y: 2 }; let p = &temp;";

        #[derive(Debug)]
        struct Point {
            x: i32,
            y: i32,
        }

        // Rust requires explicit temporary for reference
        let temp = Point { x: 1, y: 2 };
        let p = &temp;

        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);

        assert!(c_code.contains("&(struct Point)"));
        assert!(rust_equivalent.contains("&temp"));
    }

    /// Document compound literal with designated initializers
    ///
    /// C99: `(struct RGB){ .r = 255, .g = 128, .b = 0 }`
    /// Rust: `RGB { r: 255, g: 128, b: 0 }`
    ///
    /// Reference: ISO C99 §6.5.2.5, §6.7.8.17
    #[test]
    fn test_compound_literal_with_designated_init() {
        let c_code = "(struct RGB){ .r = 255, .g = 128, .b = 0 }";
        let rust_equivalent = "RGB { r: 255, g: 128, b: 0 }";

        #[allow(clippy::upper_case_acronyms)]
        struct RGB {
            r: u8,
            g: u8,
            b: u8,
        }

        fn set_color(color: RGB) -> u32 {
            (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32
        }

        // Compound literal with designated initializers
        let color_value = set_color(RGB {
            r: 255,
            g: 128,
            b: 0,
        });
        assert_eq!(color_value, 0xFF8000);

        assert!(c_code.contains(".r = 255"));
        assert!(rust_equivalent.contains("r: 255"));
    }

    /// Document modifiable compound literal (C99 feature)
    ///
    /// C99: Compound literals are lvalues and can be modified
    /// Rust: Struct literals can be mutable if bound to mut variable
    ///
    /// Reference: ISO C99 §6.5.2.5 (compound literals are lvalues)
    #[test]
    fn test_modifiable_compound_literal() {
        let c_code = r#"
struct Point* p = &(struct Point){ .x = 1, .y = 2 };
p->x = 10;  // Modifying compound literal
"#;
        let rust_equivalent = r#"
let mut temp = Point { x: 1, y: 2 };
let p = &mut temp;
p.x = 10;
"#;

        #[derive(Debug)]
        struct Point {
            x: i32,
            y: i32,
        }

        // Rust: need mut binding to modify
        let mut temp = Point { x: 1, y: 2 };
        let p = &mut temp;
        p.x = 10;

        assert_eq!(p.x, 10);
        assert_eq!(p.y, 2);

        assert!(c_code.contains("p->x = 10"));
        assert!(rust_equivalent.contains("p.x = 10"));
    }

    /// Document compound literal in return statement
    ///
    /// C99: `return (struct Point){ .x = 0, .y = 0 };`
    /// Rust: `return Point { x: 0, y: 0 };`
    ///
    /// Reference: ISO C99 §6.5.2.5
    #[test]
    fn test_compound_literal_in_return() {
        let c_code = "return (struct Point){ .x = 0, .y = 0 };";
        let rust_equivalent = "Point { x: 0, y: 0 }"; // implicit return

        #[derive(Debug, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        fn origin() -> Point {
            Point { x: 0, y: 0 } // Compound literal as return value
        }

        let p = origin();
        assert_eq!(p, Point { x: 0, y: 0 });

        assert!(c_code.contains("return (struct Point)"));
        assert!(rust_equivalent.contains("Point {"));
    }

    /// Document nested compound literals
    ///
    /// C99: `(struct Outer){ .inner = (struct Inner){ .x = 1 } }`
    /// Rust: `Outer { inner: Inner { x: 1 } }`
    ///
    /// Reference: ISO C99 §6.5.2.5
    #[test]
    fn test_nested_compound_literals() {
        let c_code = "(struct Outer){ .inner = (struct Inner){ .x = 1, .y = 2 } }";
        let rust_equivalent = "Outer { inner: Inner { x: 1, y: 2 } }";

        struct Inner {
            x: i32,
            y: i32,
        }

        struct Outer {
            inner: Inner,
        }

        fn process(o: Outer) -> i32 {
            o.inner.x + o.inner.y
        }

        // Nested compound literals
        let result = process(Outer {
            inner: Inner { x: 1, y: 2 },
        });
        assert_eq!(result, 3);

        assert!(c_code.contains("(struct Outer)"));
        assert!(rust_equivalent.contains("Outer { inner:"));
    }

    /// Document compound literal with array of structs
    ///
    /// C99: `(struct Point[]){ {1,2}, {3,4} }`
    /// Rust: `[Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]`
    ///
    /// Reference: ISO C99 §6.5.2.5
    #[test]
    fn test_compound_literal_array_of_structs() {
        let c_code = "(struct Point[]){ {.x=1, .y=2}, {.x=3, .y=4} }";
        let rust_equivalent = "[Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]";

        #[derive(Debug, Clone, Copy)]
        struct Point {
            x: i32,
            y: i32,
        }

        fn sum_points(points: &[Point]) -> i32 {
            points.iter().map(|p| p.x + p.y).sum()
        }

        // Array of struct literals
        let arr = [Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];
        let result = sum_points(&arr);
        assert_eq!(result, 10); // (1+2) + (3+4)

        assert!(c_code.contains("(struct Point[])"));
        assert!(rust_equivalent.contains("Point { x: 1"));
    }

    /// Document compound literal lifetime and scope
    ///
    /// C99: Compound literal has automatic storage duration (scope of block)
    /// Rust: Temporaries live until end of statement (or extended by borrow)
    ///
    /// Reference: ISO C99 §6.5.2.5 (storage duration)
    #[test]
    fn test_compound_literal_lifetime() {
        let c_note = "C99 compound literal lives until end of block";
        let rust_note = "Rust temporary extended by reference lifetime";

        #[derive(Debug)]
        struct Point {
            x: i32,
            #[allow(dead_code)]
            y: i32,
        }

        // Rust: temporary is extended by the reference
        let p = &Point { x: 10, y: 20 };
        assert_eq!(p.x, 10);

        // The temporary Point lives as long as the reference p

        assert!(c_note.contains("end of block"));
        assert!(rust_note.contains("extended by reference"));
    }

    /// Document compound literal with string initialization
    ///
    /// C99: `(struct Person){ .name = "Alice", .age = 30 }`
    /// Rust: `Person { name: "Alice", age: 30 }`
    ///
    /// Reference: ISO C99 §6.5.2.5
    #[test]
    fn test_compound_literal_with_strings() {
        let c_code = "(struct Person){ .name = \"Alice\", .age = 30 }";
        let rust_equivalent = "Person { name: \"Alice\", age: 30 }";

        struct Person<'a> {
            name: &'a str,
            age: i32,
        }

        fn greet(person: Person) -> String {
            format!("{} is {} years old", person.name, person.age)
        }

        let greeting = greet(Person {
            name: "Alice",
            age: 30,
        });
        assert_eq!(greeting, "Alice is 30 years old");

        assert!(c_code.contains(".name = \"Alice\""));
        assert!(rust_equivalent.contains("name: \"Alice\""));
    }

    /// Document compound literal in initialization
    ///
    /// C99: `struct Point p = (struct Point){ .x = 1, .y = 2 };`
    /// Rust: `let p = Point { x: 1, y: 2 };`
    ///
    /// Reference: ISO C99 §6.5.2.5
    #[test]
    fn test_compound_literal_in_initialization() {
        let c_code = "struct Point p = (struct Point){ .x = 1, .y = 2 };";
        let rust_equivalent = "let p = Point { x: 1, y: 2 };";

        #[derive(Debug, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        // Rust doesn't need the cast - struct literal is enough
        let p = Point { x: 1, y: 2 };
        assert_eq!(p, Point { x: 1, y: 2 });

        assert!(c_code.contains("(struct Point)"));
        assert!(rust_equivalent.contains("Point {"));
    }

    /// Document compound literal with partial initialization
    ///
    /// C99: `(struct Point){ .x = 10 }` (y is zero)
    /// Rust: `Point { x: 10, ..Default::default() }`
    ///
    /// Reference: ISO C99 §6.5.2.5, §6.7.8.21
    #[test]
    fn test_compound_literal_partial_init() {
        let c_code = "(struct Point){ .x = 10 }"; // y implicitly 0
        let rust_equivalent = "Point { x: 10, ..Default::default() }";

        #[derive(Debug, Default, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        fn process(p: Point) -> i32 {
            p.x * 2 + p.y
        }

        let result = process(Point {
            x: 10,
            ..Default::default()
        });
        assert_eq!(result, 20); // 10*2 + 0

        assert!(c_code.contains(".x = 10"));
        assert!(rust_equivalent.contains("..Default::default()"));
    }

    /// Document compound literal in array subscript context
    ///
    /// C99: Can use compound literal directly as array
    /// Rust: Array literal can be indexed directly
    ///
    /// Reference: ISO C99 §6.5.2.5
    #[test]
    fn test_compound_literal_array_subscript() {
        let c_code = "(int[]){ 10, 20, 30, 40, 50 }[2]"; // Access element 2
        let rust_equivalent = "[10, 20, 30, 40, 50][2]";

        // Rust: can index array literal directly
        let value = [10, 20, 30, 40, 50][2];
        assert_eq!(value, 30);

        // Alternative: compound literal in expression
        let result = [10, 20, 30, 40, 50].iter().sum::<i32>();
        assert_eq!(result, 150);

        assert!(c_code.contains("(int[])"));
        assert!(rust_equivalent.contains("[10, 20, 30, 40, 50][2]"));
    }

    /// Document compound literal with const (C99 restriction)
    ///
    /// C99: Compound literals cannot be used in const expressions (C99 limitation)
    /// Rust: Const expressions support struct literals in some contexts
    ///
    /// Reference: ISO C99 §6.5.2.5 (not constant expressions)
    #[test]
    fn test_compound_literal_const_context() {
        let c_note = "C99 compound literals are NOT constant expressions";
        let rust_note = "Rust const can use struct literals in many contexts";

        struct Point {
            x: i32,
            y: i32,
        }

        // Rust: const with struct literal works
        const ORIGIN: Point = Point { x: 0, y: 0 };
        assert_eq!(ORIGIN.x, 0);
        assert_eq!(ORIGIN.y, 0);

        // C99 would require: const struct Point ORIGIN = {0, 0}; (not compound literal)

        assert!(c_note.contains("NOT constant"));
        assert!(rust_note.contains("const can use"));
    }

    /// Document transformation rules summary
    ///
    /// This test documents all transformation rules and differences
    /// between C99 compound literals and Rust struct/array literals.
    #[test]
    fn test_compound_literal_transformation_rules() {
        let c_summary = r#"
C99 Compound Literal Rules:
1. Syntax: (Type){ initializer-list }
2. Creates unnamed object with automatic storage
3. Can be modified (lvalue)
4. Scope: until end of block
5. Can take address: &(Type){...}
6. NOT constant expressions
7. Combines with designated initializers
"#;

        let rust_summary = r#"
Rust Struct/Array Literal Rules:
1. Syntax: Type { fields } or [elements]
2. No cast syntax needed
3. Can be mut if bound to mut variable
4. Lifetime: temporary extended by references
5. Borrow checker prevents dangling
6. Const expressions allowed in many contexts
7. All fields must be initialized (or use ..Default)
"#;

        #[derive(Debug)]
        struct Point {
            x: i32,
            #[allow(dead_code)]
            y: i32,
        }

        // Rust: no cast needed
        let p = Point { x: 10, y: 20 };
        assert_eq!(p.x, 10);

        // Can take reference (temporary extended)
        let r = &Point { x: 1, y: 2 };
        assert_eq!(r.x, 1);

        assert!(c_summary.contains("(Type)"));
        assert!(rust_summary.contains("No cast syntax"));
    }
}
