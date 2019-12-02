use crate::block::*;
use crate::cfg::*;

pub struct DominatorTree {
    blocks: Vec<CodeBlockRef>,
    i_dom: Vec<i32>,
    pub dominated: Vec<Vec<usize>>,
    blocks_to_index: BlockMap,
}

impl DominatorTree {
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            i_dom: vec![],
            dominated: vec![],
            blocks_to_index: BlockMap::new(),
        }
    }

    fn intersect(&self, b1: i32, b2: i32) -> i32 {
        let mut finger1 = b1;
        let mut finger2 = b2;
        while finger2 != finger1 {
            while finger1 < finger2 {
                finger1 = self.i_dom[finger1 as usize];
            }

            while finger2 < finger1 {
                finger2 = self.i_dom[finger2 as usize];
            }
        }

        finger1
    }

    pub fn analyze(&mut self, cfg: &ControlFlowGraph) {
        let post_order = cfg.topological_sequence();
        let mut i = 0;
        for block in post_order.iter() {
            self.blocks.push(block.clone());
            self.blocks_to_index.insert(block.clone(), i);

            self.i_dom.push(-1);
            i += 1;
        }
        self.compute_dt(cfg);
    }

    pub fn compute_dt(&mut self, cfg: &ControlFlowGraph) {
        let start_node = self
            .blocks_to_index
            .get(&cfg.get_entry_block())
            .map(|x| *x)
            .unwrap();
        let mut changed = true;
        self.i_dom[start_node] = start_node as _;
        while changed {
            changed = false;

            for b_ind in 0..self.blocks.len() {
                if b_ind == start_node {
                    break;
                }
                let b: CodeBlockRef = self.blocks[b_ind].clone();
                let mut new_idom = 0;
                let mut processed = false;
                for pred in b.borrow().predecessors.iter() {
                    let p = *self.blocks_to_index.get(pred).unwrap();
                    if self.i_dom[p] != -1 {
                        if !processed {
                            new_idom = p as i32;
                            processed = true;
                        } else {
                            new_idom = self.intersect(p as _, new_idom as _);
                        }
                    }
                }
                if processed {
                    if self.i_dom[b_ind] != new_idom {
                        self.i_dom[b_ind] = new_idom;
                        changed = true;
                    }
                }
            }
        }

        self.dominated.resize(self.blocks.len(), vec![]);
        for n in 0..self.blocks.len() {
            if self.i_dom[n] >= 0 {
                self.dominated[self.i_dom[n] as usize].push(n);
            }
        }
    }

    pub fn dominates(
        &self,
        block: CodeBlockRef,
        potential_successor: CodeBlockRef,
        cfg: &ControlFlowGraph,
    ) -> bool {
        let id = *self.blocks_to_index.get(&block).unwrap();
        let successor_id = *self.blocks_to_index.get(&potential_successor).unwrap();
        let start_id = *self.blocks_to_index.get(&cfg.get_entry_block()).unwrap();
        let mut dominates;
        let mut next_id = successor_id;
        loop {
            dominates = next_id == id;
            next_id = self.i_dom[next_id] as _;
            if !(start_id != next_id && !dominates) {
                break;
            }
        }
        dominates || next_id == id
    }

    pub fn get_dominator(&self, block: CodeBlockRef) -> Option<CodeBlockRef> {
        let n = *self.blocks_to_index.get(&block).unwrap();
        self.blocks.get(self.i_dom[n] as usize).cloned()
    }

    pub fn get_common_dominator(
        &self,
        block1: CodeBlockRef,
        block2: CodeBlockRef,
    ) -> Option<CodeBlockRef> {
        let n1 = *self.blocks_to_index.get(&block1).unwrap();
        let n2 = *self.blocks_to_index.get(&block2).unwrap();
        let n = self.intersect(self.i_dom[n1], self.i_dom[n2]);

        self.blocks.get(n as usize).cloned()
    }

    pub fn get_dominated_blocks(
        &self,
        block: CodeBlockRef,
        dominated_blocks: &mut Vec<CodeBlockRef>,
    ) {
        let n = *self.blocks_to_index.get(&block).unwrap();
        for dblock in self.dominated[n].iter() {
            dominated_blocks.push(self.blocks[*dblock].clone());
        }
    }
}
