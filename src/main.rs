mod types;
mod clause;
mod solver;
mod parser;

use parser::DimacsParser;
use solver::Solver;
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: cargo run --release <path_to_benchmark.cnf>");
        std::process::exit(1);
    }

    let file_path = &args[1];
    println!("Loading DIMACS benchmark from: {}", file_path);

    let start_parse = Instant::now();
    let (num_vars, clauses) = DimacsParser::parse_file(file_path);
    let parse_duration = start_parse.elapsed();

    println!("Variables : {}", num_vars);
    println!("Clauses   : {}", clauses.len());
    println!("Parsed in : {:?}", parse_duration);

    let mut solver = Solver::new(num_vars);
    for clause in clauses {
        solver.add_clause(clause, false);
    }

    println!("Running CDCL Solver Engine...");
    let start_solve = Instant::now();
    let is_sat = solver.solve();
    let solve_duration = start_solve.elapsed();

    println!("==========================================");
    println!("  ELITE CDCL SAT SOLVER - RUST EDITION    ");
    println!("==========================================");
    println!("Formula Satisfiable : {}", is_sat);
    println!("Solver Time         : {:?}", solve_duration);
    println!("==========================================");
}