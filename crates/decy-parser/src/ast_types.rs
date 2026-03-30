//! AST type definitions for parsed C code.
//!
//! This module contains all the data types used to represent a parsed C program:
//! statements, expressions, types, operators, functions, structs, enums,
//! typedefs, variables, and macros.

/// Represents a single case in a switch statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    /// Case value expression (None for default case)
    pub value: Option<Expression>,
    /// Statements to execute for this case
    pub body: Vec<Statement>,
}

/// Represents a C statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Variable declaration: `int* ptr = malloc(4);`
    VariableDeclaration {
        /// Variable name
        name: String,
        /// Variable type
        var_type: Type,
        /// Optional initializer expression
        initializer: Option<Expression>,
    },
    /// Return statement: `return expr;`
    Return(Option<Expression>),
    /// Assignment statement: `x = 42;`
    Assignment {
        /// Target variable name
        target: String,
        /// Value expression to assign
        value: Expression,
    },
    /// If statement: `if (cond) { ... } else { ... }`
    If {
        /// Condition expression
        condition: Expression,
        /// Then block
        then_block: Vec<Statement>,
        /// Optional else block
        else_block: Option<Vec<Statement>>,
    },
    /// For loop: `for (init; cond; inc) { ... }`
    For {
        /// Init statements (can be multiple with comma: int i = 0, j = 10)
        init: Vec<Statement>,
        /// Optional condition expression
        condition: Option<Expression>,
        /// Optional increment statements (can be multiple with comma: i++, j--)
        increment: Vec<Statement>,
        /// Loop body
        body: Vec<Statement>,
    },
    /// While loop: `while (cond) { ... }`
    While {
        /// Condition expression
        condition: Expression,
        /// Loop body
        body: Vec<Statement>,
    },
    /// Pointer dereference assignment: `*ptr = value;`
    DerefAssignment {
        /// Target expression to dereference
        target: Expression,
        /// Value expression to assign
        value: Expression,
    },
    /// Array index assignment: `arr[i] = value;`
    ArrayIndexAssignment {
        /// Array expression
        array: Box<Expression>,
        /// Index expression
        index: Box<Expression>,
        /// Value expression to assign
        value: Expression,
    },
    /// Field assignment: `ptr->field = value;` or `obj.field = value;`
    FieldAssignment {
        /// Object/pointer expression
        object: Expression,
        /// Field name
        field: String,
        /// Value expression to assign
        value: Expression,
    },
    /// Break statement: `break;`
    Break,
    /// Continue statement: `continue;`
    Continue,
    /// Switch statement: `switch (expr) { case 1: ...; default: ...; }`
    Switch {
        /// Condition expression to switch on
        condition: Expression,
        /// List of case statements
        cases: Vec<SwitchCase>,
        /// Optional default case body
        default_case: Option<Vec<Statement>>,
    },
    /// Post-increment statement: `ptr++;`
    PostIncrement {
        /// Target variable name
        target: String,
    },
    /// Pre-increment statement: `++ptr;`
    PreIncrement {
        /// Target variable name
        target: String,
    },
    /// Post-decrement statement: `ptr--;`
    PostDecrement {
        /// Target variable name
        target: String,
    },
    /// Pre-decrement statement: `--ptr;`
    PreDecrement {
        /// Target variable name
        target: String,
    },
    /// Compound assignment: `ptr += offset;`, `x *= 2;`, etc.
    CompoundAssignment {
        /// Target variable name
        target: String,
        /// Binary operator to apply
        op: BinaryOperator,
        /// Value expression
        value: Expression,
    },
    /// DECY-185: Compound assignment to expression target: `*ptr *= 2;`, `sb->capacity *= 2;`
    /// Used when target is not a simple variable (Dereference, PointerFieldAccess, etc.)
    DerefCompoundAssignment {
        /// Target expression (e.g., the dereferenced pointer or field access)
        target: Expression,
        /// Binary operator to apply
        op: BinaryOperator,
        /// Value expression
        value: Expression,
    },
    /// Function call statement: `strlen(s);`, `strcpy(dst, src);`
    FunctionCall {
        /// Function name
        function: String,
        /// Arguments
        arguments: Vec<Expression>,
    },
}

