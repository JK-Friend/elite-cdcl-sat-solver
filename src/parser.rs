use crate::types::{Lit, Var};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DimacsParser;

impl DimacsParser {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> (usize, Vec<Vec<Lit>>) {
        let file = File::open(path).expect("Failed to open DIMACS benchmark file");
        let reader = BufReader::new(file);
        
        let mut num_vars = 0;
        let mut clauses = Vec::new();

        for line in reader.lines() {
            let line = line.expect("Failed to read line");
            let line = line.trim();
            
            // Skip empty lines and comment lines
            if line.is_empty() || line.starts_with('c') {
                continue;
            }
            
            // Parse problem header: p cnf <num_vars> <num_clauses>
            if line.starts_with("p cnf") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                num_vars = parts[2].parse().expect("Invalid number of variables in header");
                continue;
            }

            // Parse clause literals ending with 0
            let lits: Vec<Lit> = line
                .split_whitespace()
                .map(|s| s.parse::<i32>().expect("Invalid literal format"))
                .take_while(|&val| val != 0)
                .map(|val| {
                    let var = Var((val.abs() - 1) as u32);
                    let sign = val < 0;
                    Lit::new(var, sign)
                })
                .collect();

            if !lits.is_empty() {
                clauses.push(lits);
            }
        }

        (num_vars, clauses)
    }
}