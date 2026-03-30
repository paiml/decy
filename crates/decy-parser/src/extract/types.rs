//! Type, struct, enum, typedef, variable, and macro extraction from clang cursors.

#[allow(non_upper_case_globals)]

use crate::ast_types::*;
use clang_sys::*;
use std::ffi::CStr;
use std::ptr;


use super::visit_statement;
use super::expressions::visit_variable_initializer;

pub(crate) fn extract_function(cursor: CXCursor) -> Option<Function> {
    // SAFETY: Getting cursor spelling (function name)
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // SAFETY: Getting return type
    let cx_type = unsafe { clang_getCursorType(cursor) };
    let return_cx_type = unsafe { clang_getResultType(cx_type) };
    let return_type = convert_type(return_cx_type)?;

    // Extract parameters
    let num_args = unsafe { clang_Cursor_getNumArguments(cursor) };
    let mut parameters = Vec::new();

    for i in 0..num_args {
        // SAFETY: Getting argument cursor
        let arg_cursor = unsafe { clang_Cursor_getArgument(cursor, i as u32) };

        // Get parameter name
        let param_name_cxstring = unsafe { clang_getCursorSpelling(arg_cursor) };
        let param_name = unsafe {
            let c_str = CStr::from_ptr(clang_getCString(param_name_cxstring));
            let name = c_str.to_string_lossy().into_owned();
            clang_disposeString(param_name_cxstring);
            name
        };

        // Get parameter type
        let param_cx_type = unsafe { clang_getCursorType(arg_cursor) };
        if let Some(param_type) = convert_type(param_cx_type) {
            // DECY-135: Check if this is a pointer with const-qualified pointee
            let is_pointee_const = unsafe {
                if param_cx_type.kind == clang_sys::CXType_Pointer {
                    let pointee = clang_sys::clang_getPointeeType(param_cx_type);
                    clang_isConstQualifiedType(pointee) != 0
                } else {
                    false
                }
            };
            parameters.push(Parameter::new_with_const(param_name, param_type, is_pointee_const));
        }
    }

    // Extract function body by visiting children
    let mut body = Vec::new();
    let body_ptr = &mut body as *mut Vec<Statement>;

    unsafe {
        clang_visitChildren(cursor, visit_statement, body_ptr as CXClientData);
    }

    // DECY-199: Detect CUDA qualifiers by visiting function attributes
    let mut cuda_qualifier = None;
    let cuda_qual_ptr = &mut cuda_qualifier as *mut Option<CudaQualifier>;
    unsafe {
        clang_visitChildren(cursor, visit_cuda_attrs, cuda_qual_ptr as CXClientData);
    }

    let mut func = Function::new_with_body(name, return_type, parameters, body);
    func.cuda_qualifier = cuda_qualifier;
    Some(func)
}

/// Visitor callback to detect CUDA function attributes (DECY-199).
///
/// Checks for CXCursor_CUDAGlobalAttr (414), CUDADeviceAttr (413),
/// CUDAHostAttr (415) among the function's children.
extern "C" fn visit_cuda_attrs(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    let kind = unsafe { clang_getCursorKind(cursor) };
    let qual_ptr = client_data as *mut Option<CudaQualifier>;

    // clang-sys cursor kinds for CUDA attributes:
    // CXCursor_CUDADeviceAttr = 413
    // CXCursor_CUDAGlobalAttr = 414
    // CXCursor_CUDAHostAttr = 415
    match kind {
        414 => {
            // __global__
            unsafe { *qual_ptr = Some(CudaQualifier::Global) };
        }
        413 => {
            // __device__ - could combine with __host__
            unsafe {
                *qual_ptr = match *qual_ptr {
                    Some(CudaQualifier::Host) => Some(CudaQualifier::HostDevice),
                    _ => Some(CudaQualifier::Device),
                };
            }
        }
        415 => {
            // __host__ - could combine with __device__
            unsafe {
                *qual_ptr = match *qual_ptr {
                    Some(CudaQualifier::Device) => Some(CudaQualifier::HostDevice),
                    _ => Some(CudaQualifier::Host),
                };
            }
        }
        _ => {}
    }

    CXChildVisit_Continue
}