impl Statement {
    /// Check if this statement is a string function call.
    pub fn is_string_function_call(&self) -> bool {
        match self {
            Statement::FunctionCall { function, .. } => {
                matches!(function.as_str(), "strlen" | "strcmp" | "strcpy" | "strdup")
            }
            _ => false,
        }
    }

    /// Check if this statement is a function call.
    pub fn is_function_call(&self) -> bool {
        matches!(self, Statement::FunctionCall { .. })
    }

    /// Convert this statement to a function call expression if it is one.
    ///
    /// # Implementation Status
    ///
    /// Stub implementation - always returns `None`.
    /// The `Statement::FunctionCall` variant doesn't store the call as an `Expression`,
    /// so conversion would require reconstructing an `Expression::FunctionCall` from
    /// the statement's fields.
    pub fn as_function_call(&self) -> Option<&Expression> {
        None
    }
}

/// Unary operators for C expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Unary minus (-x)
    Minus,
    /// Logical NOT (!x)
    LogicalNot,
    /// Bitwise NOT (~x)
    BitwiseNot,
    /// Address-of (&x)
    AddressOf,
}

/// Binary operators for C expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Subtract,
    /// Multiplication (*)
    Multiply,
    /// Division (/)
    Divide,
    /// Modulo (%)
    Modulo,
    /// Equality (==)
    Equal,
    /// Inequality (!=)
    NotEqual,
    /// Less than (<)
    LessThan,
    /// Greater than (>)
    GreaterThan,
    /// Less than or equal (<=)
    LessEqual,
    /// Greater than or equal (>=)
    GreaterEqual,
    /// Logical AND (&&)
    LogicalAnd,
    /// Logical OR (||)
    LogicalOr,
    /// Left shift (<<)
    LeftShift,
    /// Right shift (>>)
    RightShift,
    /// Bitwise AND (&)
    BitwiseAnd,
    /// Bitwise OR (|)
    BitwiseOr,
    /// Bitwise XOR (^)
    BitwiseXor,
    /// Assignment (=) - used for embedded assignments like (c=getchar())
    Assign,
    /// Comma operator (,) - DECY-224: for multi-expression statements
    Comma,
}

