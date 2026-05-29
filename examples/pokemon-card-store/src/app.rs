use crate::components::{card_grid::CardGrid, hero::Hero, shell::StoreShell};
use crate::data::{CatalogRequest, CatalogResponse, StoreError, CATALOG_JOB};
use fission::core::ResourceKey;
use fission::prelude::*;

#[derive(Debug, Clone)]
pub struct StoreState {
    pub catalog: AsyncSnapshot<CatalogResponse, StoreError>,
    pub cart_count: u32,
}

impl Default for StoreState {
    fn default() -> Self {
        Self {
            catalog: AsyncSnapshot::waiting(),
            cart_count: 0,
        }
    }
}

impl AppState for StoreState {}

#[derive(Clone)]
pub struct StoreHomePage;

impl Widget<StoreState> for StoreHomePage {
    fn build(&self, ctx: &mut BuildCtx<StoreState>, view: &View<StoreState>) -> Node {
        let catalog_loaded = with_reducer!(ctx, CatalogLoaded, on_catalog_loaded);
        let catalog_failed = with_reducer!(ctx, CatalogFailed, on_catalog_failed);
        let add_demo_card = with_reducer!(ctx, AddDemoCard, on_add_demo_card);
        let _ = add_demo_card;

        let catalog_request = CatalogRequest { generation: 1 };
        let catalog_snapshot = view.state.catalog.clone();
        let card_grid = FutureBuilder::new(
            ResourceKey::new("pokemon-card-store.catalog"),
            CATALOG_JOB,
            catalog_request.clone(),
            catalog_snapshot,
            |ctx, view, snapshot| {
                CardGrid {
                    snapshot: snapshot.clone(),
                }
                .build(ctx, view)
            },
        )
        .deps(catalog_request)
        .on_ok(catalog_loaded)
        .on_err(catalog_failed)
        .build(ctx, view);

        StoreShell {
            child: Column {
                gap: Some(28.0),
                children: vec![
                    Hero.build(ctx, view),
                    cart_summary(view.state.cart_count),
                    card_grid,
                ],
                ..Default::default()
            }
            .into_node(),
        }
        .build(ctx, view)
    }
}

#[fission_reducer(CatalogLoaded)]
pub fn on_catalog_loaded(state: &mut StoreState, ctx: &mut ReducerContext<StoreState>) {
    if let Some(catalog) = ctx.input.job_ok(CATALOG_JOB) {
        state.catalog = AsyncSnapshot::with_data(AsyncConnectionState::Done, catalog);
    }
}

#[fission_reducer(CatalogFailed)]
pub fn on_catalog_failed(state: &mut StoreState, ctx: &mut ReducerContext<StoreState>) {
    let error = ctx
        .input
        .job_err(CATALOG_JOB)
        .unwrap_or_else(|| StoreError {
            message: ctx
                .input
                .job_error_message(CATALOG_JOB)
                .unwrap_or("Unable to load the catalogue")
                .to_string(),
        });
    state.catalog = AsyncSnapshot::with_error(AsyncConnectionState::Done, error);
}

#[fission_reducer(AddDemoCard)]
pub fn on_add_demo_card(state: &mut StoreState) {
    state.cart_count = state.cart_count.saturating_add(1);
}

fn cart_summary(cart_count: u32) -> Node {
    Container::new(
        Row {
            gap: Some(12.0),
            align_items: ir_op::AlignItems::Center,
            children: vec![
                Text::new(format!(
                    "{} {} in the server cart",
                    cart_count,
                    if cart_count == 1 { "item" } else { "items" }
                ))
                .size(16.0)
                .line_height(22.0)
                .weight(800)
                .color(color(219, 234, 254))
                .into_node(),
                Spacer {
                    flex_grow: 1.0,
                    ..Default::default()
                }
                .into_node(),
                Text::new("Signed actions post back to the server reducer path")
                    .size(13.0)
                    .line_height(18.0)
                    .color(color(147, 197, 253))
                    .into_node(),
            ],
            ..Default::default()
        }
        .into_node(),
    )
    .padding([14.0, 14.0, 12.0, 12.0])
    .border(color(59, 130, 246).with_alpha(120), 1.0)
    .border_radius(18.0)
    .bg(color(30, 64, 175).with_alpha(80))
    .into_node()
}

fn color(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}
