    pub fn generate_annotated_signature(&self, sig: &AnnotatedSignature) -> String {
        self.generate_annotated_signature_with_func(sig, None)
    }

    pub fn generate_annotated_signature_with_func(
        &self,
        sig: &AnnotatedSignature,
        func: Option<&HirFunction>,
    ) -> String {
        // DECY-241: Rename functions that conflict with Rust macros/keywords
        let safe_name = match sig.name.as_str() {
            "write" => "c_write",
            "read" => "c_read",
            "type" => "c_type",
            "match" => "c_match",
            "self" => "c_self",
            "in" => "c_in",
            name => name,
        };
        let mut result = format!("fn {}", safe_name);

        // DECY-084/085: Detect output parameters for transformation
        let (skip_output_params, output_param_types, output_is_fallible) =
            Self::detect_output_params(func);

        // DECY-072: Check if we have any non-slice reference parameters that need lifetimes
        // Slices have elided lifetimes and don't need explicit lifetime parameters
        let has_non_slice_references = sig.parameters.iter().any(|p| {
            match &p.param_type {
                AnnotatedType::Reference { inner, .. } => {
                    // Check if this is NOT a slice (slice = Reference to Array with size=None)
                    !matches!(&**inner, AnnotatedType::Simple(HirType::Array { size: None, .. }))
                }
                _ => false,
            }
        });

        // Add lifetime parameters only if we have non-slice references
        if !sig.lifetimes.is_empty() && has_non_slice_references {
            let lifetime_params: Vec<String> =
                sig.lifetimes.iter().map(|lt| lt.name.clone()).collect();
            result.push_str(&format!("<{}>", lifetime_params.join(", ")));
        }

        // Add function parameters (DECY-084: filter out output params)
        result.push('(');
        let params: Vec<String> = sig
            .parameters
            .iter()
            .filter(|p| !skip_output_params.contains(&p.name))
            .map(|p| self.generate_annotated_param(p, func))
            .collect();
        result.push_str(&params.join(", "));
        result.push(')');

        // Generate return type
        self.append_annotated_return_type(
            &mut result,
            sig,
            func,
            &output_param_types,
            output_is_fallible,
        );

        result
    }

    fn append_annotated_return_type(
        &self,
        result: &mut String,
        sig: &AnnotatedSignature,
        func: Option<&HirFunction>,
        output_param_types: &[HirType],
        output_is_fallible: bool,
    ) {
        // Special handling for main function (DECY-AUDIT-001)
        let return_type_str = self.annotated_type_to_string(&sig.return_type);
        if sig.name == "main" && return_type_str == "i32" {
            return;
        }

        // DECY-084/085: Generate return type considering output parameters
        if !output_param_types.is_empty() {
            let out_type_str = if output_param_types.len() == 1 {
                Self::map_type(&output_param_types[0])
            } else {
                let type_strs: Vec<String> =
                    output_param_types.iter().map(Self::map_type).collect();
                format!("({})", type_strs.join(", "))
            };

            if output_is_fallible {
                result.push_str(&format!(" -> Result<{}, i32>", out_type_str));
            } else {
                result.push_str(&format!(" -> {}", out_type_str));
            }
        } else {
            // DECY-142: Check for Vec return type (malloc'd array returns)
            if let Some(f) = func {
                if let Some(vec_element_type) = self.detect_vec_return(f) {
                    let element_type_str = Self::map_type(&vec_element_type);
                    result.push_str(&format!(" -> Vec<{}>", element_type_str));
                    return;
                }
            }
            // Add return type if not void
            if return_type_str != "()" {
                result.push_str(&format!(" -> {}", return_type_str));
            }
        }
    }