/// Represents a C expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Integer literal: `42`
    IntLiteral(i32),
    /// Float literal: `3.14` (stored as string to preserve precision)
    FloatLiteral(String),
    /// String literal: `"hello"`
    StringLiteral(String),
    /// Character literal: `'a'`, `'\0'`, `'\n'`
    CharLiteral(i8),
    /// Variable reference: `x`
    Variable(String),
    /// Binary operation: `a + b`
    BinaryOp {
        /// Operator
        op: BinaryOperator,
        /// Left operand
        left: Box<Expression>,
        /// Right operand
        right: Box<Expression>,
    },
    /// Function call: `malloc(4)`
    FunctionCall {
        /// Function name
        function: String,
        /// Arguments
        arguments: Vec<Expression>,
    },
    /// Pointer dereference: `*ptr`
    Dereference(Box<Expression>),
    /// Unary operation: `-x`, `!x`
    UnaryOp {
        /// Operator
        op: UnaryOperator,
        /// Operand
        operand: Box<Expression>,
    },
    /// Array indexing: `arr[i]`
    ArrayIndex {
        /// Array expression
        array: Box<Expression>,
        /// Index expression
        index: Box<Expression>,
    },
    /// Struct field access: `obj.field`
    FieldAccess {
        /// Object expression
        object: Box<Expression>,
        /// Field name
        field: String,
    },
    /// Pointer field access: `ptr->field`
    PointerFieldAccess {
        /// Pointer expression
        pointer: Box<Expression>,
        /// Field name
        field: String,
    },
    /// Post-increment expression: `ptr++`
    PostIncrement {
        /// Operand expression
        operand: Box<Expression>,
    },
    /// Pre-increment expression: `++ptr`
    PreIncrement {
        /// Operand expression
        operand: Box<Expression>,
    },
    /// Post-decrement expression: `ptr--`
    PostDecrement {
        /// Operand expression
        operand: Box<Expression>,
    },
    /// Pre-decrement expression: `--ptr`
    PreDecrement {
        /// Operand expression
        operand: Box<Expression>,
    },
    /// Sizeof expression: `sizeof(int)` or `sizeof(struct Data)`
    Sizeof {
        /// Type name as a string (e.g., "int", "struct Data")
        type_name: String,
    },
    /// Cast expression: `(int)x` or `(void*)ptr`
    ///
    /// C-style cast that converts an expression to a target type.
    /// Maps to Rust `as` operator for safe casts, or `transmute` for unsafe casts.
    ///
    /// # Examples
    ///
    /// ```c
    /// int x = (int)3.14;           // float to int
    /// void* ptr = (void*)buffer;   // pointer cast
    /// long l = (long)small_int;    // widening cast
    /// ```
    Cast {
        /// Target type to cast to
        target_type: Type,
        /// Expression being cast
        expr: Box<Expression>,
    },
    /// Compound literal: `(struct Point){10, 20}` or `(int[]){1, 2, 3}`
    ///
    /// C99 compound literals create anonymous objects of a specified type.
    /// Useful for passing struct values to functions or creating temporary objects.
    ///
    /// # Examples
    ///
    /// ```c
    /// struct Point p = (struct Point){10, 20};       // struct compound literal
    /// int* arr = (int[]){1, 2, 3, 4, 5};             // array compound literal
    /// draw((struct Rect){.x=0, .y=0, .w=100, .h=50}); // with designated initializers
    /// ```
    CompoundLiteral {
        /// Type of the compound literal (struct Point, int[], etc.)
        literal_type: Type,
        /// Initializer expressions (values for struct fields or array elements)
        initializers: Vec<Expression>,
    },
    /// Ternary/Conditional expression: `cond ? then_val : else_val`
    ///
    /// The C ternary operator evaluates the condition and returns either
    /// the then_val or else_val based on whether condition is truthy.
    ///
    /// # DECY-192
    ///
    /// Added to support K&R Chapter 2.11 Conditional Expressions.
    ///
    /// # Examples
    ///
    /// ```c
    /// int max = (a > b) ? a : b;
    /// char* msg = (x == 0) ? "zero" : "nonzero";
    /// ```
    Ternary {
        /// Condition expression (evaluated as boolean)
        condition: Box<Expression>,
        /// Value if condition is true
        then_expr: Box<Expression>,
        /// Value if condition is false
        else_expr: Box<Expression>,
    },
    /// C++ new expression: `new T(args)` (DECY-207)
    CxxNew {
        /// Type name being allocated
        type_name: String,
        /// Constructor arguments
        arguments: Vec<Expression>,
    },
    /// C++ delete expression: `delete ptr` (DECY-207)
    CxxDelete {
        /// Expression being deleted
        operand: Box<Expression>,
    },
    /// C++ nullptr literal (DECY-226)
    NullLiteral,
    /// C++ bool literal true/false (DECY-226)
    BoolLiteral(bool),
}

impl Expression {
    /// Check if this expression is a string function call (strlen, strcmp, strcpy, strdup).
    pub fn is_string_function_call(&self) -> bool {
        match self {
            Expression::FunctionCall { function, .. } => {
                matches!(function.as_str(), "strlen" | "strcmp" | "strcpy" | "strdup")
            }
            _ => false,
        }
    }

    /// Get the string function name if this is a string function call.
    pub fn string_function_name(&self) -> Option<&str> {
        match self {
            Expression::FunctionCall { function, .. } if self.is_string_function_call() => {
                Some(function.as_str())
            }
            _ => None,
        }
    }

    /// Check if this expression has a string literal argument.
    pub fn has_string_literal_argument(&self) -> bool {
        match self {
            Expression::FunctionCall { arguments, .. } => {
                arguments.iter().any(|arg| matches!(arg, Expression::StringLiteral(_)))
            }
            _ => false,
        }
    }
}

