use super::cycleanalysis::*;
use crate::block::*;
use crate::cfg::*;
use std::cell::RefCell;
use std::rc::Rc;

pub type SafeRegionList = Vec<Rc<RefCell<SafeRegion>>>;

#[derive(Default)]
pub struct SafeRegion {
    pub parent: Option<Rc<RefCell<SafeRegion>>>,
    pub children: SafeRegionList,
    pub block: CodeBlockRef,
}

#[derive(Default)]
pub struct SafeRegionAnalysis {
    pub root: SafeRegion,
    pub regions: std::collections::HashMap<CodeBlockRef, Rc<RefCell<SafeRegion>>>,
}

use crate::instructions::Instruction;

pub fn get_blocks_with_backward_branches(ca: &CycleAnalysis) -> BlockSet {
    let edges = ca.all_back_edges().to_vec();

    let mut set = BlockSet::new();
    for edge in edges.iter() {
        if edge.borrow().ty != EdgeType::Branch {
            continue;
        }
        let block: CodeBlockRef = edge.borrow().head.as_ref().unwrap().clone();
        if get_branch(&block).is_none() {
            continue;
        }
        set.insert(block);
    }
    set
}

pub fn get_branch(block: &CodeBlockRef) -> Option<Instruction> {
    if block.borrow().instructions.is_empty() {
        return None;
    }
    let branch = block.borrow().instructions.last().unwrap().clone();
    match branch {
        Instruction::Jmp(_) | Instruction::JmpNz(_) | Instruction::JmpZ(_) => {
            return Some(branch.clone())
        }
        _ => return None,
    }
}

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
