use fission_widgets::{
    Button, Column, Container, Icon, Row, Scroll, Text, TextContent, VStack, BuildCtx, View, Node, Widget, Tooltip, TextInput,
    MenuButton, MenuItem, Toast, ToastKind, Select, SelectItem,
};
use fission_core::op::{Color, BoxShadow};
use fission_core::{AppState, WidgetNodeId, ActionEnvelope, ActionId};
use fission_icons::material;
use fission_shell_desktop::DesktopApp;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, fission_macros::Action)]
struct ToggleToast;

#[derive(Clone, Debug, Serialize, Deserialize, fission_macros::Action)]
struct ToggleMenu;

#[derive(Clone, Debug, Serialize, Deserialize, fission_macros::Action)]
struct ToggleSelect;

#[derive(Clone, Debug, Serialize, Deserialize, fission_macros::Action)]
struct SelectVariant(String);

#[derive(Default, Clone, Debug)]
struct State {
    filter: String,
    multiline_value: String,
    show_toast: bool,
    show_menu: bool,
    show_select: bool,
    selected_variant: Option<String>,
}

impl AppState for State {}

struct IconsApp;

impl Widget<State> for IconsApp {
    fn build(&self, ctx: &mut BuildCtx<State>, view: &View<State>) -> Node {
        let title = Text::new("Material Icons Gallery")
            .size(32.0);

        let mut multiline_text = view.state.multiline_value.clone();
        if multiline_text.is_empty() {
            multiline_text = "This is a multiline text input.\nIt should contribute its real height to the layout.\nTry scrolling the gallery below!".into();
        }

        let input = TextInput {
            value: multiline_text,
            multiline: true,
            width: Some(400.0),
            ..Default::default()
        }.into_node();

        // Control Row
        let controls = Row {
            gap: Some(16.0),
            children: vec![
                Button {
                    child: Some(Box::new(Text::new("Toggle Toast").into_node())),
                    on_press: Some(ctx.bind(ToggleToast, |s: &mut State, _| s.show_toast = !s.show_toast)),
                    ..Default::default()
                }.into_node(),
                MenuButton {
                    id: WidgetNodeId::explicit("demo_menu"),
                    label: "Options".into(),
                    is_open: view.state.show_menu,
                    on_toggle: Some(ctx.bind(ToggleMenu, |s: &mut State, _| s.show_menu = !s.show_menu)),
                    items: vec![
                        MenuItem { label: "Item 1".into(), icon: Some(material::action::home::regular().into()), on_select: None },
                        MenuItem { label: "Item 2".into(), icon: None, on_select: None },
                    ],
                }.build(ctx, view),
                Select {
                    id: WidgetNodeId::explicit("variant_select"),
                    selected_label: view.state.selected_variant.clone(),
                    placeholder: "Choose Variant".into(),
                    is_open: view.state.show_select,
                    on_toggle: Some(ctx.bind(ToggleSelect, |s: &mut State, _| s.show_select = !s.show_select)),
                    items: vec![
                        SelectItem { 
                            label: "Regular".into(), 
                            icon: None, 
                            on_select: ctx.bind(SelectVariant("Regular".into()), |s, a| { s.selected_variant = Some(a.0); s.show_select = false; })
                        },
                        SelectItem { 
                            label: "Outlined".into(), 
                            icon: None, 
                            on_select: ctx.bind(SelectVariant("Outlined".into()), |s, a| { s.selected_variant = Some(a.0); s.show_select = false; })
                        },
                    ],
                    width: Some(180.0),
                }.build(ctx, view),
            ],
            ..Default::default()
        }.into_node();

        // Group icons by (Category, Name)
        let all = fission_icons::material::all_icons();
        let mut grouped: HashMap<(String, String), HashMap<String, fn() -> &'static str>> = HashMap::new();
        
        for (cat, name, variant, func) in all {
            grouped.entry((cat.to_string(), name.to_string()))
                .or_default()
                .insert(variant.to_string(), func);
        }

        let mut keys: Vec<_> = grouped.keys().cloned().collect();
        keys.sort();

        let mut grid_items = Vec::new();

        // Limit to first 200 for performance
        for (idx, (cat, name)) in keys.into_iter().take(200).enumerate() {
            let variants = &grouped[&(cat.clone(), name.clone())];
            
            if let (Some(regular), Some(outlined)) = (variants.get("regular"), variants.get("outlined")) {
                let mut regular_node = Icon::svg(regular()).size(32.0).into_node();
                
                // Add tooltip to the very first icon to verify Flyout positioning
                if idx == 0 {
                    regular_node = Tooltip {
                        id: WidgetNodeId::explicit("verify_flyout_tooltip"),
                        child: Box::new(regular_node),
                        text: "This tooltip should scroll with the icon!".into(),
                    }.build(ctx, view);
                }

                let card = Container::new(
                    VStack {
                        spacing: Some(8.0),
                        children: vec![
                            Text::new(format!("{} / {}", cat, name)).size(12.0).color(Color { r: 100, g: 100, b: 100, a: 255 }).into_node(),
                            Row {
                                gap: Some(16.0),
                                children: vec![
                                    VStack {
                                        spacing: Some(4.0),
                                        children: vec![
                                            regular_node,
                                            Text::new("Regular").size(10.0).color(Color::BLACK).into_node(),
                                        ]
                                    }.into_node(),
                                    VStack {
                                        spacing: Some(4.0),
                                        children: vec![
                                            Icon::svg(outlined()).size(32.0).into_node(),
                                            Text::new("Outlined").size(10.0).color(Color::BLACK).into_node(),
                                        ]
                                    }.into_node(),
                                ],
                                ..Default::default()
                            }.into_node()
                        ]
                    }.into_node()
                )
                .padding_all(16.0)
                .bg(Color::WHITE)
                .border_radius(8.0)
                .shadow(BoxShadow { 
                    color: Color { r: 0, g: 0, b: 0, a: 20 }, 
                    blur_radius: 4.0, 
                    offset: (0.0, 2.0) 
                })
                .into_node();
    
                grid_items.push(card);
            }
        }

        // Grid layout simulation with Rows
        let rows: Vec<Node> = grid_items.chunks(3).map(|chunk| {
            Row {
                gap: Some(16.0),
                children: chunk.to_vec(),
                ..Default::default()
            }.into_node()
        }).collect();

        let content = Scroll {
            child: Some(Box::new(
                VStack {
                    spacing: Some(16.0),
                    children: rows,
                }.into_node()
            )),
            height: Some(500.0), // Reduced to fit within default window (600px)
            show_scrollbar: true,
            ..Default::default()
        };

        let main_content = Container::new(
            VStack {
                spacing: Some(24.0),
                children: vec![
                    title.into_node(),
                    controls,
                    input,
                    content.into_node(),
                ]
            }.into_node()
        )
        .padding_all(24.0)
        .bg(Color { r: 245, g: 245, b: 245, a: 255 })
        .flex_grow(1.0)
        .into_node();

        // Wrap in ZStack to support Toast overlay
        let mut layers = vec![main_content];

        if view.state.show_toast {
            // Position toast at bottom center
            // fission-core doesn't have Align widget yet, so use Positioned with explicit coordinates
            // Assuming 800x600 window approx.
            // TODO: Use real window constraints when available
            let toast = Toast {
                id: WidgetNodeId::explicit("demo_toast"),
                kind: ToastKind::Info,
                message: "This is a toast notification".into(),
                on_close: Some(ctx.bind(ToggleToast, |s: &mut State, _| s.show_toast = false)),
            }.build(ctx, view);

            layers.push(
                fission_widgets::Positioned {
                    left: Some(250.0), // Approximate center
                    bottom: Some(20.0),
                    width: None, height: None,
                    child: Some(Box::new(toast)),
                    ..Default::default()
                }.into_node()
            );
        }

        fission_widgets::ZStack {
            children: layers,
            ..Default::default()
        }.into_node()
    }
}

fn main() -> anyhow::Result<()> {
    let app = DesktopApp::new(IconsApp);
    app.run()
}