/// Represents a C typedef declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Typedef {
    /// Typedef name (the alias)
    pub name: String,
    /// Underlying type being aliased
    pub underlying_type: Type,
}

impl Typedef {
    /// Create a new typedef.
    pub fn new(name: String, underlying_type: Type) -> Self {
        Self { name, underlying_type }
    }

    /// Get the typedef name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the underlying type as a string representation.
    pub fn underlying_type(&self) -> &str {
        // Return a string representation of the type
        match &self.underlying_type {
            Type::Void => "void",
            Type::Bool => "_Bool",
            Type::Int => "int",
            Type::UnsignedInt => "unsigned int", // DECY-158
            Type::Float => "float",
            Type::Double => "double",
            Type::Char => "char",
            Type::SignedChar => "signed char", // DECY-250
            Type::Pointer(inner) => match **inner {
                Type::Char => "char*",
                Type::SignedChar => "signed char*", // DECY-250
                Type::Int => "int*",
                Type::UnsignedInt => "unsigned int*", // DECY-158
                Type::Float => "float*",
                Type::Double => "double*",
                Type::Void => "void*",
                _ => "pointer",
            },
            Type::Struct(name) => name,
            Type::FunctionPointer { .. } => "function pointer",
            Type::Array { .. } => "array",
            // DECY-172: TypeAlias returns the alias name
            Type::TypeAlias(name) => name,
        }
    }

    /// Check if this typedef is a pointer type.
    pub fn is_pointer(&self) -> bool {
        matches!(self.underlying_type, Type::Pointer(_))
    }

    /// Check if this typedef is a struct type.
    pub fn is_struct(&self) -> bool {
        matches!(self.underlying_type, Type::Struct(_))
    }

    /// Check if this typedef is a function pointer type.
    pub fn is_function_pointer(&self) -> bool {
        matches!(self.underlying_type, Type::FunctionPointer { .. })
    }

    /// Check if this typedef is an array type.
    pub fn is_array(&self) -> bool {
        // Arrays are not yet in the Type enum, so return false for now
        false
    }
}

/// Represents a struct field.
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: Type,
}

impl StructField {
    /// Create a new struct field.
    pub fn new(name: String, field_type: Type) -> Self {
        Self { name, field_type }
    }

    /// Get the field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if this field is a function pointer.
    pub fn is_function_pointer(&self) -> bool {
        matches!(self.field_type, Type::FunctionPointer { .. })
    }
}

/// Represents a struct definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    /// Struct name
    pub name: String,
    /// Struct fields
    pub fields: Vec<StructField>,
}

impl Struct {
    /// Create a new struct.
    pub fn new(name: String, fields: Vec<StructField>) -> Self {
        Self { name, fields }
    }

    /// Get the struct name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the struct fields.
    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }
}

/// C++ access specifier (DECY-200).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessSpecifier {
    /// public:
    Public,
    /// protected:
    Protected,
    /// private: (default for class)
    Private,
}

/// C++ overloaded operator kind (DECY-208).
///
/// Maps to Rust `std::ops` traits in codegen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CxxOperatorKind {
    /// operator+ -> std::ops::Add
    Add,
    /// operator- -> std::ops::Sub
    Sub,
    /// operator* -> std::ops::Mul
    Mul,
    /// operator/ -> std::ops::Div
    Div,
    /// operator% -> std::ops::Rem
    Rem,
    /// operator== -> PartialEq::eq
    Equal,
    /// operator!= -> PartialEq::ne (derived from eq)
    NotEqual,
    /// operator< -> PartialOrd::lt
    Less,
    /// operator<= -> PartialOrd::le
    LessEqual,
    /// operator> -> PartialOrd::gt
    Greater,
    /// operator>= -> PartialOrd::ge
    GreaterEqual,
    /// operator[] -> std::ops::Index
    Index,
    /// operator<< -> std::ops::Shl (or Display for ostream)
    Shl,
    /// operator>> -> std::ops::Shr
    Shr,
    /// operator+= -> std::ops::AddAssign
    AddAssign,
    /// operator-= -> std::ops::SubAssign
    SubAssign,
}

