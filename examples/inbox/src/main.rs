use fission_core::{BuildCtx, View, Widget, NodeId, WidgetNodeId, Env, Handler};
use fission_core::ui::{Node, Text, Container};
use fission_core::op::{Color, BoxShadow};
use fission_widgets::{
    Grid, GridItem, SplitView, SplitDirection, Router, Route, Toast, ToastKind, Drawer, DrawerSide, SafeArea,
    HStack, Overlay, Transition, Center
};
use fission_shell_desktop::DesktopApp;
use fission_i18n::{I18nRegistry, Locale, TranslationBundle};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

mod model;
mod components;
mod features;

use model::*;
use components::{Sidebar, EmailList, EmailDetail, RightSidebar};
use features::{SettingsModal, ContactsModal, ComposeModal, BrowserModal};
use fission_core::{SystemEffect, ReducerContext, ActionInput, Action, ActionRegistry};

// --- APP ---

struct InboxApp;

impl Widget<InboxState> for InboxApp {
    fn build(&self, ctx: &mut BuildCtx<InboxState>, view: &View<InboxState>) -> Node {
        // Register Modals
        if view.state.show_settings {
            let node = SettingsModal.build(ctx, view);
            ctx.register_portal(node);
        }
        if view.state.show_contacts {
            let node = ContactsModal.build(ctx, view);
            ctx.register_portal(node);
        }
        if view.state.show_compose {
            let node = ComposeModal.build(ctx, view);
            ctx.register_portal(node);
        }
        if view.state.show_browser_demo {
            let node = BrowserModal.build(ctx, view);
            ctx.register_portal(node);
        }
        
        // Register Mobile Drawer
        // Note: We always build it but control visibility via state passed to Drawer
        // Actually Drawer takes `is_open`.
        // We put Sidebar inside it.
        let drawer_node = Drawer {
            id: WidgetNodeId::explicit("mobile_drawer"),
            side: DrawerSide::Left,
            is_open: view.state.show_mobile_menu,
            on_dismiss: Some(ctx.bind(SetMobileMenuOpen(false), (|s, a, _| s.show_mobile_menu = a.0) as Handler<InboxState, SetMobileMenuOpen>)),
            content: Box::new(Sidebar.build(ctx, view)),
            width: Some(250.0),
        }.build(ctx, view);
        
        // We register portal if open? Or Drawer handles it?
        // Drawer handles portal registration internally if open.
        // But wait, Drawer `build` returns Spacer if closed?
        // Yes. So we can just call build.
        // Wait, if Drawer `build` registers portal, it modifies `ctx`.
        // So we just call it.
        // BUT `build` returns a Node. We must include that Node in the tree?
        // Drawer returns a Spacer.
        // So we can ignore it? No, we must return a single Root node.
        // We can't just call it and discard.
        // We can put it in a ZStack?
        // Or just let it register portal and discard the spacer?
        // `ctx.register_portal` is side-effect.
        // So `let _ = drawer_node;` works?
        // Yes.
        // BUT strict widget rules: "build" should be pure?
        // `build` mutates `ctx`.
        // So we should include the returned node in the tree or just drop it if it's spacer.
        // Let's drop it.
        
        let _ = drawer_node; 
        
        // Register Toast
        if view.state.show_toast {
            let toast = Toast {
                id: WidgetNodeId::explicit("app_toast"),
                kind: ToastKind::Success,
                message: "Action completed successfully".into(),
                on_close: Some(ctx.bind(ToggleToast(false), (|s, _, _| s.show_toast = false) as Handler<InboxState, ToggleToast>)),
            }.build(ctx, view);
            
            ctx.register_portal(
                fission_widgets::Positioned {
                    left: Some(20.0), bottom: Some(20.0), // Bottom left toast
                    width: None, height: None,
                    child: Some(Box::new(toast)),
                    ..Default::default()
                }.into_node()
            );
        }

        let main_content = SafeArea {
            id: None,
            child: Box::new(
                SplitView {
                    id: WidgetNodeId::explicit("main_split"),
                    direction: SplitDirection::Horizontal,
                    split_ratio: 0.22,
                    on_resize: None,
                    first: Box::new(Sidebar.build(ctx, view)),
                    second: Box::new(
                        HStack {
                            spacing: None,
                            children: vec![
                                Router {
                                    current_path: view.state.current_path.clone(),
                                    routes: vec![
                                        Route {
                                            path: "/inbox".into(),
                                            builder: Arc::new(|c, v, _p| {
                                                EmailList { folder: "Inbox".into() }.build(c, v)
                                            }),
                                        },
                                        Route {
                                            path: "/:folder".into(),
                                            builder: Arc::new(|c, v, p| {
                                                let folder = p.get("folder").unwrap_or(&"Inbox".to_string()).clone();
                                                EmailList { folder }.build(c, v)
                                            }),
                                        },
                                        Route {
                                            path: "/:folder/:id".into(),
                                            builder: Arc::new(|c, v, p| {
                                                let folder = p.get("folder").unwrap_or(&"Inbox".to_string()).clone();
                                                let id = p.get("id").unwrap_or(&"0".to_string()).parse().unwrap_or(0);
                                                EmailDetail { folder, id }.build(c, v)
                                            }),
                                        },
                                    ],
                                    not_found: Some(Arc::new(|_c, _v, _| {
                                        fission_core::ui::Text::new("Folder not found").into_node()
                                    })),
                                }.build(ctx, view),
                                RightSidebar.build(ctx, view),
                            ],
                        }.build(ctx, view)
                    ),
                }.build(ctx, view)
            )
        }.into();

        let overlay_tip = if view.state.show_quick_tip {
            Transition {
                id: WidgetNodeId::explicit("quick_tip_fade"),
                value: 1.0,
                property: fission_core::AnimationPropertyId::Opacity,
                duration: 300,
                delay: 0,
                child: Box::new(
                    Center {
                        child: Box::new(
                            fission_widgets::Card {
                                child: Box::new(
                                    fission_widgets::VStack {
                                        spacing: Some(6.0),
                                        children: vec![
                                            fission_core::ui::Text::new("Tip: press ? for shortcuts").into_node(),
                                            fission_core::ui::Text::new("You can pin labels and drag to reorder.").size(12.0).into_node(),
                                        ],
                                    }.into_node()
                                ),
                                ..Default::default()
                            }.build(ctx, view)
                        )
                    }.build(ctx, view)
                ),
            }.build(ctx, view)
        } else {
            fission_core::ui::widgets::Spacer::default().into_node()
        };

        Overlay {
            id: None,
            content: Box::new(main_content),
            overlay: Box::new(overlay_tip),
        }.into()
    }
}