/// Extract a C++ class from a CXCursor_ClassDecl cursor (DECY-200).
///
/// Visits class children to find fields, methods, constructors, and destructors.
/// Maps to AST `Class` struct for downstream conversion to Rust struct + impl.
pub(crate) fn extract_class(cursor: CXCursor) -> Option<Class> {
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let n = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        n
    };

    // Skip anonymous classes
    if name.is_empty() {
        return None;
    }

    let mut class = Class::new(name);
    let class_ptr = &mut class as *mut Class;

    unsafe {
        clang_visitChildren(cursor, visit_class_children, class_ptr as CXClientData);
    }

    Some(class)
}

/// Visitor callback for C++ class children (DECY-200).
///
/// Handles: CXCursor_FieldDecl, CXCursor_CXXMethod, CXCursor_Constructor,
/// CXCursor_Destructor, CXCursor_CXXAccessSpecifier.
extern "C" fn visit_class_children(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    let class = unsafe { &mut *(client_data as *mut Class) };
    let kind = unsafe { clang_getCursorKind(cursor) };

    match kind {
        // CXCursor_FieldDecl = 6
        6 => {
            let field_name_cx = unsafe { clang_getCursorSpelling(cursor) };
            let field_name = unsafe {
                let c_str = CStr::from_ptr(clang_getCString(field_name_cx));
                let n = c_str.to_string_lossy().into_owned();
                clang_disposeString(field_name_cx);
                n
            };
            let field_type = unsafe { clang_getCursorType(cursor) };
            if let Some(ft) = convert_type(field_type) {
                class.fields.push(StructField::new(field_name, ft));
            }
        }
        // CXCursor_CXXMethod = 21
        21 => {
            if let Some(func) = extract_function(cursor) {
                let is_const = unsafe { clang_CXXMethod_isConst(cursor) != 0 };
                let is_static = unsafe { clang_CXXMethod_isStatic(cursor) != 0 };
                let is_virtual = unsafe { clang_CXXMethod_isVirtual(cursor) != 0 };
                class.methods.push(Method {
                    function: func,
                    access: AccessSpecifier::Public, // simplified: default to public
                    is_const,
                    is_static,
                    is_virtual,
                });
            }
        }
        // CXCursor_Constructor = 24
        24 => {
            // Extract constructor parameters
            let num_args = unsafe { clang_Cursor_getNumArguments(cursor) };
            let mut params = Vec::new();
            for i in 0..num_args {
                let arg_cursor = unsafe { clang_Cursor_getArgument(cursor, i as u32) };
                let param_name_cx = unsafe { clang_getCursorSpelling(arg_cursor) };
                let param_name = unsafe {
                    let c_str = CStr::from_ptr(clang_getCString(param_name_cx));
                    let n = c_str.to_string_lossy().into_owned();
                    clang_disposeString(param_name_cx);
                    n
                };
                let param_type = unsafe { clang_getCursorType(arg_cursor) };
                if let Some(pt) = convert_type(param_type) {
                    params.push(Parameter::new(param_name, pt));
                }
            }
            // Store first constructor's params (primary constructor)
            if class.constructor_params.is_empty() {
                class.constructor_params = params;
            }
        }
        // CXCursor_Destructor = 25
        25 => {
            class.has_destructor = true;
        }
        _ => {}
    }

    CXChildVisit_Continue
}

