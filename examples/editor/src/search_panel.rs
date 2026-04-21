use crate::model::{EditorState, ExecuteSearch, OpenFile, UpdateSearchQuery};
use fission_core::op::Color;
use fission_core::ui::{Button, ButtonContentAlign, ButtonVariant, Container, Node, Scroll, Text, TextInput};
use fission_core::{ActionEnvelope, BuildCtx, FlexDirection, Handler, View, Widget};
use fission_widgets::{VStack, HStack, Spacer};
use serde_json;

pub struct SearchPanel;

impl Widget<EditorState> for SearchPanel {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        let tokens = &view.env.theme.tokens;
        let text_color = Color { r: 204, g: 204, b: 204, a: 255 };
        let dim_color = Color { r: 140, g: 140, b: 140, a: 255 };

        let update_query = ctx.bind(
            UpdateSearchQuery(String::new()),
            (|s: &mut EditorState, a: UpdateSearchQuery, _| s.search_query = a.0)
                as Handler<EditorState, UpdateSearchQuery>,
        );

        let execute = ctx.bind(
            ExecuteSearch,
            (|s: &mut EditorState, _, _| s.run_search())
                as Handler<EditorState, ExecuteSearch>,
        );

        let open_id = ctx.bind(
            OpenFile(String::new()),
            (|s: &mut EditorState, a: OpenFile, _| s.open_file(a.0))
                as Handler<EditorState, OpenFile>,
        ).id;

        let mut children = vec![
            // Search input row
            HStack {
                spacing: Some(4.0),
                children: vec![
                    TextInput {
                        value: view.state.search_query.clone(),
                        placeholder: Some("Search...".into()),
                        on_change: Some(update_query),
                        ..Default::default()
                    }.into_node(),
                    Button {
                        variant: ButtonVariant::Outline,
                        child: Some(Box::new(Text::new("Go").size(12.0).into_node())),
                        on_press: Some(execute),
                        width: Some(36.0),
                        height: Some(32.0),
                        ..Default::default()
                    }.into_node(),
                ],
            }.into_node(),
        ];

        // Results
        if !view.state.search_results.is_empty() {
            children.push(
                Text::new(format!("{} results", view.state.search_results.len()))
                    .size(11.0)
                    .color(dim_color)
                    .into_node(),
            );

            let mut result_nodes = Vec::new();
            for result in view.state.search_results.iter().take(50) {
                let label = format!("{}:{}", result.path.rsplit('/').next().unwrap_or(&result.path), result.line);
                result_nodes.push(
                    Button {
                        variant: ButtonVariant::Ghost,
                        content_align: ButtonContentAlign::Start,
                        child: Some(Box::new(
                            VStack {
                                spacing: Some(1.0),
                                children: vec![
                                    Text::new(label).size(12.0).color(text_color).into_node(),
                                    Text::new(result.context.chars().take(60).collect::<String>())
                                        .size(11.0)
                                        .color(dim_color)
                                        .into_node(),
                                ],
                            }.into_node(),
                        )),
                        on_press: Some(ActionEnvelope {
                            id: open_id,
                            payload: serde_json::to_vec(&OpenFile(result.path.clone())).unwrap(),
                        }),
                        padding: Some([4.0, 4.0, 0.0, 0.0]),
                        ..Default::default()
                    }.into_node(),
                );
            }

            children.push(
                Scroll {
                    direction: FlexDirection::Column,
                    child: Some(Box::new(
                        VStack { spacing: Some(2.0), children: result_nodes }.into_node(),
                    )),
                    show_scrollbar: true,
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    ..Default::default()
                }.into_node(),
            );
        } else if !view.state.search_query.is_empty() {
            children.push(
                Text::new("No results found")
                    .size(12.0)
                    .color(dim_color)
                    .into_node(),
            );
        }

        Container::new(
            VStack { spacing: Some(8.0), children }.into_node(),
        )
        .padding_all(8.0)
        .flex_grow(1.0)
        .into_node()
    }
}
