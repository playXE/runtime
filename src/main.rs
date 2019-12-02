extern crate runtime;

use runtime::block::*;
use runtime::cfg;
use runtime::instructions::*;

use std::cell::*;
use std::rc::Rc;
fn main() {
    let mut cfg = cfg::ControlFlowGraph::new();

    let mut block: CodeBlock = CodeBlock::default();
    block.id = cfg.new_id();
    block.instructions.push(Instruction::Jmp(1));
    let mut b1 = cfg.insert_block(CodeBlockRef::new(block));

    let mut block: CodeBlock = CodeBlock::default();
    block.id = cfg.new_id();
    block.instructions.push(Instruction::Jmp(1));

    let mut b2 = cfg.insert_block(CodeBlockRef::new(block));
    //cfg.entry = b1.clone();
    let edge = Edge {
        head: Some(b2.clone()),
        tail: Some(b1.clone()),
        ty: EdgeType::Branch,
        ..Default::default()
    };

    let edge = cfg.insert_edge(Rc::new(RefCell::new(edge)));
    b1.borrow_mut().out_edges.push(edge);

    let mut ca = runtime::analysis::cycleanalysis::CycleAnalysis::new();
    ca.analyze(&cfg);
    let mut dt = runtime::analysis::dom::DominatorTree::new(&cfg);
    dt.analyze();

    println!("{:?}", dt.dominated);
    println!("{}", ca.back_edges.len());
}
