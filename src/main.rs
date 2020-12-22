extern crate lalrpop_util;

extern crate global_counter;

mod ast;
mod collapse_automata;
mod compute_consts;
mod errors;
mod expand_fn;
mod flatten;
mod interpreter;
mod optimization;
mod parser_wrapper;
mod typed_ast;
mod typing;
mod viz;
use collapse_automata::collapse_automata;
use compute_consts::compute_consts;
use docopt::Docopt;
use errors::TinyjazzError;
use expand_fn::expand_functions;
use flatten::flatten;
use interpreter::interprete;
use optimization::{make_graph, ProgramGraph};
use parser_wrapper::parse;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, process::exit};
use typing::type_prog;
//Docopt generates a CLI automatically from this usage string. Pretty amazing.
const USAGE: &'static str = include_str!("USAGE.docopt");

#[derive(Debug, Deserialize)]
struct Args {
    arg_file: String,
    flag_version: bool,
    flag_dot: bool,
    flag_print: bool,
    flag_i: Option<String>,
    flag_s: Option<usize>,
}

// fn print_expr(expr: &ast::Expr) -> String {
//     if let ast::Expr::Var(v) = expr {
//         format!("{}", v.value)
//     } else {
//         format!("{:?}", expr)
//     }
// }

// fn print_stat(stat: &ast::Statement) {
//     match stat {
//         ast::Statement::Assign(vec) => {
//             for v in vec {
//                 println!("      {} = {}", v.var.value, print_expr(&v.expr.value));
//             }
//         }
//         ast::Statement::If(a) => {
//             println!("      {:?}", a);
//         }
//         ast::Statement::FnAssign(a) => {
//             println!("      {:?}", a);
//         }
//     }
// }

// pub fn print_prog(prog: &ast::Program) {
//     for (_, modules) in &prog.modules {
//         println!("{} : \n\n", modules.name);
//         for (_, node) in &modules.nodes {
//             println!("  {} : \n\n", node.name.value);
//             for stat in &node.statements {
//                 print_stat(stat)
//             }
//             println!("\n\n  transitions : ");
//             for (expr, a, _b) in &node.transitions {
//                 println!("  |{} -> {}", print_expr(&expr.value), a.value);
//             }
//         }
//     }
// }
//println!("{:#?}", expr);
fn process_file(path: PathBuf) -> Result<ProgramGraph, TinyjazzError> {
    let (mut prog, files) = parse(path)?;
    compute_consts(&mut prog).map_err(|e| (e, files.clone()))?;
    collapse_automata(&mut prog).map_err(|e| (e, files.clone()))?;
    //print_prog(&prog);
    flatten(&mut prog).map_err(|e| (e, files.clone()))?;
    let mut type_map = HashMap::new();
    expand_functions(&mut prog, &mut type_map).map_err(|e| (e, files.clone()))?;
    prog.functions = HashMap::new(); //the functions are no longer useful
                                     //at this point, the ast is ready to be typed.
    let prog = type_prog(prog, type_map).map_err(|e| (e, files.clone()))?;
    let graph = make_graph(&prog).map_err(|e| (e, files.clone()))?;
    Ok(graph)
}

fn run_interpreter(graph: &ProgramGraph, steps: usize, input_script_path: Option<String>) {
    for outputs in interprete(graph, input_script_path).take(steps) {
        println!("{:?}", outputs);
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("tinyjazz version 0.0.1");
        return;
    }
    let prog_result = process_file(args.arg_file.into());
    match prog_result {
        Err(err) => {
            err.print().unwrap();
            exit(1)
        }
        Ok(prog) => {
            if args.flag_print {
                println!("{:#?}", prog)
            }
            if args.flag_dot {
                viz::render(&prog);
            }
            if let Some(steps) = args.flag_s {
                run_interpreter(&prog, steps, args.flag_i)
            }
        }
    }
}