// Handlers for Browser Demo
fn on_open_system_link(_state: &mut InboxState, action: OpenSystemLink, ctx: &mut ReducerContext<InboxState>) {
    ctx.effects.add(SystemEffect::OpenUrl { url: action.0, in_app: false });
}

fn on_open_in_app_link(_state: &mut InboxState, action: OpenInAppLink, ctx: &mut ReducerContext<InboxState>) {
    ctx.effects.add(SystemEffect::OpenUrl { url: action.0, in_app: true });
}

fn on_start_auth(_state: &mut InboxState, _action: StartAuth, ctx: &mut ReducerContext<InboxState>) {
    ctx.effects.add(SystemEffect::Authenticate { 
        url: "https://auth.example.com/login".into(),
        callback_scheme: "fission-inbox://callback".into()
    });
}

// --- SETUP ---

fn create_env() -> Env {
    let mut env = Env::default();
    
    let mut en_messages = HashMap::new();
    en_messages.insert("folder.inbox".into(), "Inbox".into());
    en_messages.insert("folder.starred".into(), "Starred".into());
    en_messages.insert("folder.sent".into(), "Sent".into());
    en_messages.insert("folder.drafts".into(), "Drafts".into());
    en_messages.insert("folder.trash".into(), "Trash".into());
    en_messages.insert("button.compose".into(), "Compose".into());
    en_messages.insert("app.title".into(), "Fission Inbox".into());
    
    env.i18n.add_bundle(TranslationBundle {
        locale: Locale("en-US".into()),
        messages: en_messages,
    });

    let mut es_messages = HashMap::new();
    es_messages.insert("folder.inbox".into(), "Bandeja de entrada".into());
    es_messages.insert("folder.starred".into(), "Destacados".into());
    es_messages.insert("folder.sent".into(), "Enviados".into());
    es_messages.insert("folder.drafts".into(), "Borradores".into());
    es_messages.insert("folder.trash".into(), "Papelera".into());
    es_messages.insert("button.compose".into(), "Redactar".into());
    es_messages.insert("app.title".into(), "Buzón Fission".into());
    
    env.i18n.add_bundle(TranslationBundle {
        locale: Locale("es-ES".into()),
        messages: es_messages,
    });
    
    env
}

