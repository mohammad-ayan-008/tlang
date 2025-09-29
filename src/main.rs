use std::{
    env,
    fs::read_to_string,
    io::{BufRead, Write, stdin, stdout},
    path::Path,
    process::exit,
};

use inkwell::{context::Context, targets::InitializationConfig};

use crate::{lexer::Scanner, parser::Parser, token::Token};

mod codegen;
mod expr;
mod lexer;
mod parser;
mod stmt;
mod token;
mod tokentype;

#[allow(warnings)]
fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args.len() > 2 {
        println!("Using : jlox [script]");
        exit(64);
    } else if args.len() == 2 {
        let path = args.get(1).unwrap();
        execute_file(&path);
    } else {
    }
}
fn execute_file(path: &String) {
    let data = read_to_string(path).unwrap();
    match run(data) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
}

fn run(bytes: String) -> Result<(), String> {
    let scanner: Scanner = Scanner::new(bytes);

    let tokens: Vec<Token> = scanner.scanTokens();

    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;

    inkwell::targets::Target::initialize_all(&InitializationConfig::default());

    let target_triple = inkwell::targets::TargetMachine::get_default_triple();
    let target = inkwell::targets::Target::from_triple(&target_triple)
        .expect("Could not create target from triple");

    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic", // CPU type
            "",        // features
            inkwell::OptimizationLevel::Default,
            inkwell::targets::RelocMode::PIC,
            inkwell::targets::CodeModel::Default,
        )
        .expect("Unable to create target machine");

    let context = Context::create();
    let mut codegen = codegen::Compiler::new(&context, "tasm");

    codegen.generate(statements);
    //println!("{}", codegen.module.print_to_string().to_string());
    target_machine
        .write_to_file(
            &codegen.module,
            inkwell::targets::FileType::Object,
            Path::new("output.o"),
        )
        .expect("Failed to write object file");
    //println!("{:#?}",tokens);
    Ok(())
}