/// A C++ class method with its access level (DECY-200).
#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    /// The function definition
    pub function: Function,
    /// Access specifier (public/protected/private)
    pub access: AccessSpecifier,
    /// Whether this is a const method
    pub is_const: bool,
    /// Whether this is a static method
    pub is_static: bool,
    /// Whether this is virtual
    pub is_virtual: bool,
    /// Overloaded operator kind, if this is an operator method (DECY-208)
    pub operator_kind: Option<CxxOperatorKind>,
}

/// Represents a C++ class (DECY-200).
///
/// Maps to Rust `struct` + `impl` block. Constructors become `new()`,
/// destructors become `impl Drop`.
#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    /// Class name
    pub name: String,
    /// Data fields (inherited from struct model)
    pub fields: Vec<StructField>,
    /// Methods (including constructors/destructors)
    pub methods: Vec<Method>,
    /// Constructor parameters (from the first constructor found)
    pub constructor_params: Vec<Parameter>,
    /// Whether a destructor was defined
    pub has_destructor: bool,
    /// Base class name for single inheritance (DECY-209), None if no base
    pub base_class: Option<String>,
}

impl Class {
    /// Create a new empty class.
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            methods: Vec::new(),
            constructor_params: Vec::new(),
            has_destructor: false,
            base_class: None,
        }
    }
}

/// Represents a C++ namespace (DECY-201).
///
/// Maps to Rust `mod` block. Nested namespaces become nested modules.
#[derive(Debug, Clone, PartialEq)]
pub struct Namespace {
    /// Namespace name (empty for anonymous namespaces)
    pub name: String,
    /// Functions declared in this namespace
    pub functions: Vec<Function>,
    /// Structs declared in this namespace
    pub structs: Vec<Struct>,
    /// Classes declared in this namespace
    pub classes: Vec<Class>,
    /// Nested namespaces
    pub namespaces: Vec<Namespace>,
}

impl Namespace {
    /// Create a new empty namespace.
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: Vec::new(),
            structs: Vec::new(),
            classes: Vec::new(),
            namespaces: Vec::new(),
        }
    }
}

/// Represents a variable declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    /// Variable name
    name: String,
    /// Variable type
    var_type: Type,
    /// Optional initializer expression
    initializer: Option<Expression>,
    /// Static storage class (file-local)
    is_static: bool,
    /// Extern storage class (external linkage)
    is_extern: bool,
    /// Const qualifier (immutable)
    is_const: bool,
}

impl Variable {
    /// Create a new variable.
    pub fn new(name: String, var_type: Type) -> Self {
        Self {
            name,
            var_type,
            initializer: None,
            is_static: false,
            is_extern: false,
            is_const: false,
        }
    }

    /// Create a new variable with an initializer.
    pub fn new_with_initializer(name: String, var_type: Type, initializer: Expression) -> Self {
        Self {
            name,
            var_type,
            initializer: Some(initializer),
            is_static: false,
            is_extern: false,
            is_const: false,
        }
    }

    /// Create a new variable with storage class specifiers.
    pub fn new_with_storage_class(
        name: String,
        var_type: Type,
        initializer: Option<Expression>,
        is_static: bool,
        is_extern: bool,
        is_const: bool,
    ) -> Self {
        Self { name, var_type, initializer, is_static, is_extern, is_const }
    }

    /// Get the variable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the variable type.
    pub fn var_type(&self) -> &Type {
        &self.var_type
    }

    /// Check if this variable is a function pointer.
    pub fn is_function_pointer(&self) -> bool {
        matches!(self.var_type, Type::FunctionPointer { .. })
    }

    /// Get the number of parameters if this is a function pointer.
    pub fn function_pointer_param_count(&self) -> usize {
        match &self.var_type {
            Type::FunctionPointer { param_types, .. } => param_types.len(),
            _ => 0,
        }
    }

    /// Check if this function pointer has a void return type.
    pub fn function_pointer_has_void_return(&self) -> bool {
        match &self.var_type {
            Type::FunctionPointer { return_type, .. } => matches!(**return_type, Type::Void),
            _ => false,
        }
    }