fn main() -> anyhow::Result<()> {
    let mut app = DesktopApp::new(InboxApp)
        .with_env(create_env())
        .with_sync_env(|state: &InboxState, env: &mut Env| {
            env.locale = state.locale.clone();
        });
        
    // Register global handlers
    let mut registry = ActionRegistry::new();
    registry.register(on_open_system_link as Handler<InboxState, OpenSystemLink>);
    registry.register(on_open_in_app_link as Handler<InboxState, OpenInAppLink>);
    registry.register(on_start_auth as Handler<InboxState, StartAuth>);
    
    app.absorb_registry(registry);
        
    app.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use fission_core::event::{InputEvent, PointerButton, PointerEvent};
    use fission_test::TestHarness;

    fn click(h: &mut TestHarness<InboxState>, x: f32, y: f32) -> Result<()> {
        let point = fission_core::LayoutPoint::new(x, y);
        h.send_event(InputEvent::Pointer(PointerEvent::Down {
            point,
            button: PointerButton::Primary,
        }))?;
        h.send_event(InputEvent::Pointer(PointerEvent::Up {
            point,
            button: PointerButton::Primary,
        }))?;
        Ok(())
    }

    #[test]
    fn settings_modal_backdrop_closes() -> Result<()> {
        let mut state = InboxState::default();
        state.show_settings = true;
        let mut h = TestHarness::new(state).with_root_widget(InboxApp);
        h.pump()?;

        click(&mut h, 10.0, 10.0)?;

        let state = h.runtime.get_app_state::<InboxState>().unwrap();
        assert!(!state.show_settings, "settings modal should close on backdrop click");
        Ok(())
    }

    #[test]
    fn contacts_modal_backdrop_closes() -> Result<()> {
        let mut state = InboxState::default();
        state.show_contacts = true;
        let mut h = TestHarness::new(state).with_root_widget(InboxApp);
        h.pump()?;

        click(&mut h, 10.0, 10.0)?;

        let state = h.runtime.get_app_state::<InboxState>().unwrap();
        assert!(!state.show_contacts, "contacts modal should close on backdrop click");
        Ok(())
    }

    #[test]
    fn compose_modal_backdrop_closes() -> Result<()> {
        let mut state = InboxState::default();
        state.show_compose = true;
        let mut h = TestHarness::new(state).with_root_widget(InboxApp);
        h.pump()?;

        click(&mut h, 10.0, 10.0)?;

        let state = h.runtime.get_app_state::<InboxState>().unwrap();
        assert!(!state.show_compose, "compose modal should close on backdrop click");
        Ok(())
    }

    #[test]
    fn mobile_drawer_backdrop_closes() -> Result<()> {
        let mut state = InboxState::default();
        state.show_mobile_menu = true;
        let mut h = TestHarness::new(state).with_root_widget(InboxApp);
        h.pump()?;

        click(&mut h, 700.0, 20.0)?;

        let state = h.runtime.get_app_state::<InboxState>().unwrap();
        assert!(!state.show_mobile_menu, "mobile drawer should close on backdrop click");
        Ok(())
    }
}
