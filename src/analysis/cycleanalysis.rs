use crate::block::*;
use crate::cfg::*;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct CycleAnalysis {
    pub back_edges: Vec<Rc<RefCell<Edge>>>,
}
impl CycleAnalysis {
    pub fn new() -> Self {
        Self { back_edges: vec![] }
    }

    pub fn analyze(&mut self, cfg: &ControlFlowGraph) {
        let mut visited = BlockSet::new();
        let mut stack = vec![];

        stack.push(cfg.blocks.first().unwrap().clone());
        while !stack.is_empty() {
            let block = stack.pop().unwrap();
            for edge in block.borrow().out_edges.iter() {
                if visited.insert(edge.borrow().tail.as_ref().unwrap().clone()) {
                    stack.push(edge.borrow().tail.as_ref().unwrap().clone());
                } else {
                    if !self.back_edges.contains(edge) {
                        self.back_edges.push(edge.clone());
                    }
                }
            }
        }
    }

    pub fn all_back_edges_mut(&mut self) -> &mut [Rc<RefCell<Edge>>] {
        &mut self.back_edges
    }
    pub fn all_back_edges(&self) -> &[Rc<RefCell<Edge>>] {
        &self.back_edges
    }

    pub fn is_back_edge(&self, edge: &Rc<RefCell<Edge>>) -> bool {
        self.back_edges
            .iter()
            .fold(0, |acc, x| if x == edge { acc + 1 } else { acc })
            != 0
    }
}
