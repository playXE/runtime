use super::cycleanalysis::*;
use super::{dom::*, postdom::*};
use crate::block::*;
use crate::cfg::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, PartialEq, Eq)]
pub struct Hammock {
    pub parent: Option<Rc<RefCell<Hammock>>>,
    pub children: Vec<Rc<RefCell<Hammock>>>,
    pub entry: CodeBlockRef, // if the entry and exit are the same, the hammock contains a single block
    pub exit: CodeBlockRef,
}

impl Hammock {
    pub fn is_leaf(&self) -> bool {
        self.exit == self.entry
    }
}

pub struct HammockAnalysis {
    pub root: Rc<RefCell<Hammock>>,
    pub map: std::collections::HashMap<CodeBlockRef, Rc<RefCell<Hammock>>>,
}

pub type BlockToHammockMap = std::collections::HashMap<CodeBlockRef, Rc<RefCell<Hammock>>>;

impl HammockAnalysis {
    pub fn analyze(
        &mut self,
        cfg: &ControlFlowGraph,
        dom: &DominatorTree,
        pdom: &PostDominatorTree,
    ) {
        self.root.borrow_mut().entry = cfg.entry.clone();
        self.root.borrow_mut().exit = cfg.exit.clone();

        for block in cfg.blocks.iter() {
            if block == &cfg.entry {
                continue;
            }
            if block == &cfg.exit {
                continue;
            }

            self.root
                .borrow_mut()
                .children
                .push(Rc::new(RefCell::new(Hammock {
                    parent: Some(self.root.clone()),
                    entry: block.clone(),
                    exit: block.clone(),
                    children: vec![],
                })))
        }

        self.split_hammock(self.root.clone(), dom, pdom, cfg);
    }

    pub fn expand_hammock(
        &mut self,
        entry: &mut CodeBlockRef,
        exit: &mut CodeBlockRef,
        parent_entry: CodeBlockRef,
        parent_exit: CodeBlockRef,
        dt: &DominatorTree,
        pdt: &PostDominatorTree,
        cfg: &ControlFlowGraph,
    ) -> bool {
        let mut dominator = dt.get_dominator(entry.clone()).unwrap();
        if dominator == *entry || dominator == parent_exit {
            return false;
        }
        while dominator.borrow().successors.len() < 2 {
            dominator = dt.get_dominator(dominator).unwrap();
            if dominator == parent_entry || dominator == parent_exit {
                return false;
            }
        }
        let post_dominator = pdt.get_post_dominator(dominator.clone());
        if !pdt.dominates(post_dominator.clone(), exit.clone(), cfg) {
            return false;
        }

        let changed = *entry != dominator || *exit != post_dominator;
        *entry = dominator;
        *exit = post_dominator;

        changed
    }

    pub fn create_new_hammock(
        &mut self,
        dt: &DominatorTree,
        pdt: &PostDominatorTree,
        cfg: &ControlFlowGraph,
        unvisited: &mut BlockToHammockMap,
        hammock: Rc<RefCell<Hammock>>,
    ) -> Rc<RefCell<Hammock>> {
        let mut entry = hammock.borrow().entry.clone();
        let mut exit = hammock.borrow().exit.clone();

        let mut changed = true;
        while changed {
            changed = self.expand_hammock(
                &mut entry,
                &mut exit,
                hammock
                    .borrow()
                    .parent
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .entry
                    .clone(),
                hammock
                    .borrow()
                    .parent
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .exit
                    .clone(),
                dt,
                pdt,
                cfg,
            );

            if unvisited
                .iter()
                .fold(0, |acc, (x, _)| if x == &entry { acc + 1 } else { acc })
                == 0
            {
                return hammock;
            }
        }

        if entry == hammock.borrow().entry {
            return hammock;
        }
        if exit == hammock.borrow().entry {
            return hammock;
        }

        let mut new_hammock = unvisited.get(&entry).cloned().unwrap();
        new_hammock.borrow_mut().entry = entry.clone();
        new_hammock.borrow_mut().exit = exit.clone();
        unvisited.remove(&entry);
        let mut unvisited_iter: Vec<Rc<RefCell<Hammock>>> = vec![];
        for v in unvisited.iter() {
            unvisited_iter.push(v.1.clone());
        }
        for v in unvisited_iter.iter() {
            let mut v: Rc<RefCell<Hammock>> = v.clone();
            if v.borrow().entry == entry {
                continue;
            }
            if v.borrow().entry == exit {
                continue;
            }

            if !dt.dominates(entry.clone(), v.borrow().entry.clone(), cfg) {
                continue;
            }
            if !pdt.dominates(exit.clone(), v.borrow().exit.clone(), cfg) {
                continue;
            }

            v.borrow_mut().parent = Some(new_hammock.clone());

            for x in new_hammock
                .borrow()
                .parent
                .as_ref()
                .unwrap()
                .borrow()
                .children
                .iter()
            {
                if unvisited_iter.contains(x) {
                    new_hammock.borrow_mut().children.push(x.clone());
                }
            }
            unvisited.remove(&v.borrow().entry);
        }

        new_hammock
    }

    pub fn split_hammock(
        &mut self,
        hammock: Rc<RefCell<Hammock>>,
        dt: &DominatorTree,
        pdt: &PostDominatorTree,
        cfg: &ControlFlowGraph,
    ) {
        let mut unvisited = BlockToHammockMap::new();
        for child in hammock.borrow().children.iter() {
            unvisited.insert(child.borrow().entry.clone(), child.clone());
        }

        let mut changed = true;

        while changed {
            changed = false;
            for child in unvisited.clone().iter() {
                let new_hammock =
                    self.create_new_hammock(dt, pdt, cfg, &mut unvisited, child.1.clone());
                let is_as_subset = !new_hammock.borrow().is_leaf();
                if is_as_subset {
                    self.split_hammock(new_hammock, dt, pdt, cfg);
                    changed = true;
                    break;
                }
            }
        }
    }
}
