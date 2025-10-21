use decy_parser::CParser;

fn main() {
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void* allocate(int size) {
            void* x;
            x = malloc(size);
            return x;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];
    
    println!("Function: {}", func.name);
    println!("Body length: {}", func.body.len());
    for (i, stmt) in func.body.iter().enumerate() {
        println!("[{}] {:?}", i, stmt);
    }
}