/// Extract typedef information from a clang cursor.
/// Returns (Option<Typedef>, Option<Struct>) - struct is Some when typedef is for anonymous struct.
pub(crate) fn extract_typedef(cursor: CXCursor) -> (Option<Typedef>, Option<Struct>) {
    // SAFETY: Getting typedef name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // SAFETY: Getting underlying type of typedef
    let cx_type = unsafe { clang_getTypedefDeclUnderlyingType(cursor) };

    // DECY-147: Check if underlying type is anonymous struct
    // Anonymous struct pattern: typedef struct { ... } Name;
    let canonical = unsafe { clang_getCanonicalType(cx_type) };
    if canonical.kind == CXType_Record {
        let decl = unsafe { clang_getTypeDeclaration(canonical) };
        let struct_name_cxstring = unsafe { clang_getCursorSpelling(decl) };
        let struct_name = unsafe {
            let c_str = CStr::from_ptr(clang_getCString(struct_name_cxstring));
            let sn = c_str.to_string_lossy().into_owned();
            clang_disposeString(struct_name_cxstring);
            sn
        };

        // If struct name is empty, this is an anonymous struct typedef
        if struct_name.is_empty() {
            // Extract struct fields from the declaration
            let mut fields = Vec::new();
            let fields_ptr = &mut fields as *mut Vec<StructField>;

            unsafe {
                clang_visitChildren(decl, visit_struct_fields, fields_ptr as CXClientData);
            }

            // Return struct with typedef name, no typedef needed
            return (None, Some(Struct::new(name, fields)));
        }
    }

    let underlying_type = convert_type(cx_type);
    match underlying_type {
        Some(ut) => (Some(Typedef::new(name, ut)), None),
        None => (None, None),
    }
}

/// Extract struct information from a clang cursor.
pub(crate) fn extract_struct(cursor: CXCursor) -> Option<Struct> {
    // SAFETY: Getting struct name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // Skip anonymous structs
    if name.is_empty() {
        return None;
    }

    // Extract struct fields by visiting children
    let mut fields = Vec::new();
    let fields_ptr = &mut fields as *mut Vec<StructField>;

    unsafe {
        clang_visitChildren(cursor, visit_struct_fields, fields_ptr as CXClientData);
    }

    Some(Struct::new(name, fields))
}

/// DECY-240: Extract enum information from a clang cursor.
///
/// Extracts C enum declarations, including explicit values.
pub(crate) fn extract_enum(cursor: CXCursor) -> Option<Enum> {
    // SAFETY: Getting enum name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // Extract enum variants by visiting children
    let mut variants: Vec<EnumVariant> = Vec::new();
    let variants_ptr = &mut variants as *mut Vec<EnumVariant>;

    // Visitor callback for enum constants
    extern "C" fn visit_enum_constants(
        cursor: CXCursor,
        _parent: CXCursor,
        client_data: CXClientData,
    ) -> CXChildVisitResult {
        let variants = unsafe { &mut *(client_data as *mut Vec<EnumVariant>) };

        // SAFETY: Getting cursor kind
        let kind = unsafe { clang_getCursorKind(cursor) };

        // CXCursor_EnumConstantDecl = 7
        if kind == 7 {
            // Get variant name
            let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
            let variant_name = unsafe {
                let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
                let name = c_str.to_string_lossy().into_owned();
                clang_disposeString(name_cxstring);
                name
            };

            // Get variant value
            let value = unsafe { clang_getEnumConstantDeclValue(cursor) };

            variants.push(EnumVariant::new(variant_name, Some(value)));
        }

        CXChildVisit_Continue
    }

    unsafe {
        clang_visitChildren(cursor, visit_enum_constants, variants_ptr as CXClientData);
    }

    // Only return if there are variants (skip empty enums)
    if variants.is_empty() {
        return None;
    }

    Some(Enum::new(name, variants))
}

