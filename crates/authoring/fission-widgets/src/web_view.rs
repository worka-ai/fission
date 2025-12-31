use fission_core::{BuildCtx, View, Widget, Node, WidgetNodeId};
use fission_core::ui::{CustomNode, LowerDyn};
use fission_core::lowering::{NodeBuilder, LoweringContext};
use fission_ir::{LayoutOp, Op, EmbedKind, NodeId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebView {
    pub id: WidgetNodeId,
    pub url: String,
    pub user_agent: Option<String>,
}

impl<S: fission_core::AppState> Widget<S> for WebView {
    fn build(&self, ctx: &mut BuildCtx<S>, _view: &View<S>) -> Node {
        ctx.register_web_view(fission_core::registry::WebRegistration {
            node_id: self.id,
            url: self.url.clone(),
            user_agent: self.user_agent.clone(),
        });
        
        Node::Custom(CustomNode {
            debug_tag: "WebView".into(),
            lowerer: Some(std::sync::Arc::new(WebViewLowerer {
                id: self.id,
                url: self.url.clone(),
            })),
        })
    }
}

#[derive(Debug)]
struct WebViewLowerer {
    id: WidgetNodeId,
    url: String,
}

impl LowerDyn for WebViewLowerer {
    fn lower_dyn(&self, cx: &mut LoweringContext) -> NodeId {
        let id = cx.widget_node_id(self.id);
        
        let mut builder = NodeBuilder::new(id, Op::Layout(LayoutOp::Embed {
            kind: EmbedKind::Web,
            widget_id: self.id,
            width: None,
            height: None,
        }));
        
        builder.build(cx)
    }

    fn stable_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.id.hash(&mut h);
        self.url.hash(&mut h);
        h.finish()
    }
}