    /// Check if this variable is a string literal (char* with literal initializer).
    ///
    /// Detects patterns like: `const char* msg = "Hello";`
    ///
    /// # Implementation
    ///
    /// Checks if:
    /// - Type is a pointer to char (`char*`)
    /// - Has an initializer that is a `StringLiteral` expression
    ///
    /// Note: Const qualifier detection not yet implemented - checks all char* pointers.
    pub fn is_string_literal(&self) -> bool {
        // Check if type is char*
        let is_char_ptr =
            matches!(self.var_type, Type::Pointer(ref inner) if **inner == Type::Char);

        // Check if initializer is a string literal
        if let Some(initializer) = &self.initializer {
            is_char_ptr && matches!(initializer, Expression::StringLiteral(_))
        } else {
            false
        }
    }

    /// Check if this variable is a string buffer (char* allocated with malloc).
    ///
    /// Detects patterns like: `char* buffer = malloc(100);`
    ///
    /// # Implementation
    ///
    /// Checks if:
    /// - Type is a pointer to char (`char*`)
    /// - Has an initializer that is a malloc/calloc function call
    pub fn is_string_buffer(&self) -> bool {
        // Check if type is char*
        let is_char_ptr =
            matches!(self.var_type, Type::Pointer(ref inner) if **inner == Type::Char);

        // Check if initializer is malloc/calloc call
        if let Some(Expression::FunctionCall { function, .. }) = &self.initializer {
            is_char_ptr && (function == "malloc" || function == "calloc")
        } else {
            false
        }
    }

    /// Get the initializer expression for this variable.
    ///
    /// Returns `Some(&Expression)` if the variable has an initializer, `None` otherwise.
    pub fn initializer(&self) -> Option<&Expression> {
        self.initializer.as_ref()
    }

    /// Check if this variable has static storage class (file-local).
    pub fn is_static(&self) -> bool {
        self.is_static
    }

    /// Check if this variable is extern (external linkage).
    pub fn is_extern(&self) -> bool {
        self.is_extern
    }

    /// Check if this variable is const (immutable).
    pub fn is_const(&self) -> bool {
        self.is_const
    }
}

/// Represents an enum variant (constant) in C.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    /// Variant name
    pub name: String,
    /// Explicit value if specified
    pub value: Option<i64>,
}

impl EnumVariant {
    /// Create a new enum variant.
    pub fn new(name: String, value: Option<i64>) -> Self {
        Self { name, value }
    }
}

/// Represents a C enum definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    /// Enum name (empty string for anonymous enums)
    pub name: String,
    /// Enum variants
    pub variants: Vec<EnumVariant>,
}

impl Enum {
    /// Create a new enum.
    pub fn new(name: String, variants: Vec<EnumVariant>) -> Self {
        Self { name, variants }
    }
}

/// Abstract Syntax Tree representing parsed C code.
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    functions: Vec<Function>,
    typedefs: Vec<Typedef>,
    structs: Vec<Struct>,
    macros: Vec<MacroDefinition>,
    variables: Vec<Variable>,
    enums: Vec<Enum>,
    /// C++ classes (DECY-200)
    classes: Vec<Class>,
    /// C++ namespaces (DECY-201)
    namespaces: Vec<Namespace>,
}

/// Represents a C macro definition (#define).
///
/// C macros come in two forms:
/// - **Object-like**: Simple text replacement (e.g., `#define MAX 100`)
/// - **Function-like**: Parameterized text replacement (e.g., `#define SQR(x) ((x) * (x))`)
///
/// # Examples
///
/// ```no_run
/// use decy_parser::parser::{CParser, MacroDefinition};
///
/// // Parse a simple object-like macro
/// let parser = CParser::new()?;
/// let ast = parser.parse("#define MAX 100\nint main() { return 0; }")?;
/// assert_eq!(ast.macros().len(), 1);
/// assert_eq!(ast.macros()[0].name(), "MAX");
/// assert!(ast.macros()[0].is_object_like());
///
/// // Parse a function-like macro
/// let ast2 = parser.parse("#define SQR(x) ((x) * (x))\nint main() { return 0; }")?;
/// assert_eq!(ast2.macros()[0].name(), "SQR");
/// assert!(ast2.macros()[0].is_function_like());
/// assert_eq!(ast2.macros()[0].parameters(), &["x"]);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Reference
///
/// K&R §4.11, ISO C99 §6.10.3
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDefinition {
    /// Macro name
    pub name: String,
    /// Parameters (empty for object-like macros)
    pub parameters: Vec<String>,
    /// Macro body (unparsed, tokenized without spaces)
    pub body: String,
}