/// Extract macro definition from a clang cursor.
///
/// Extract variable declaration information from a clang cursor.
///
/// Extracts global and local variable declarations, including function pointers.
///
/// # Examples
///
/// Simple: `int x;`
/// Function pointer: `int (*callback)(int);`
pub(crate) fn extract_variable(cursor: CXCursor) -> Option<Variable> {
    // SAFETY: Getting cursor spelling (variable name)
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // SAFETY: Getting variable type
    let cx_type = unsafe { clang_getCursorType(cursor) };
    let var_type = convert_type(cx_type)?;

    // Extract storage class specifiers
    // CX_StorageClass values (from clang-sys):
    // CX_SC_Invalid = 0, CX_SC_None = 1, CX_SC_Extern = 2, CX_SC_Static = 3,
    // CX_SC_PrivateExtern = 4, CX_SC_OpenCLWorkGroupLocal = 5,
    // CX_SC_Auto = 6, CX_SC_Register = 7
    let storage_class = unsafe { clang_Cursor_getStorageClass(cursor) };
    let is_static = storage_class == 3; // CX_SC_Static
    let is_extern = storage_class == 2; // CX_SC_Extern

    // Check if type is const-qualified
    let is_const = unsafe { clang_isConstQualifiedType(cx_type) != 0 };

    // Extract initializer by visiting children
    let mut initializer: Option<Expression> = None;
    let initializer_ptr = &mut initializer as *mut Option<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_variable_initializer, initializer_ptr as CXClientData);
    }

    Some(Variable::new_with_storage_class(
        name,
        var_type,
        initializer,
        is_static,
        is_extern,
        is_const,
    ))
}

///
/// Object-like: `#define MAX 100`
/// Function-like: `#define SQR(x) ((x) * (x))`
pub(crate) fn extract_macro(cursor: CXCursor) -> Option<MacroDefinition> {
    // SAFETY: Getting macro name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // Skip empty macro names
    if name.is_empty() {
        return None;
    }

    // Get macro body using clang_Cursor_isMacroFunctionLike and clang token APIs
    // For now, we'll check if it's function-like and extract tokens
    let is_function_like = unsafe { clang_sys::clang_Cursor_isMacroFunctionLike(cursor) } != 0;

    // Get the source range and tokens for the macro
    let range = unsafe { clang_getCursorExtent(cursor) };
    let tu = unsafe { clang_Cursor_getTranslationUnit(cursor) };

    let mut tokens: *mut CXToken = ptr::null_mut();
    let mut num_tokens: u32 = 0;

    unsafe {
        clang_tokenize(tu, range, &mut tokens, &mut num_tokens);
    }

    // Extract macro body from tokens
    // Skip the first token (macro name) and extract the rest
    let mut parameters = Vec::new();
    let mut body_tokens = Vec::new();
    let mut in_params = false;

    for i in 0..num_tokens {
        let token = unsafe { *tokens.offset(i as isize) };
        let token_kind = unsafe { clang_getTokenKind(token) };
        let token_spelling = unsafe { clang_getTokenSpelling(tu, token) };
        let token_str = unsafe {
            let c_str = CStr::from_ptr(clang_getCString(token_spelling));
            let s = c_str.to_string_lossy().into_owned();
            clang_disposeString(token_spelling);
            s
        };

        // Skip the macro name (first token)
        if i == 0 {
            continue;
        }

        // Check for parameter list (function-like macros)
        if is_function_like && i == 1 && token_str == "(" {
            in_params = true;
            continue;
        }

        if in_params {
            if token_str == ")" {
                in_params = false;
                continue;
            } else if token_str != ","
                && (token_kind == CXToken_Identifier || token_kind == CXToken_Keyword)
            {
                // Accept both identifiers and keywords as parameter names
                // C allows keywords in macro parameter names since they're in macro scope
                parameters.push(token_str);
            }
        } else {
            body_tokens.push(token_str);
        }
    }

    // Clean up tokens
    unsafe {
        clang_disposeTokens(tu, tokens, num_tokens);
    }

    // Join body tokens without spaces (preserving original formatting)
    let body = body_tokens.join("");

    if is_function_like {
        Some(MacroDefinition::new_function_like(name, parameters, body))
    } else {
        Some(MacroDefinition::new_object_like(name, body))
    }
}

