mod tokenizer;
use tokenizer::{tokenize_recursive,print_tokens};
mod parser;
mod operation_enums;

use parser::parse_update_function;

fn main() {
    let formula : String = "!{x}: ~ EX {x}".to_string();
    let tokens = match tokenize_recursive(&mut formula.chars().peekable(), true) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };
    print_tokens(&tokens);

    match parse_update_function(&tokens) {
        Ok(tree) => {
            let t = *tree;
            println!("{}", t.subform_str);
            println!("{}", t.height)
        },
        Err(message) => println!("{}", message),
    }
}
