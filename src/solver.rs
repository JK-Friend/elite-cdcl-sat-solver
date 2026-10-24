use crate::types::{Lit, Var};
use crate::clause::{Clause, ClauseId};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LBool {
    False,
    True,
    Undef,
}

pub struct Solver {
    pub num_vars: usize,
    pub clauses: Vec<Clause>,
    watches: Vec<Vec<ClauseId>>,
    assignment: Vec<LBool>,
    trail: Vec<Lit>,
    trail_lim: Vec<usize>,
    reason: Vec<Option<ClauseId>>,
    level: Vec<usize>,
    
    // VSIDS & Conflict Learning State
    activity: Vec<f64>,
    var_inc: f64,
    watch_data: Vec<(usize, usize)>,

    // Advanced CDCL State (Restarts & Memory Management)
    conflicts: u64,
    next_restart: u64,
    learned_clauses_count: usize,
}

impl Solver {
    pub fn new(num_vars: usize) -> Self {
        Solver {
            num_vars,
            clauses: Vec::new(),
            watches: vec![Vec::new(); num_vars * 2 + 2],
            assignment: vec![LBool::Undef; num_vars],
            trail: Vec::with_capacity(num_vars),
            trail_lim: Vec::new(),
            reason: vec![None; num_vars],
            level: vec![0; num_vars],
            activity: vec![0.0; num_vars],
            var_inc: 1.0,
            watch_data: Vec::new(),
            conflicts: 0,
            next_restart: 100,
            learned_clauses_count: 0,
        }
    }

    pub fn add_clause(&mut self, lits: Vec<Lit>, is_learned: bool) {
        let cid = ClauseId(self.clauses.len());
        if lits.len() >= 2 {
            self.watches[lits[0].negate().index()].push(cid);
            self.watches[lits[1].negate().index()].push(cid);
            self.watch_data.push((0, 1));
        } else {
            self.watch_data.push((0, 0));
        }
        self.clauses.push(Clause::new(lits, is_learned));
    }

    #[inline]
    pub fn value_lit(&self, lit: Lit) -> LBool {
        let v = self.assignment[lit.var().0 as usize];
        if v == LBool::Undef {
            LBool::Undef
        } else if lit.sign() {
            if v == LBool::False { LBool::True } else { LBool::False }
        } else {
            v
        }
    }

    #[inline]
    fn bump_activity(&mut self, var: Var) {
        let idx = var.0 as usize;
        self.activity[idx] += self.var_inc;
        if self.activity[idx] > 1e100 {
            for a in &mut self.activity {
                *a *= 1e-100;
            }
            self.var_inc *= 1e-100;
        }
    }

    pub fn propagate(&mut self) -> Option<ClauseId> {
        let mut trail_head = 0;
        while trail_head < self.trail.len() {
            let p = self.trail[trail_head];
            trail_head += 1;

            let mut i = 0;
            let mut watches_list = std::mem::take(&mut self.watches[p.index()]);

            while i < watches_list.len() {
                let cid = watches_list[i];
                let clause = &self.clauses[cid.0];
                
                let (w0, w1) = self.watch_data[cid.0];
                let other_watch_idx = if clause.lits[w0] == p.negate() { w0 } else { w1 };
                
                if self.value_lit(clause.lits[other_watch_idx]) == LBool::True {
                    i += 1;
                    continue;
                }

                let mut found_new_watch = false;
                for k in 0..clause.len() {
                    if k != w0 && k != w1 && self.value_lit(clause.lits[k]) != LBool::False {
                        self.watch_data[cid.0] = if other_watch_idx == w0 { (k, w1) } else { (w0, k) };
                        let new_watch_lit = clause.lits[k].negate();
                        self.watches[new_watch_lit.index()].push(cid);
                        found_new_watch = true;
                        break;
                    }
                }

                if found_new_watch {
                    watches_list.swap_remove(i);
                } else {
                    if self.value_lit(clause.lits[other_watch_idx]) == LBool::False {
                        self.watches[p.index()].extend(watches_list);
                        return Some(cid);
                    } else {
                        let lit_to_propagate = clause.lits[other_watch_idx];
                        let var = lit_to_propagate.var();
                        let val = if lit_to_propagate.sign() { LBool::False } else { LBool::True };
                        
                        self.assignment[var.0 as usize] = val;
                        self.level[var.0 as usize] = self.trail_lim.len();
                        self.reason[var.0 as usize] = Some(cid);
                        self.trail.push(lit_to_propagate);
                        i += 1;
                    }
                }
            }
            self.watches[p.index()] = watches_list;
        }
        None
    }

