pub mod cycleanalysis;
pub mod dom;
pub mod hammockgraph;
pub mod postdom;
pub mod saferegion;

pub struct Analysis<'a> {
    cfg: &'a crate::cfg::ControlFlowGraph,
    dom: Option<dom::DominatorTree>,
    post_dom: Option<postdom::PostDominatorTree>,
    cycle: Option<cycleanalysis::CycleAnalysis>,
    hammockgraph: Option<hammockgraph::HammockAnalysis>,
    saferegion: Option<saferegion::SafeRegionAnalysis>,
}
impl<'a> Analysis<'a> {
    pub fn new(cfg: &'a crate::cfg::ControlFlowGraph) -> Self {
        Self {
            cfg: cfg,
            dom: None,
            post_dom: None,
            cycle: None,
            hammockgraph: None,
            saferegion: None,
        }
    }
}