/// Visitor callback for struct fields.
///
/// # Safety
///
/// This function is called by clang_visitChildren and must follow C calling conventions.
#[allow(non_upper_case_globals)]
pub(crate) extern "C" fn visit_struct_fields(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    // SAFETY: Converting client data back to fields vector pointer
    let fields = unsafe { &mut *(client_data as *mut Vec<StructField>) };

    // SAFETY: Getting cursor kind
    let kind = unsafe { clang_getCursorKind(cursor) };

    if kind == CXCursor_FieldDecl {
        // Get field name
        let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
        let name = unsafe {
            let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
            let name = c_str.to_string_lossy().into_owned();
            clang_disposeString(name_cxstring);
            name
        };

        // Get field type
        let cx_type = unsafe { clang_getCursorType(cursor) };
        if let Some(field_type) = convert_type(cx_type) {
            fields.push(StructField::new(name, field_type));
        }
    }

    CXChildVisit_Continue
}

#[allow(non_upper_case_globals)]
pub(crate) fn convert_type(cx_type: CXType) -> Option<Type> {
    // SAFETY: Getting type kind
    match cx_type.kind {
        CXType_Void => Some(Type::Void),
        3 => Some(Type::Bool), // CXType_Bool = 3 — C99 _Bool
        CXType_Int => Some(Type::Int),
        CXType_UInt => Some(Type::UnsignedInt), // DECY-158: unsigned int → u32
        CXType_UChar => Some(Type::Char),       // unsigned char → u8 (DECY-057 fix)
        CXType_UShort => Some(Type::UnsignedInt), // unsigned short → u32 (safe approximation)
        CXType_ULong => Some(Type::UnsignedInt), // unsigned long → u32 (safe approximation)
        CXType_Short => Some(Type::Int),        // short → i32
        CXType_Long => Some(Type::Int),         // long → i32
        CXType_LongLong => Some(Type::Int),     // long long → i32 (simplified)
        CXType_ULongLong => Some(Type::UnsignedInt), // DECY-158: unsigned long long → u32
        CXType_Float => Some(Type::Float),
        CXType_Double => Some(Type::Double),
        23 => Some(Type::Double), // CXType_LongDouble → f64 (Rust has no long double)
        CXType_Char_S | CXType_Char_U => Some(Type::Char),
        14 => Some(Type::SignedChar), // CXType_SChar - explicitly signed char → i8 (DECY-250)
        CXType_Pointer => {
            // SAFETY: Getting pointee type
            let pointee = unsafe { clang_getPointeeType(cx_type) };

            // Check if the pointee is a function - this is a function pointer
            if pointee.kind == CXType_FunctionProto || pointee.kind == CXType_FunctionNoProto {
                // This is a function pointer type
                // Extract return type
                let return_cx_type = unsafe { clang_getResultType(pointee) };
                let return_type = convert_type(return_cx_type)?;

                // Extract parameter types
                let num_args = unsafe { clang_getNumArgTypes(pointee) };
                let mut param_types = Vec::new();

                for i in 0..num_args {
                    let arg_type = unsafe { clang_getArgType(pointee, i as u32) };
                    if let Some(param_type) = convert_type(arg_type) {
                        param_types.push(param_type);
                    }
                }

                return Some(Type::FunctionPointer {
                    param_types,
                    return_type: Box::new(return_type),
                });
            }

            // Regular pointer (not function pointer)
            convert_type(pointee).map(|t| Type::Pointer(Box::new(t)))
        }
        CXType_FunctionProto | CXType_FunctionNoProto => {
            // Function type (not a pointer to function, but the function type itself)
            // This can occur in typedefs like: typedef int Func(int);
            // Extract return type
            let return_cx_type = unsafe { clang_getResultType(cx_type) };
            let return_type = convert_type(return_cx_type)?;

            // Extract parameter types
            let num_args = unsafe { clang_getNumArgTypes(cx_type) };
            let mut param_types = Vec::new();

            for i in 0..num_args {
                let arg_type = unsafe { clang_getArgType(cx_type, i as u32) };
                if let Some(param_type) = convert_type(arg_type) {
                    param_types.push(param_type);
                }
            }

            Some(Type::FunctionPointer { param_types, return_type: Box::new(return_type) })
        }
        CXType_Record => {
            // SAFETY: Getting type declaration to extract struct name
            let decl = unsafe { clang_getTypeDeclaration(cx_type) };
            let name_cxstring = unsafe { clang_getCursorSpelling(decl) };
            let name = unsafe {
                let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
                let struct_name = c_str.to_string_lossy().into_owned();
                clang_disposeString(name_cxstring);
                struct_name
            };
            Some(Type::Struct(name))
        }
        CXType_Elaborated => {
            // Elaborated types wrap other types (e.g., "struct Point" wraps the Record type)
            // Get the canonical type to unwrap it
            let canonical = unsafe { clang_getCanonicalType(cx_type) };
            convert_type(canonical)
        }
        CXType_Typedef => {
            // DECY-172: Get typedef name first to check for known type aliases
            let typedef_decl = unsafe { clang_getTypeDeclaration(cx_type) };
            let typedef_name_cxstring = unsafe { clang_getCursorSpelling(typedef_decl) };
            let typedef_name = unsafe {
                let c_str = CStr::from_ptr(clang_getCString(typedef_name_cxstring));
                let tn = c_str.to_string_lossy().into_owned();
                clang_disposeString(typedef_name_cxstring);
                tn
            };

            // DECY-172: Preserve size_t, ssize_t, ptrdiff_t as TypeAlias
            // These need to map to usize/isize in Rust for compatibility with .len() etc.
            match typedef_name.as_str() {
                "size_t" | "ssize_t" | "ptrdiff_t" => {
                    return Some(Type::TypeAlias(typedef_name));
                }
                _ => {}
            }

            // DECY-147: For typedefs to anonymous structs, use typedef name as struct name
            // Example: typedef struct { int x; } Point; → Type::Struct("Point")
            let canonical = unsafe { clang_getCanonicalType(cx_type) };

            // Check if this is a typedef to an anonymous struct
            if canonical.kind == CXType_Record {
                let decl = unsafe { clang_getTypeDeclaration(canonical) };
                let struct_name_cxstring = unsafe { clang_getCursorSpelling(decl) };
                let struct_name = unsafe {
                    let c_str = CStr::from_ptr(clang_getCString(struct_name_cxstring));
                    let sn = c_str.to_string_lossy().into_owned();
                    clang_disposeString(struct_name_cxstring);
                    sn
                };

                // If struct is anonymous, use the typedef name instead
                if struct_name.is_empty() {
                    return Some(Type::Struct(typedef_name));
                }
            }

            // Default: recursively convert the canonical type
            convert_type(canonical)
        }
        CXType_ConstantArray => {
            // Array type - extract element type and size
            let element_cx_type = unsafe { clang_getArrayElementType(cx_type) };
            let element_type = convert_type(element_cx_type)?;

            // Get array size
            let array_size = unsafe { clang_getArraySize(cx_type) };
            let size = if array_size >= 0 { Some(array_size) } else { None };

            Some(Type::Array { element_type: Box::new(element_type), size })
        }
        114 => {
            // CXType_IncompleteArray - flexible array member (C99 §6.7.2.1)
            // DECY-136: char data[] → Vec<u8>
            // Flexible array members have no size specified
            let element_cx_type = unsafe { clang_getArrayElementType(cx_type) };
            let element_type = convert_type(element_cx_type)?;

            // Generate as Array with size None (will be transformed to Vec in codegen)
            Some(Type::Array { element_type: Box::new(element_type), size: None })
        }
        106 => {
            // CXType_Enum - C enums are integers
            // DECY-240: Map enum types to i32 for Rust compatibility
            Some(Type::Int)
        }
        _ => None,
    }
}