impl MacroDefinition {
    /// Create a new object-like macro.
    pub fn new_object_like(name: String, body: String) -> Self {
        Self { name, parameters: vec![], body }
    }

    /// Create a new function-like macro.
    pub fn new_function_like(name: String, parameters: Vec<String>, body: String) -> Self {
        Self { name, parameters, body }
    }

    /// Get the macro name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the macro parameters.
    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    /// Get the macro body.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Check if this is a function-like macro.
    pub fn is_function_like(&self) -> bool {
        !self.parameters.is_empty()
    }

    /// Check if this is an object-like macro.
    pub fn is_object_like(&self) -> bool {
        self.parameters.is_empty()
    }
}

impl Ast {
    /// Create a new empty AST.
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            typedefs: Vec::new(),
            structs: Vec::new(),
            macros: Vec::new(),
            variables: Vec::new(),
            enums: Vec::new(),
            classes: Vec::new(),
            namespaces: Vec::new(),
        }
    }

    /// Get the functions in the AST.
    pub fn functions(&self) -> &[Function] {
        &self.functions
    }

    /// Get mutable access to functions (DECY-221: for CUDA qualifier post-processing).
    pub fn functions_mut(&mut self) -> &mut [Function] {
        &mut self.functions
    }

    /// Add a function to the AST.
    pub fn add_function(&mut self, function: Function) {
        self.functions.push(function);
    }

    /// Get the typedefs in the AST.
    pub fn typedefs(&self) -> &[Typedef] {
        &self.typedefs
    }

    /// Add a typedef to the AST.
    pub fn add_typedef(&mut self, typedef: Typedef) {
        self.typedefs.push(typedef);
    }

    /// Get the structs in the AST.
    pub fn structs(&self) -> &[Struct] {
        &self.structs
    }

    /// Add a struct to the AST.
    /// Deduplicates by name to avoid duplicate definitions from system includes.
    pub fn add_struct(&mut self, struct_def: Struct) {
        // Deduplicate: don't add if a struct with the same name already exists
        if !self.structs.iter().any(|s| s.name() == struct_def.name()) {
            self.structs.push(struct_def);
        }
    }

    /// Get the macro definitions in the AST.
    pub fn macros(&self) -> &[MacroDefinition] {
        &self.macros
    }

    /// Add a macro definition to the AST.
    pub fn add_macro(&mut self, macro_def: MacroDefinition) {
        self.macros.push(macro_def);
    }

    /// Get the variables in the AST.
    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }

    /// Add a variable to the AST.
    pub fn add_variable(&mut self, variable: Variable) {
        self.variables.push(variable);
    }

    /// Get the enums in the AST.
    pub fn enums(&self) -> &[Enum] {
        &self.enums
    }

    /// Add an enum to the AST.
    pub fn add_enum(&mut self, enum_def: Enum) {
        self.enums.push(enum_def);
    }

    /// Get the C++ classes in the AST (DECY-200).
    pub fn classes(&self) -> &[Class] {
        &self.classes
    }

    /// Add a C++ class to the AST (DECY-200).
    pub fn add_class(&mut self, class: Class) {
        if !self.classes.iter().any(|c| c.name == class.name) {
            self.classes.push(class);
        }
    }

    /// Get the C++ namespaces in the AST (DECY-201).
    pub fn namespaces(&self) -> &[Namespace] {
        &self.namespaces
    }

    /// Add a C++ namespace to the AST (DECY-201).
    pub fn add_namespace(&mut self, ns: Namespace) {
        self.namespaces.push(ns);
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::new()
    }
}

