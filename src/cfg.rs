use crate::block::*;

use std::cell::*;
use std::rc::Rc;

pub type EdgePair = (Rc<RefCell<Edge>>, Rc<RefCell<Edge>>);
pub type BlockMap = std::collections::HashMap<CodeBlockRef, usize>;

#[derive(Default, Debug)]
pub struct ControlFlowGraph {
    pub edges: Rc<RefCell<Vec<Rc<RefCell<Edge>>>>>,
    pub blocks: Vec<CodeBlockRef>,
    pub next_id: usize,
    pub entry: CodeBlockRef,
    pub exit: CodeBlockRef,
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        let mut this = Self::default();
        let entry = CodeBlockRef::new(CodeBlock {
            id: 0,
            ..Default::default()
        });
        this.blocks.push(entry.clone());
        this.entry = entry;
        let exit = CodeBlockRef::new(CodeBlock {
            id: 0,
            ..Default::default()
        });
        this.blocks.push(exit.clone());
        this.exit = exit;
        this.next_id = 2;
        this
    }

    pub fn get_entry_block(&self) -> CodeBlockRef {
        self.entry.clone()
    }
    pub fn compute_new_block_id(&mut self) {
        self.next_id = 0;
        for block in self.blocks.iter() {
            self.next_id = std::cmp::max(self.next_id, block.borrow().id);
        }
        self.next_id += 1;
    }
    pub fn new_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id - 1
    }

    pub fn size(&self) -> usize {
        self.blocks.len()
    }
    pub fn ins_count(&self) -> usize {
        let mut count = 0;
        for block in self.blocks.iter() {
            count += block.borrow().instructions.len();
        }
        count
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn insert_block(&mut self, block: CodeBlockRef) -> CodeBlockRef {
        self.blocks.insert(self.blocks.len(), block.clone());
        block
    }

    pub fn clone_block(&mut self, block: CodeBlockRef) {
        let id = self.new_id();
        /*let new_block = CodeBlock {
            instructions: block.borrow().instructions.clone(),
            children: vec![],
            id: block.borrow().id,
        };*/
        let mut new_block = CodeBlock::default();
        new_block.instructions = block.borrow().instructions.clone();
        new_block.id = id;
        self.insert_block(CodeBlockRef::new(new_block));
    }

    pub fn remove_edge(&mut self, edge: Rc<RefCell<Edge>>) {
        let edge_: &Edge = &edge.borrow();
        let out = {
            let mut value_: Option<usize> = None;
            for (i, value) in edge_
                .head
                .as_ref()
                .unwrap()
                .borrow()
                .out_edges
                .iter()
                .enumerate()
            {
                if *value.borrow() == *edge.borrow() {
                    value_ = Some(i);
                    break;
                }
            }
            value_
        };
        assert!(out.is_some());

        let mut head = edge_.head.as_ref().unwrap().clone();
        head.borrow_mut().out_edges.remove(out.unwrap());
        let in_ = {
            let mut value_: Option<usize> = None;
            for (i, value) in edge_
                .tail
                .as_ref()
                .unwrap()
                .borrow()
                .in_edges
                .iter()
                .enumerate()
            {
                if *value.borrow() == *edge.borrow() {
                    value_ = Some(i);
                    break;
                }
            }
            value_
        };
        assert!(in_.is_some());
        drop(head);

        let mut tail = edge_.tail.as_ref().unwrap().clone();
        tail.borrow_mut().in_edges.remove(in_.unwrap());

        let heads = {
            let mut value_: Option<usize> = None;
            for (i, value) in edge_
                .head
                .as_ref()
                .unwrap()
                .borrow()
                .successors
                .iter()
                .enumerate()
            {
                if *value == *edge.borrow().tail.as_ref().unwrap() {
                    value_ = Some(i);
                    break;
                }
            }
            value_
        };
        let mut head = edge_.head.as_ref().unwrap().clone();
        head.borrow_mut().successors.remove(heads.unwrap());
        let heads = {
            let mut value_: Option<usize> = None;
            for (i, value) in edge_
                .head
                .as_ref()
                .unwrap()
                .borrow()
                .predecessors
                .iter()
                .enumerate()
            {
                if *value == *edge.borrow().head.as_ref().unwrap() {
                    value_ = Some(i);
                    break;
                }
            }
            value_
        };
        let mut head = edge_.head.as_ref().unwrap().clone();
        head.borrow_mut().predecessors.remove(heads.unwrap());

        let i = {
            let mut x = None;
            for (i, val) in self.edges.borrow().iter().enumerate() {
                if *val.borrow() == *edge.borrow() {
                    x = Some(i);
                }
            }
            x.unwrap()
        };
        self.edges.borrow_mut().remove(i);
    }

    pub fn insert_edge(&mut self, edge: Rc<RefCell<Edge>>) -> Rc<RefCell<Edge>> {
        let len = self.edges.borrow().len();
        self.edges.borrow_mut().insert(len, edge.clone());
        let tail = edge.borrow().tail.as_ref().unwrap().clone();
        let head = edge.borrow().head.as_ref().unwrap().clone();
        edge.borrow_mut()
            .head
            .as_mut()
            .unwrap()
            .borrow_mut()
            .out_edges
            .push(edge.clone());
        edge.borrow_mut()
            .tail
            .as_mut()
            .unwrap()
            .borrow_mut()
            .in_edges
            .push(edge.clone());

        edge.borrow_mut()
            .head
            .as_mut()
            .unwrap()
            .borrow_mut()
            .successors
            .push(tail.clone());
        edge.borrow_mut()
            .tail
            .as_mut()
            .unwrap()
            .borrow_mut()
            .predecessors
            .push(head.clone());
        edge.clone()
    }

    pub fn split_edge(&mut self, edge: Rc<RefCell<Edge>>, new_block: CodeBlockRef) -> EdgePair {
        let head = edge.borrow().head.clone();
        let tail = edge.borrow().tail.clone();
        let ty = edge.borrow().ty;
        self.remove_edge(edge.clone());
        self.insert_block(new_block.clone());

        let first_edge = self.insert_edge(Rc::new(RefCell::new(Edge {
            head: head,
            tail: Some(new_block.clone()),
            ty: ty,
            ..Default::default()
        })));
        let second_edge = self.insert_edge(Rc::new(RefCell::new(Edge {
            head: Some(new_block.clone()),
            tail: tail,
            ty: ty,
            ..Default::default()
        })));

        (first_edge, second_edge)
    }

    pub fn topological_sequence(&self) -> Vec<CodeBlockRef> {
        let mut visited: BlockSet = BlockSet::new();
        let mut sequence: Vec<CodeBlockRef> = vec![];
        let mut queue = std::collections::LinkedList::new();
        queue.push_back(self.get_entry_block());
        while sequence.len() != self.size() {
            if queue.is_empty() {
                for block in sequence.iter() {
                    for successor in block.borrow().successors.iter() {
                        if visited
                            .iter()
                            .fold(0, |acc, x| if x == successor { acc + 1 } else { acc })
                            == 0
                        {
                            queue.push_back(successor.clone());
                            break;
                        }
                    }
                    if !queue.is_empty() {
                        break;
                    }
                }
                if queue.is_empty() {
                    break;
                }
            }

            let current = queue.pop_front().unwrap();
            if !visited.insert(current.clone()) {
                continue;
            }
            sequence.push(current.clone());
            for block in current.borrow().successors.iter() {
                let mut no_dependences = true;
                for pred in current.borrow().predecessors.iter() {
                    if visited
                        .iter()
                        .fold(0, |acc, x| if x == pred { acc + 1 } else { 0 })
                        == 0
                    {
                        no_dependences = false;
                        break;
                    }
                }

                if no_dependences {
                    queue.push_back(block.clone());
                }
            }
        }
        sequence
    }

    pub fn reverse_topological_sequence(&self) -> Vec<CodeBlockRef> {
        let mut visited: BlockSet = BlockSet::new();
        let mut sequence: Vec<CodeBlockRef> = vec![];
        let mut queue = std::collections::LinkedList::new();
        queue.push_back(self.get_entry_block());
        while sequence.len() != self.size() {
            if queue.is_empty() {
                for block in sequence.iter() {
                    for successor in block.borrow().predecessors.iter() {
                        if visited
                            .iter()
                            .fold(0, |acc, x| if x == successor { acc + 1 } else { acc })
                            == 0
                        {
                            queue.push_back(successor.clone());
                            break;
                        }
                    }
                    if !queue.is_empty() {
                        break;
                    }
                }
                if queue.is_empty() {
                    break;
                }
            }

            let current = queue.pop_front().unwrap();
            if !visited.insert(current.clone()) {
                continue;
            }
            sequence.push(current.clone());
            for block in current.borrow().predecessors.iter() {
                let mut no_dependences = true;
                for pred in current.borrow().successors.iter() {
                    if visited
                        .iter()
                        .fold(0, |acc, x| if x == pred { acc + 1 } else { 0 })
                        == 0
                    {
                        no_dependences = false;
                        break;
                    }
                }

                if no_dependences {
                    queue.push_back(block.clone());
                }
            }
        }
        sequence
    }
}
