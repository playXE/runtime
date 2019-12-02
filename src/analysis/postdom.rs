use crate::block::*;
use crate::cfg::*;

pub type IndexVector = Vec<i32>;
pub type IndexArrayVector = Vec<IndexVector>;

pub struct PostDominatorTree {
    pub blocks: Vec<CodeBlockRef>,
    pub p_dom: IndexVector,
    pub dominated: IndexArrayVector,
    pub frontiers: IndexArrayVector,
    pub blocks_to_index: BlockMap,
}

impl PostDominatorTree {
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            p_dom: vec![],
            dominated: vec![],
            frontiers: vec![],
            blocks_to_index: BlockMap::new(),
        }
    }

    pub fn dominates(
        &self,
        block: CodeBlockRef,
        potential_predecessor: CodeBlockRef,
        cfg: &ControlFlowGraph,
    ) -> bool {
        let id = *self.blocks_to_index.get(&block).unwrap();
        let successor_id = *self.blocks_to_index.get(&potential_predecessor).unwrap();
        let start_id = *self.blocks_to_index.get(&cfg.exit).unwrap();
        let mut dominates;
        let mut next_id = successor_id;
        loop {
            dominates = next_id == id;
            next_id = self.p_dom[next_id] as _;
            if !(start_id != next_id && !dominates) {
                break;
            }
        }
        dominates || next_id == id
    }

    pub fn analyze(&mut self, cfg: &ControlFlowGraph) {
        let post_order = cfg.reverse_topological_sequence();
        let mut i = 0;
        for block in post_order.iter() {
            self.blocks.push(block.clone());
            self.blocks_to_index.insert(block.clone(), i);
            self.p_dom.push(-1);
            i += 1;
        }
        self.compute_dt(cfg);
    }

    pub fn intersect(&self, b1: i32, b2: i32) -> i32 {
        let mut finger1 = b1;
        let mut finger2 = b2;
        while finger1 != finger2 {
            while finger1 < finger2 {
                finger1 = self.p_dom[finger1 as usize];
            }
            while finger2 < finger1 {
                finger2 = self.p_dom[finger2 as usize];
            }
        }
        finger1
    }

    pub fn compute_dt(&mut self, cfg: &ControlFlowGraph) {
        let end_node = *self.blocks_to_index.get(&cfg.exit.clone()).unwrap();
        let mut changed = true;
        self.p_dom[end_node] = end_node as _;

        while changed {
            changed = false;
            for b_ind in 0..self.blocks.len() {
                if b_ind == end_node {
                    continue;
                }
                let b: CodeBlockRef = self.blocks[b_ind].clone();
                assert!(b.borrow().successors.is_empty());
                let mut new_pdom = 0;
                let mut processed = false;
                for succ in b.borrow().successors.iter() {
                    let p = *self.blocks_to_index.get(succ).unwrap();
                    assert!(p < self.p_dom.len());
                    if self.p_dom[p] != -1 {
                        if !processed {
                            new_pdom = p as i32;
                            processed = true;
                        } else {
                            new_pdom = self.intersect(p as _, new_pdom as _)
                        }
                    }
                }
                if processed {
                    if self.p_dom[b_ind] != new_pdom {
                        self.p_dom[b_ind] = new_pdom;
                        changed = true;
                    }
                }
            }
        }

        self.dominated.resize(self.blocks.len(), vec![]);
        for n in 0..self.blocks.len() {
            if self.p_dom[n] >= 0 {
                self.dominated[self.p_dom[n] as usize].push(n as _);
            }
        }

        self.frontiers.resize(self.blocks.len(), vec![]);
        for b_ind in 0..self.blocks.len() {
            let block: CodeBlockRef = self.blocks[b_ind].clone();
            if block.borrow().successors.len() < 2 {
                continue;
            }
            let mut blocks_with_this_block_in_their_frontier: std::collections::HashSet<usize> =
                std::collections::HashSet::new();

            for successor in block.borrow().successors.iter() {
                let mut runner: CodeBlockRef = successor.clone();

                while runner != self.get_post_dominator(block.clone()) {
                    blocks_with_this_block_in_their_frontier
                        .insert(*self.blocks_to_index.get(&runner).unwrap());
                    runner = self.get_post_dominator(runner);
                }
            }

            for frontier_block in blocks_with_this_block_in_their_frontier.iter() {
                self.frontiers[b_ind].push(*frontier_block as _);
            }
        }
    }

    pub fn get_post_dominator(&self, block: CodeBlockRef) -> CodeBlockRef {
        let n = *self.blocks_to_index.get(&block).unwrap();
        self.blocks[self.p_dom[n] as usize].clone()
    }
}