    /// First-UIP Conflict Analysis & Non-Chronological Backjumping
    fn analyze(&mut self, conflict_cid: ClauseId) -> (Vec<Lit>, usize) {
        let mut path_c = 0;
        let mut p = Lit(u32::MAX);
        let mut visited = vec![false; self.num_vars];
        let mut learnt_lits = Vec::new();
        learnt_lits.push(Lit(0));

        let mut curr_cid = Some(conflict_cid);
        let current_level = self.trail_lim.len();

        if current_level == 0 {
            return (vec![], 0);
        }

        let mut index = self.trail.len() as i32 - 1;

        loop {
            let clause_lits = self.clauses[curr_cid.unwrap().0].lits.clone();
            let start_j = if curr_cid == Some(conflict_cid) { 0 } else { 1 };
            
            for j in start_j..clause_lits.len() {
                let q = clause_lits[j];
                let v = q.var();
                let v_idx = v.0 as usize;

                if !visited[v_idx] && self.level[v_idx] > 0 {
                    visited[v_idx] = true;
                    self.bump_activity(v);

                    if self.level[v_idx] == current_level {
                        path_c += 1;
                    } else {
                        learnt_lits.push(q);
                    }
                }
            }

            while index >= 0 {
                let candidate = self.trail[index as usize];
                index -= 1;
                if visited[candidate.var().0 as usize] {
                    p = candidate;
                    break;
                }
            }

            path_c -= 1;
            if path_c <= 0 {
                break;
            }

            curr_cid = self.reason[p.var().0 as usize];
        }

        learnt_lits[0] = p.negate();

        let mut backjump_level = 0;
        for i in 1..learnt_lits.len() {
            let l_level = self.level[learnt_lits[i].var().0 as usize];
            if l_level > backjump_level {
                backjump_level = l_level;
            }
        }

        (learnt_lits, backjump_level)
    }

    fn cancel_until(&mut self, level: usize) {
        while self.trail_lim.len() > level {
            let boundary = self.trail_lim.pop().unwrap();
            while self.trail.len() > boundary {
                let lit = self.trail.pop().unwrap();
                let v_idx = lit.var().0 as usize;
                self.assignment[v_idx] = LBool::Undef;
                self.reason[v_idx] = None;
            }
        }
    }

    pub fn solve(&mut self) -> bool {
        if self.propagate().is_some() {
            return false;
        }

        loop {
            let mut next_var = None;
            let mut max_act = -1.0;

            for v in 0..self.num_vars {
                if self.assignment[v] == LBool::Undef {
                    if self.activity[v] > max_act {
                        max_act = self.activity[v];
                        next_var = Some(v);
                    }
                }
            }

            let v = match next_var {
                Some(var) => var,
                None => return true,
            };

            self.trail_lim.push(self.trail.len());
            self.assignment[v] = LBool::True;
            let lit = Lit::new(Var(v as u32), false);
            self.level[v] = self.trail_lim.len();
            self.trail.push(lit);

            while let Some(conflict_cid) = self.propagate() {
                self.conflicts += 1;

                if self.trail_lim.is_empty() {
                    return false;
                }

                let (learned_clause, backjump_level) = self.analyze(conflict_cid);
                self.cancel_until(backjump_level);

                if learned_clause.is_empty() {
                    return false;
                }

                let lit_to_prop = learned_clause[0];
                let is_unit = learned_clause.len() == 1;
                let cid = ClauseId(self.clauses.len());
                self.add_clause(learned_clause, true);
                if !is_unit {
                    self.learned_clauses_count += 1;
                }

                let var = lit_to_prop.var();
                let val = if lit_to_prop.sign() { LBool::False } else { LBool::True };
                self.assignment[var.0 as usize] = val;
                self.level[var.0 as usize] = backjump_level;
                self.reason[var.0 as usize] = Some(cid);
                self.trail.push(lit_to_prop);

                self.var_inc *= 0.95;

                if self.conflicts >= self.next_restart {
                    self.cancel_until(0);
                    self.next_restart = self.conflicts + 100;
                    break;
                }
            }
        }
    }
}