/// CUDA function qualifier (DECY-199).
///
/// Represents `__global__`, `__device__`, `__host__`, or combined qualifiers
/// extracted from CUDA source files via clang-sys cursor attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CudaQualifier {
    /// `__global__` - kernel entry point, callable from host, runs on device
    Global,
    /// `__device__` - callable from device only, runs on device
    Device,
    /// `__host__` - callable from host only, runs on host (default)
    Host,
    /// `__host__ __device__` - callable from both host and device
    HostDevice,
}

/// Represents a C function.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Return type
    pub return_type: Type,
    /// Parameters
    pub parameters: Vec<Parameter>,
    /// Function body (statements)
    pub body: Vec<Statement>,
    /// CUDA qualifier (DECY-199), None for plain C/C++ functions
    pub cuda_qualifier: Option<CudaQualifier>,
}

impl Function {
    /// Create a new function.
    pub fn new(name: String, return_type: Type, parameters: Vec<Parameter>) -> Self {
        Self { name, return_type, parameters, body: Vec::new(), cuda_qualifier: None }
    }

    /// Create a new function with body.
    pub fn new_with_body(
        name: String,
        return_type: Type,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
    ) -> Self {
        Self { name, return_type, parameters, body, cuda_qualifier: None }
    }
}

/// Represents a C type.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)] // TypeAlias is a meaningful variant name
pub enum Type {
    /// void
    Void,
    /// C99 _Bool type (maps to bool in Rust)
    Bool,
    /// int
    Int,
    /// unsigned int (DECY-158)
    UnsignedInt,
    /// float
    Float,
    /// double
    Double,
    /// char (unsigned char or plain char → u8)
    Char,
    /// signed char → i8 (DECY-250)
    SignedChar,
    /// Pointer to a type
    Pointer(Box<Type>),
    /// Struct type (e.g., struct Point)
    Struct(String),
    /// Function pointer type (e.g., int (*callback)(int))
    FunctionPointer {
        /// Parameter types
        param_types: Vec<Type>,
        /// Return type
        return_type: Box<Type>,
    },
    /// Array type (e.g., `int arr[10]`)
    /// For typedef assertions like: `typedef char check[sizeof(int) == 4 ? 1 : -1]`
    Array {
        /// Element type
        element_type: Box<Type>,
        /// Array size (None for unknown/expression-based size)
        size: Option<i64>,
    },
    /// Type alias (typedef) - preserves the alias name
    /// DECY-172: Used for size_t, ssize_t, ptrdiff_t, etc.
    TypeAlias(String),
}

/// Represents a function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: Type,
    /// Whether the pointee type is const (for pointer params like `const char*`)
    /// DECY-135: Track const qualifier to enable const char* → &str transformation
    pub is_pointee_const: bool,
}

impl Parameter {
    /// Create a new parameter.
    pub fn new(name: String, param_type: Type) -> Self {
        Self { name, param_type, is_pointee_const: false }
    }

    /// Create a new parameter with const pointee information.
    /// DECY-135: Used for const char* parameters
    pub fn new_with_const(name: String, param_type: Type, is_pointee_const: bool) -> Self {
        Self { name, param_type, is_pointee_const }
    }

    /// Check if this parameter is a function pointer.
    pub fn is_function_pointer(&self) -> bool {
        matches!(self.param_type, Type::FunctionPointer { .. })
    }

    /// Check if this parameter is a const char pointer (const char*).
    ///
    /// DECY-135: Now properly checks if pointee is const-qualified.
    /// Returns `true` only for `const char*` parameters, not `char*`.
    pub fn is_const_char_pointer(&self) -> bool {
        self.is_pointee_const
            && matches!(self.param_type, Type::Pointer(ref inner) if matches!(**inner, Type::Char))
    }

    /// Check if this parameter is any char pointer (char* or const char*).
    pub fn is_char_pointer(&self) -> bool {
        matches!(self.param_type, Type::Pointer(ref inner) if matches!(**inner, Type::Char))
    }
}
