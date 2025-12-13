use anyhow::Result;
use fission_core::{Runtime, Action, ActionId, AppState, CurrentTime, AdvanceTo, Tick, Desugar, LoweringContext};
use fission_core::lowering::build_layout_tree;
use fission_ir::NodeId;
use fission_layout::{LayoutSnapshot, LayoutSize, LayoutEngine};
use fission_render::{Renderer, DisplayList, LayoutRect};
use std::sync::{Arc, Mutex};

// A mock renderer that captures the display list for inspection.
#[derive(Default, Clone)]
pub struct MockRenderer {
    pub last_display_list: Arc<Mutex<Option<DisplayList>>>,
}

impl Renderer for MockRenderer {
    fn render(&mut self, display_list: &DisplayList) -> Result<()> {
        let mut lock = self.last_display_list.lock().unwrap();
        *lock = Some(display_list.clone());
        Ok(())
    }
}

pub struct TestHarness {
    pub runtime: Runtime,
    pub renderer: MockRenderer,
    pub layout_engine: LayoutEngine,
    pub last_snapshot: Option<LayoutSnapshot>,
    pub root_widget: Option<Box<dyn Desugar>>,
}

impl Default for TestHarness {
    fn default() -> Self {
        Self {
            runtime: Runtime::default(),
            renderer: MockRenderer::default(),
            layout_engine: LayoutEngine::new(),
            last_snapshot: None,
            root_widget: None,
        }
    }
}

impl TestHarness {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_app_state<S: AppState + 'static>(mut self, state: S) -> Self {
        self.runtime.add_app_state(Box::new(state)).expect("Failed to add app state");
        self
    }

    pub fn with_root_widget<W: Desugar + 'static>(mut self, widget: W) -> Self {
        self.root_widget = Some(Box::new(widget));
        self
    }

    pub fn register_reducer<S: AppState + 'static>(
        mut self,
        action_id: ActionId,
        reducer: fn(&mut S, &dyn Action, NodeId) -> Result<()>,
    ) -> Self {
        self.runtime.register_reducer::<S>(action_id, reducer).unwrap();
        self
    }

    pub fn dispatch(&mut self, action: impl Action + 'static) -> Result<()> {
        let target = NodeId::derived(0, &[0]);
        self.runtime.dispatch(Box::new(action), target)
    }

    pub fn tick(&mut self, dt: CurrentTime) -> Result<()> {
        let action = Tick { dt };
        self.dispatch(action)
    }

    pub fn advance_to(&mut self, time: CurrentTime) -> Result<()> {
        self.dispatch(AdvanceTo { time })
    }

    pub fn current_time(&self) -> CurrentTime {
        self.runtime.clock().current_time()
    }

    // A simulated "frame" evaluation
    pub fn pump(&mut self) -> Result<()> {
        // 1. Lowering
        let mut layout_input_nodes = Vec::new();
        
        if let Some(root) = &self.root_widget {
            let mut cx = LoweringContext::new();
            let _root_id = root.desugar(&mut cx);
            // Convert CoreIR to LayoutInputNodes
            layout_input_nodes = build_layout_tree(&cx.ir);
        }

        // 2. Layout
        let viewport = LayoutSize { width: 800.0, height: 600.0 };
        let snapshot = self.layout_engine.compute_layout(&layout_input_nodes, viewport)?;
        self.last_snapshot = Some(snapshot.clone());

        // 3. Render
        let display_list = DisplayList::new(LayoutRect::new(0.0, 0.0, 800.0, 600.0));
        // Real rendering would traverse the snapshot and emit ops. 
        // For now, our dummy renderer and display list generator are simple.
        self.renderer.render(&display_list)?;

        Ok(())
    }
    
    pub fn get_last_display_list(&self) -> Option<DisplayList> {
        self.renderer.last_display_list.lock().unwrap().clone()
    }
}
