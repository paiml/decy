    fn append_signature_return_type(
        &self,
        sig: &mut String,
        func: &HirFunction,
        output_param_type: Option<&HirType>,
        output_is_fallible: bool,
        annotated_sig: &AnnotatedSignature,
    ) {
        // Special handling for main function (DECY-AUDIT-001)
        // C's int main() must become Rust's fn main() (no return type)
        // Rust's entry point returns () and uses std::process::exit(N) for exit codes
        if func.name() == "main" && matches!(func.return_type(), HirType::Int) {
            return;
        }

        // DECY-084 GREEN: Generate return type considering output parameters
        // Priority: output param type > original return type
        if let Some(out_type) = output_param_type {
            let out_type_str = Self::map_type(out_type);
            if output_is_fallible {
                sig.push_str(&format!(" -> Result<{}, i32>", out_type_str));
            } else {
                sig.push_str(&format!(" -> {}", out_type_str));
            }
        } else {
            // DECY-142: Check if function returns malloc'd array → use Vec<T>
            if let Some(vec_element_type) = self.detect_vec_return(func) {
                let element_type_str = Self::map_type(&vec_element_type);
                sig.push_str(&format!(" -> Vec<{}>", element_type_str));
            } else {
                // Generate return type with lifetime annotation (skip for void)
                if !matches!(&annotated_sig.return_type, AnnotatedType::Simple(HirType::Void)) {
                    let return_type_str = self.annotated_type_to_string(&annotated_sig.return_type);
                    sig.push_str(&format!(" -> {}", return_type_str));
                }
            }
        }
    }

    fn generate_annotated_param(
        &self,
        p: &decy_ownership::lifetime_gen::AnnotatedParameter,
        func: Option<&HirFunction>,
    ) -> String {
        // Check if this is a slice parameter (Reference to Array with size=None)
        let is_slice = match &p.param_type {
            AnnotatedType::Reference { inner, .. } => match &**inner {
                AnnotatedType::Simple(HirType::Array { size, .. }) => size.is_none(),
                _ => false,
            },
            _ => false,
        };

        if is_slice {
            // DECY-072: Slices don't need 'mut' prefix or explicit lifetimes
            let type_str = match &p.param_type {
                AnnotatedType::Reference { inner, mutable, .. } => {
                    if let AnnotatedType::Simple(HirType::Array { element_type, .. }) =
                        &**inner
                    {
                        if *mutable {
                            format!("&mut [{}]", Self::map_type(element_type))
                        } else {
                            format!("&[{}]", Self::map_type(element_type))
                        }
                    } else {
                        self.annotated_type_to_string(&p.param_type)
                    }
                }
                _ => self.annotated_type_to_string(&p.param_type),
            };
            return format!("{}: {}", p.name, type_str);
        }

        // DECY-111: Transform pointer parameters to mutable references
        // DECY-123: Skip transformation if pointer arithmetic is used
        if let AnnotatedType::Simple(HirType::Pointer(inner)) = &p.param_type {
            return self.generate_annotated_pointer_param(&p.name, inner, func);
        }

        // DECY-196: Handle unsized array parameters → slice references
        if let AnnotatedType::Simple(HirType::Array { element_type, size: None }) =
            &p.param_type
        {
            let element_str = Self::map_type(element_type);
            return format!("{}: &mut [{}]", p.name, element_str);
        }

        // DECY-041: Add mut for all non-slice parameters to match C semantics
        format!("mut {}: {}", p.name, self.annotated_type_to_string(&p.param_type))
    }

    pub fn annotated_type_to_string(&self, annotated_type: &AnnotatedType) -> String {
        match annotated_type {
            AnnotatedType::Simple(hir_type) => Self::map_type(hir_type),
            AnnotatedType::Reference { inner, mutable, lifetime } => {
                // DECY-072: Special case for slices: &Vec<T> → &[T]
                // Check if inner is a Vec type
                if let AnnotatedType::Simple(HirType::Vec(element_type)) = &**inner {
                    let element_str = Self::map_type(element_type);
                    if *mutable {
                        return format!("&mut [{}]", element_str);
                    } else {
                        return format!("&[{}]", element_str);
                    }
                }

                let mut result = String::from("&");

                // Add lifetime if present
                if let Some(lt) = lifetime {
                    result.push_str(&lt.name);
                    result.push(' ');
                }

                // Add mutability
                if *mutable {
                    result.push_str("mut ");
                }

                // Add inner type
                result.push_str(&self.annotated_type_to_string(inner));

                result
            }
        }
    }
