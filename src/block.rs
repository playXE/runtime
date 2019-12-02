use std::cell::RefCell;
use std::rc::Rc;

use crate::instructions::Instruction;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum EdgeType {
    Branch,
    FallThrough,
    Dummy,
    Invalid,
}

impl Default for EdgeType {
    fn default() -> Self {
        Self::Invalid
    }
}

#[derive(PartialEq, Eq, Hash, Default, Debug)]
pub struct Edge {
    pub ty: EdgeType,
    pub head: Option<CodeBlockRef>,
    pub tail: Option<CodeBlockRef>,
}

#[derive(PartialEq, Eq, Default, Debug)]
pub struct CodeBlock {
    pub children: Vec<CodeBlockRef>,
    pub instructions: Vec<Instruction>,
    pub id: usize,
    pub in_edges: Vec<Rc<RefCell<Edge>>>,
    pub out_edges: Vec<Rc<RefCell<Edge>>>,
    pub successors: Vec<CodeBlockRef>,
    pub predecessors: Vec<CodeBlockRef>,
}

use std::hash::{Hash, Hasher};

impl Hash for CodeBlock {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.instructions.hash(h);
        for block in self.children.iter() {
            block.borrow().hash(h);
        }
        self.children.len().hash(h);
    }
}

impl CodeBlock {
    pub fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CodeBlockRef {
    val: Rc<RefCell<CodeBlock>>,
}

impl Hash for CodeBlockRef {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.val.borrow().hash(h);
    }
}
impl Default for CodeBlockRef {
    fn default() -> Self {
        Self {
            val: Rc::new(RefCell::new(CodeBlock::default())),
        }
    }
}
impl CodeBlockRef {
    pub fn new(b: CodeBlock) -> Self {
        Self {
            val: Rc::new(RefCell::new(b)),
        }
    }
    pub fn borrow(&self) -> std::cell::Ref<'_, CodeBlock> {
        self.val.borrow()
    }
    pub fn borrow_mut(&mut self) -> std::cell::RefMut<'_, CodeBlock> {
        self.val.borrow_mut()
    }
}

pub type BlockSet = std::collections::HashSet<CodeBlockRef>;

pub fn get_block_that_can_observe_side_effects(blocks: &[CodeBlockRef]) -> Vec<Instruction> {
    let mut instructions = vec![];
    for block in blocks.iter() {
        let block: &CodeBlockRef = block;
        for instruction in block.borrow().instructions.iter() {
            if instruction.can_observe_side_effects() {
                instructions.push(*instruction);
            }
        }
    }

    instructions
}

pub fn get_blocks_with_calls_to_functions_that_observe_side_effects(
    blocks: &[CodeBlockRef],
) -> BlockSet {
    let mut set = BlockSet::new();
    for block in blocks.iter() {
        for instruction in block.borrow().instructions.iter() {
            match instruction {
                Instruction::Call(_) | Instruction::TailCall(_) => {
                    set.insert(block.clone());
                }
                _ => (),
            }
        }
    }
    set
}
