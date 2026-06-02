use super::custom_render::CustomRenderObject;
use super::traits::{InternalLower, InternalLowerer};
use super::widgets::{
    ActionScope, Align, Button, Checkbox, Clip, Column, Composite, Container, FocusScope,
    GestureDetector, Grid, GridItem, Icon, Image, LazyColumn, Overlay, Positioned, Radio, RichText,
    Row, SafeArea, Scroll, SemanticsRegion, Slider, Spacer, Switch, Text, TextInput, Transform,
    Video, ZStack,
};
use crate::lowering::InternalLoweringCx;
use fission_ir::{Op, StructuralOp, WidgetId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Widget {
    kind: Box<WidgetKind>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum WidgetKind {
    Identified { id: WidgetId, child: Widget },
    ActionScope(ActionScope),
    Row(Row),
    Column(Column),
    Align(Align),
    FocusScope(FocusScope),
    Clip(Clip),
    Text(Text),
    RichText(RichText),
    Transform(Transform),
    Button(Button),
    TextInput(TextInput),
    Scroll(Scroll),
    SemanticsRegion(SemanticsRegion),
    Image(Image),
    Video(Video),
    ZStack(ZStack),
    Overlay(Overlay),
    Container(Container),
    GestureDetector(GestureDetector),
    Grid(Grid),
    GridItem(GridItem),
    Checkbox(Checkbox),
    Switch(Switch),
    Radio(Radio),
    SafeArea(SafeArea),
    Positioned(Positioned),
    Spacer(Spacer),
    Slider(Slider),
    LazyColumn(LazyColumn),
    Icon(Icon),
    Composite(Composite),
    Custom(InternalRenderNode),
}

impl Widget {
    pub(crate) fn with_id(self, id: WidgetId) -> Self {
        let kind = match *self.kind {
            WidgetKind::Identified { child, .. } => WidgetKind::Identified { id, child },
            WidgetKind::ActionScope(w) => WidgetKind::Identified {
                id,
                child: Widget {
                    kind: Box::new(WidgetKind::ActionScope(w)),
                },
            },
            WidgetKind::Custom(w) => WidgetKind::Identified {
                id,
                child: Widget {
                    kind: Box::new(WidgetKind::Custom(w)),
                },
            },
            WidgetKind::Row(mut w) => {
                w.id = Some(id);
                WidgetKind::Row(w)
            }
            WidgetKind::Column(mut w) => {
                w.id = Some(id);
                WidgetKind::Column(w)
            }
            WidgetKind::Align(mut w) => {
                w.id = Some(id);
                WidgetKind::Align(w)
            }
            WidgetKind::FocusScope(mut w) => {
                w.id = Some(id);
                WidgetKind::FocusScope(w)
            }
            WidgetKind::Clip(mut w) => {
                w.id = Some(id);
                WidgetKind::Clip(w)
            }
            WidgetKind::Text(mut w) => {
                w.id = Some(id);
                WidgetKind::Text(w)
            }
            WidgetKind::RichText(mut w) => {
                w.id = Some(id);
                WidgetKind::RichText(w)
            }
            WidgetKind::Transform(mut w) => {
                w.id = Some(id);
                WidgetKind::Transform(w)
            }
            WidgetKind::Button(mut w) => {
                w.id = Some(id);
                WidgetKind::Button(w)
            }
            WidgetKind::TextInput(mut w) => {
                w.id = Some(id);
                WidgetKind::TextInput(w)
            }
            WidgetKind::Scroll(mut w) => {
                w.id = Some(id);
                WidgetKind::Scroll(w)
            }
            WidgetKind::SemanticsRegion(mut w) => {
                w.id = Some(id);
                WidgetKind::SemanticsRegion(w)
            }
            WidgetKind::Image(mut w) => {
                w.id = Some(id);
                WidgetKind::Image(w)
            }
            WidgetKind::Video(mut w) => {
                w.id = Some(id);
                WidgetKind::Video(w)
            }
            WidgetKind::ZStack(mut w) => {
                w.id = Some(id);
                WidgetKind::ZStack(w)
            }
            WidgetKind::Overlay(mut w) => {
                w.id = Some(id);
                WidgetKind::Overlay(w)
            }
            WidgetKind::Container(mut w) => {
                w.id = Some(id);
                WidgetKind::Container(w)
            }
            WidgetKind::GestureDetector(mut w) => {
                w.id = Some(id);
                WidgetKind::GestureDetector(w)
            }
            WidgetKind::Grid(mut w) => {
                w.id = Some(id);
                WidgetKind::Grid(w)
            }
            WidgetKind::GridItem(mut w) => {
                w.id = Some(id);
                WidgetKind::GridItem(w)
            }
            WidgetKind::Checkbox(mut w) => {
                w.id = Some(id);
                WidgetKind::Checkbox(w)
            }
            WidgetKind::Switch(mut w) => {
                w.id = Some(id);
                WidgetKind::Switch(w)
            }
            WidgetKind::Radio(mut w) => {
                w.id = Some(id);
                WidgetKind::Radio(w)
            }
            WidgetKind::SafeArea(mut w) => {
                w.id = Some(id);
                WidgetKind::SafeArea(w)
            }
            WidgetKind::Positioned(mut w) => {
                w.id = Some(id);
                WidgetKind::Positioned(w)
            }
            WidgetKind::Spacer(mut w) => {
                w.id = Some(id);
                WidgetKind::Spacer(w)
            }
            WidgetKind::Slider(mut w) => {
                w.id = Some(id);
                WidgetKind::Slider(w)
            }
            WidgetKind::LazyColumn(mut w) => {
                w.id = Some(id);
                WidgetKind::LazyColumn(w)
            }
            WidgetKind::Icon(mut w) => {
                w.id = Some(id);
                WidgetKind::Icon(w)
            }
            WidgetKind::Composite(mut w) => {
                w.id = Some(id);
                WidgetKind::Composite(w)
            }
        };
        Self {
            kind: Box::new(kind),
        }
    }

    pub fn id<I>(self, id: I) -> Self
    where
        I: Into<WidgetId>,
    {
        self.with_id(id.into())
    }

    pub(crate) fn custom(node: InternalRenderNode) -> Self {
        Self {
            kind: Box::new(WidgetKind::Custom(node)),
        }
    }

    pub(crate) fn into_text(self) -> Result<Text, Self> {
        match *self.kind {
            WidgetKind::Text(text) => Ok(text),
            kind => Err(Self {
                kind: Box::new(kind),
            }),
        }
    }

    pub(crate) fn kind_name(&self) -> &'static str {
        match &*self.kind {
            WidgetKind::Identified { .. } => "Identified",
            WidgetKind::ActionScope(_) => "ActionScope",
            WidgetKind::Row(_) => "Row",
            WidgetKind::Column(_) => "Column",
            WidgetKind::Align(_) => "Align",
            WidgetKind::FocusScope(_) => "FocusScope",
            WidgetKind::Clip(_) => "Clip",
            WidgetKind::Text(_) => "Text",
            WidgetKind::RichText(_) => "RichText",
            WidgetKind::Transform(_) => "Transform",
            WidgetKind::Button(_) => "Button",
            WidgetKind::TextInput(_) => "TextInput",
            WidgetKind::Scroll(_) => "Scroll",
            WidgetKind::SemanticsRegion(_) => "SemanticsRegion",
            WidgetKind::Image(_) => "Image",
            WidgetKind::Video(_) => "Video",
            WidgetKind::ZStack(_) => "ZStack",
            WidgetKind::Overlay(_) => "Overlay",
            WidgetKind::Container(_) => "Container",
            WidgetKind::GestureDetector(_) => "GestureDetector",
            WidgetKind::Grid(_) => "Grid",
            WidgetKind::GridItem(_) => "GridItem",
            WidgetKind::Checkbox(_) => "Checkbox",
            WidgetKind::Switch(_) => "Switch",
            WidgetKind::Radio(_) => "Radio",
            WidgetKind::SafeArea(_) => "SafeArea",
            WidgetKind::Positioned(_) => "Positioned",
            WidgetKind::Spacer(_) => "Spacer",
            WidgetKind::Slider(_) => "Slider",
            WidgetKind::LazyColumn(_) => "LazyColumn",
            WidgetKind::Icon(_) => "Icon",
            WidgetKind::Composite(_) => "Composite",
            WidgetKind::Custom(_) => "Custom",
        }
    }

    pub(crate) fn devtools_explicit_id(&self) -> Option<WidgetId> {
        match &*self.kind {
            WidgetKind::Identified { id, .. } => Some(*id),
            WidgetKind::ActionScope(_) => None,
            WidgetKind::Row(w) => w.id,
            WidgetKind::Column(w) => w.id,
            WidgetKind::Align(w) => w.id,
            WidgetKind::FocusScope(w) => w.id,
            WidgetKind::Clip(w) => w.id,
            WidgetKind::Text(w) => w.id,
            WidgetKind::RichText(w) => w.id,
            WidgetKind::Transform(w) => w.id,
            WidgetKind::Button(w) => w.id,
            WidgetKind::TextInput(w) => w.id,
            WidgetKind::Scroll(w) => w.id,
            WidgetKind::SemanticsRegion(w) => w.id,
            WidgetKind::Image(w) => w.id,
            WidgetKind::Video(w) => w.id,
            WidgetKind::ZStack(w) => w.id,
            WidgetKind::Overlay(w) => w.id,
            WidgetKind::Container(w) => w.id,
            WidgetKind::GestureDetector(w) => w.id,
            WidgetKind::Grid(w) => w.id,
            WidgetKind::GridItem(w) => w.id,
            WidgetKind::Checkbox(w) => w.id,
            WidgetKind::Switch(w) => w.id,
            WidgetKind::Radio(w) => w.id,
            WidgetKind::SafeArea(w) => w.id,
            WidgetKind::Positioned(w) => w.id,
            WidgetKind::Spacer(w) => w.id,
            WidgetKind::Slider(w) => w.id,
            WidgetKind::LazyColumn(w) => w.id,
            WidgetKind::Icon(w) => w.id,
            WidgetKind::Composite(w) => w.id,
            WidgetKind::Custom(_) => None,
        }
    }

    pub(crate) fn devtools_children(&self) -> Vec<&Widget> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => vec![child],
            WidgetKind::ActionScope(w) => vec![&w.child],
            WidgetKind::Row(w) => w.children.iter().collect(),
            WidgetKind::Column(w) => w.children.iter().collect(),
            WidgetKind::Align(w) => vec![&w.child],
            WidgetKind::FocusScope(w) => w.children.iter().collect(),
            WidgetKind::Clip(w) => vec![&w.child],
            WidgetKind::RichText(w) => w.inline_widgets.iter().map(|span| &span.widget).collect(),
            WidgetKind::Transform(w) => vec![&w.child],
            WidgetKind::Button(w) => w.child.iter().collect(),
            WidgetKind::Scroll(w) => w.child.iter().collect(),
            WidgetKind::SemanticsRegion(w) => w.child.iter().collect(),
            WidgetKind::ZStack(w) => w.children.iter().collect(),
            WidgetKind::Overlay(w) => vec![&w.content, &w.overlay],
            WidgetKind::Container(w) => w.child.iter().collect(),
            WidgetKind::GestureDetector(w) => vec![&w.child],
            WidgetKind::Grid(w) => w.children.iter().collect(),
            WidgetKind::GridItem(w) => vec![&w.child],
            WidgetKind::SafeArea(w) => vec![&w.child],
            WidgetKind::Positioned(w) => w.child.iter().collect(),
            WidgetKind::LazyColumn(w) => w.children.iter().collect(),
            WidgetKind::Composite(w) => vec![&w.child],
            WidgetKind::Text(_)
            | WidgetKind::TextInput(_)
            | WidgetKind::Image(_)
            | WidgetKind::Video(_)
            | WidgetKind::Checkbox(_)
            | WidgetKind::Switch(_)
            | WidgetKind::Radio(_)
            | WidgetKind::Spacer(_)
            | WidgetKind::Slider(_)
            | WidgetKind::Icon(_)
            | WidgetKind::Custom(_) => Vec::new(),
        }
    }

    pub(crate) fn devtools_debug_label(&self) -> Option<String> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.devtools_debug_label(),
            WidgetKind::Text(w) => Some(format!("{:?}", w.content)),
            WidgetKind::RichText(w) => {
                let text = w
                    .runs
                    .iter()
                    .map(|run| run.text.as_str())
                    .collect::<Vec<_>>()
                    .join("");
                (!text.is_empty()).then_some(text)
            }
            WidgetKind::Button(w) => w
                .semantics
                .as_ref()
                .and_then(|semantics| semantics.label.clone()),
            WidgetKind::TextInput(w) => w
                .label
                .as_ref()
                .or(w.placeholder.as_ref())
                .map(|content| format!("{content:?}")),
            WidgetKind::Image(w) => Some(format!("{:?}", w.request.source)),
            WidgetKind::Video(w) => Some(w.source.clone()),
            WidgetKind::Checkbox(w) => w.label.clone(),
            WidgetKind::Radio(w) => w.label.clone(),
            WidgetKind::SemanticsRegion(w) => w.label.clone().or_else(|| w.identifier.clone()),
            WidgetKind::Icon(w) => Some(format!("{:?}", w.source)),
            WidgetKind::Custom(w) => Some(w.debug_tag.clone()),
            _ => None,
        }
    }

    pub(crate) fn devtools_properties(&self) -> BTreeMap<String, String> {
        let mut properties = BTreeMap::new();
        if let Some(id) = self.devtools_explicit_id() {
            properties.insert("widget_id".into(), id.as_u128().to_string());
        }
        let child_count = self.devtools_children().len();
        if child_count > 0 {
            properties.insert("child_count".into(), child_count.to_string());
        }

        match &*self.kind {
            WidgetKind::Identified { id, .. } => {
                properties.insert("identified_id".into(), id.as_u128().to_string());
            }
            WidgetKind::Row(w) => {
                insert_optional(&mut properties, "gap", w.gap);
                properties.insert("flex_grow".into(), w.flex_grow.to_string());
                properties.insert("flex_shrink".into(), w.flex_shrink.to_string());
                properties.insert("align_items".into(), format!("{:?}", w.align_items));
                properties.insert("justify_content".into(), format!("{:?}", w.justify_content));
            }
            WidgetKind::Column(w) => {
                insert_optional(&mut properties, "gap", w.gap);
                properties.insert("flex_grow".into(), w.flex_grow.to_string());
                properties.insert("flex_shrink".into(), w.flex_shrink.to_string());
                properties.insert("align_items".into(), format!("{:?}", w.align_items));
                properties.insert("justify_content".into(), format!("{:?}", w.justify_content));
            }
            WidgetKind::Button(w) => {
                insert_optional(&mut properties, "width", w.width);
                insert_optional(&mut properties, "height", w.height);
                properties.insert("variant".into(), format!("{:?}", w.variant));
                properties.insert("disabled".into(), w.disabled.to_string());
                properties.insert("has_action".into(), w.on_press.is_some().to_string());
            }
            WidgetKind::Container(w) => {
                insert_optional(&mut properties, "width", w.width);
                insert_optional(&mut properties, "height", w.height);
                insert_optional(&mut properties, "border_radius", Some(w.border_radius));
                properties.insert(
                    "has_background".into(),
                    w.background_fill.is_some().to_string(),
                );
            }
            WidgetKind::Scroll(w) => {
                insert_optional(&mut properties, "width", w.width);
                insert_optional(&mut properties, "height", w.height);
                properties.insert("direction".into(), format!("{:?}", w.direction));
                properties.insert("show_scrollbar".into(), w.show_scrollbar.to_string());
            }
            WidgetKind::Text(w) => {
                insert_optional(&mut properties, "width", w.width);
                insert_optional(&mut properties, "height", w.height);
                insert_optional(&mut properties, "size", w.font_size);
                properties.insert("wrap".into(), w.wrap.to_string());
            }
            WidgetKind::RichText(w) => {
                insert_optional(&mut properties, "width", w.width);
                insert_optional(&mut properties, "height", w.height);
                properties.insert("runs".into(), w.runs.len().to_string());
                properties.insert("inline_widgets".into(), w.inline_widgets.len().to_string());
                properties.insert("wrap".into(), w.wrap.to_string());
            }
            WidgetKind::TextInput(w) => {
                insert_optional(&mut properties, "width", w.width);
                insert_optional(&mut properties, "height", w.height);
                properties.insert("text_len".into(), w.value.len().to_string());
                properties.insert("multiline".into(), w.multiline.to_string());
                properties.insert("enabled".into(), w.enabled.to_string());
                properties.insert("read_only".into(), w.read_only.to_string());
            }
            WidgetKind::Image(w) => {
                insert_optional(&mut properties, "width", w.width);
                insert_optional(&mut properties, "height", w.height);
                properties.insert("fit".into(), format!("{:?}", w.fit));
            }
            WidgetKind::Video(w) => {
                insert_optional(&mut properties, "width", w.width);
                insert_optional(&mut properties, "height", w.height);
                properties.insert("autoplay".into(), w.autoplay.to_string());
                properties.insert("loop_playback".into(), w.loop_playback.to_string());
            }
            WidgetKind::Grid(w) => {
                properties.insert("columns".into(), w.columns.len().to_string());
                properties.insert("rows".into(), w.rows.len().to_string());
                insert_optional(&mut properties, "column_gap", w.column_gap);
                insert_optional(&mut properties, "row_gap", w.row_gap);
            }
            WidgetKind::GridItem(w) => {
                properties.insert("row_start".into(), format!("{:?}", w.row_start));
                properties.insert("row_end".into(), format!("{:?}", w.row_end));
                properties.insert("col_start".into(), format!("{:?}", w.col_start));
                properties.insert("col_end".into(), format!("{:?}", w.col_end));
            }
            WidgetKind::Checkbox(w) => {
                properties.insert("checked".into(), w.checked.to_string());
                properties.insert("has_action".into(), w.on_toggle.is_some().to_string());
            }
            WidgetKind::Switch(w) => {
                properties.insert("checked".into(), w.checked.to_string());
                properties.insert("has_action".into(), w.on_toggle.is_some().to_string());
            }
            WidgetKind::Radio(w) => {
                properties.insert("checked".into(), w.checked.to_string());
                properties.insert("has_action".into(), w.on_select.is_some().to_string());
            }
            WidgetKind::Spacer(w) => {
                insert_optional(&mut properties, "width", w.width);
                insert_optional(&mut properties, "height", w.height);
            }
            WidgetKind::Slider(w) => {
                properties.insert("value".into(), w.value.to_string());
                properties.insert("min".into(), w.min.to_string());
                properties.insert("max".into(), w.max.to_string());
                properties.insert("has_action".into(), w.on_change.is_some().to_string());
            }
            WidgetKind::Icon(w) => {
                insert_optional(&mut properties, "size", w.size);
            }
            WidgetKind::Custom(w) => {
                properties.insert("debug_tag".into(), w.debug_tag.clone());
                properties.insert(
                    "has_render_object".into(),
                    w.render_object.is_some().to_string(),
                );
            }
            _ => {}
        }
        properties
    }

    pub(crate) fn as_row(&self) -> Option<&Row> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.as_row(),
            WidgetKind::Row(widget) => Some(widget),
            _ => None,
        }
    }

    pub(crate) fn as_column(&self) -> Option<&Column> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.as_column(),
            WidgetKind::Column(widget) => Some(widget),
            _ => None,
        }
    }

    pub(crate) fn as_container(&self) -> Option<&Container> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.as_container(),
            WidgetKind::Container(widget) => Some(widget),
            _ => None,
        }
    }

    pub(crate) fn as_scroll(&self) -> Option<&Scroll> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.as_scroll(),
            WidgetKind::Scroll(widget) => Some(widget),
            _ => None,
        }
    }

    pub(crate) fn as_rich_text(&self) -> Option<&RichText> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.as_rich_text(),
            WidgetKind::RichText(widget) => Some(widget),
            _ => None,
        }
    }

    pub(crate) fn as_text(&self) -> Option<&Text> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.as_text(),
            WidgetKind::Text(widget) => Some(widget),
            _ => None,
        }
    }

    pub(crate) fn as_text_input(&self) -> Option<&TextInput> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.as_text_input(),
            WidgetKind::TextInput(widget) => Some(widget),
            _ => None,
        }
    }

    pub(crate) fn as_button(&self) -> Option<&Button> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.as_button(),
            WidgetKind::Button(widget) => Some(widget),
            _ => None,
        }
    }

    pub(crate) fn as_gesture_detector(&self) -> Option<&GestureDetector> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.as_gesture_detector(),
            WidgetKind::GestureDetector(widget) => Some(widget),
            _ => None,
        }
    }

    pub(crate) fn as_zstack(&self) -> Option<&ZStack> {
        match &*self.kind {
            WidgetKind::Identified { child, .. } => child.as_zstack(),
            WidgetKind::ZStack(widget) => Some(widget),
            _ => None,
        }
    }
}

pub trait WidgetIdExt: Into<Widget> + Sized {
    fn id<I>(self, id: I) -> Widget
    where
        I: Into<WidgetId>,
    {
        let id = id.into();
        crate::build::with_widget_id(id, || {
            let widget: Widget = self.into();
            widget.with_id(id)
        })
    }
}

impl<T> WidgetIdExt for T where T: Into<Widget> {}

fn insert_optional<T: ToString>(
    properties: &mut BTreeMap<String, String>,
    key: &'static str,
    value: Option<T>,
) {
    if let Some(value) = value {
        properties.insert(key.to_string(), value.to_string());
    }
}

impl Widget {
    pub(crate) fn lower(&self, cx: &mut InternalLoweringCx) -> WidgetId {
        match &*self.kind {
            WidgetKind::Identified { id, child } => {
                let child_id = child.lower(cx);
                let mut builder = crate::lowering::InternalIrBuilder::new(
                    (*id).into(),
                    Op::Structural(StructuralOp::Group {
                        stable_hash: id.as_u128() as u64,
                    }),
                );
                builder.add_child(child_id);
                builder.build(cx)
            }
            WidgetKind::ActionScope(w) => w.lower(cx),
            WidgetKind::Row(w) => w.lower(cx),
            WidgetKind::Column(w) => w.lower(cx),
            WidgetKind::Align(w) => w.lower(cx),
            WidgetKind::FocusScope(w) => w.lower(cx),
            WidgetKind::Clip(w) => w.lower(cx),
            WidgetKind::Text(w) => w.lower(cx),
            WidgetKind::RichText(w) => w.lower(cx),
            WidgetKind::Transform(w) => w.lower(cx),
            WidgetKind::Button(w) => w.lower(cx),
            WidgetKind::TextInput(w) => w.lower(cx),
            WidgetKind::Scroll(w) => w.lower(cx),
            WidgetKind::SemanticsRegion(w) => w.lower(cx),
            WidgetKind::Image(w) => w.lower(cx),
            WidgetKind::Video(w) => w.lower(cx),
            WidgetKind::ZStack(w) => w.lower(cx),
            WidgetKind::Overlay(w) => w.lower(cx),
            WidgetKind::Container(w) => w.lower(cx),
            WidgetKind::GestureDetector(w) => w.lower(cx),
            WidgetKind::Grid(w) => w.lower(cx),
            WidgetKind::GridItem(w) => w.lower(cx),
            WidgetKind::Checkbox(w) => w.lower(cx),
            WidgetKind::Switch(w) => w.lower(cx),
            WidgetKind::Radio(w) => w.lower(cx),
            WidgetKind::SafeArea(w) => w.lower(cx),
            WidgetKind::Positioned(w) => w.lower(cx),
            WidgetKind::Spacer(w) => w.lower(cx),
            WidgetKind::Slider(w) => w.lower(cx),
            WidgetKind::LazyColumn(w) => w.lower(cx),
            WidgetKind::Icon(w) => w.lower(cx),
            WidgetKind::Composite(w) => w.lower(cx),
            WidgetKind::Custom(w) => {
                let lowerer = w
                    .lowerer
                    .as_ref()
                    .expect("CustomWidget lowerer must be set");
                let child_id = lowerer.lower_dyn(cx);
                let wrapper = cx.next_node_id();
                let mut builder = crate::lowering::InternalIrBuilder::new(
                    wrapper,
                    Op::Structural(StructuralOp::Group {
                        stable_hash: lowerer.stable_key(),
                    }),
                );
                builder.add_child(child_id);
                let node_id = builder.build(cx);

                // If the custom node carries a render object, store it in the
                // IR so that hit-testing and event handling can find it later.
                // We wrap the `Arc<dyn CustomRenderObject>` in a `RenderObjectHolder`
                // so it can be stored as `Arc<dyn Any + Send + Sync>` in the
                // dependency-free IR crate and downcast back later.
                if let Some(render_obj) = &w.render_object {
                    let holder = crate::ui::custom_render::RenderObjectHolder(render_obj.clone());
                    let erased: fission_ir::AnyRenderObject = Arc::new(holder);
                    // Register the render object at the wrapper AND every node in
                    // the lowered subtree so the parent-walk from any hit descendant
                    // finds it regardless of tree depth.
                    cx.ir.custom_render_objects.insert(node_id, erased.clone());
                    fn register_subtree(
                        ir: &mut fission_ir::CoreIR,
                        node_id: fission_ir::WidgetId,
                        erased: &fission_ir::AnyRenderObject,
                    ) {
                        ir.custom_render_objects.insert(node_id, erased.clone());
                        if let Some(children) = ir.nodes.get(&node_id).map(|n| n.children.clone()) {
                            for child_id in children {
                                register_subtree(ir, child_id, erased);
                            }
                        }
                    }
                    register_subtree(&mut cx.ir, child_id, &erased);
                }

                node_id
            }
        }
    }
}

impl From<Row> for Widget {
    fn from(w: Row) -> Self {
        Self {
            kind: Box::new(WidgetKind::Row(w)),
        }
    }
}
impl From<ActionScope> for Widget {
    fn from(w: ActionScope) -> Self {
        Self {
            kind: Box::new(WidgetKind::ActionScope(w)),
        }
    }
}
impl From<Column> for Widget {
    fn from(w: Column) -> Self {
        Self {
            kind: Box::new(WidgetKind::Column(w)),
        }
    }
}
impl From<Align> for Widget {
    fn from(w: Align) -> Self {
        Self {
            kind: Box::new(WidgetKind::Align(w)),
        }
    }
}
impl From<FocusScope> for Widget {
    fn from(w: FocusScope) -> Self {
        Self {
            kind: Box::new(WidgetKind::FocusScope(w)),
        }
    }
}
impl From<Clip> for Widget {
    fn from(w: Clip) -> Self {
        Self {
            kind: Box::new(WidgetKind::Clip(w)),
        }
    }
}
impl From<Text> for Widget {
    fn from(w: Text) -> Self {
        Self {
            kind: Box::new(WidgetKind::Text(w)),
        }
    }
}
impl From<RichText> for Widget {
    fn from(w: RichText) -> Self {
        Self {
            kind: Box::new(WidgetKind::RichText(w)),
        }
    }
}
impl From<Transform> for Widget {
    fn from(w: Transform) -> Self {
        Self {
            kind: Box::new(WidgetKind::Transform(w)),
        }
    }
}
impl From<Button> for Widget {
    fn from(w: Button) -> Self {
        Self {
            kind: Box::new(WidgetKind::Button(w)),
        }
    }
}
impl From<TextInput> for Widget {
    fn from(w: TextInput) -> Self {
        Self {
            kind: Box::new(WidgetKind::TextInput(w)),
        }
    }
}
impl From<Scroll> for Widget {
    fn from(w: Scroll) -> Self {
        Self {
            kind: Box::new(WidgetKind::Scroll(w)),
        }
    }
}
impl From<SemanticsRegion> for Widget {
    fn from(w: SemanticsRegion) -> Self {
        Self {
            kind: Box::new(WidgetKind::SemanticsRegion(w)),
        }
    }
}
impl From<Image> for Widget {
    fn from(w: Image) -> Self {
        Self {
            kind: Box::new(WidgetKind::Image(w)),
        }
    }
}
impl From<Video> for Widget {
    fn from(w: Video) -> Self {
        let node_id = crate::build::current_widget_id()
            .or(w.id)
            .unwrap_or_else(|| fission_ir::WidgetId::explicit(&w.source));
        crate::build::try_register_video(crate::registry::VideoRegistration {
            node_id,
            source: w.source.clone(),
            autoplay: w.autoplay,
            loop_playback: w.loop_playback,
        });
        Self {
            kind: Box::new(WidgetKind::Video(w)),
        }
    }
}
impl From<ZStack> for Widget {
    fn from(w: ZStack) -> Self {
        Self {
            kind: Box::new(WidgetKind::ZStack(w)),
        }
    }
}
impl From<Overlay> for Widget {
    fn from(w: Overlay) -> Self {
        Self {
            kind: Box::new(WidgetKind::Overlay(w)),
        }
    }
}
impl From<Container> for Widget {
    fn from(w: Container) -> Self {
        Self {
            kind: Box::new(WidgetKind::Container(w)),
        }
    }
}
impl From<GestureDetector> for Widget {
    fn from(w: GestureDetector) -> Self {
        Self {
            kind: Box::new(WidgetKind::GestureDetector(w)),
        }
    }
}
impl From<Grid> for Widget {
    fn from(w: Grid) -> Self {
        Self {
            kind: Box::new(WidgetKind::Grid(w)),
        }
    }
}
impl From<GridItem> for Widget {
    fn from(w: GridItem) -> Self {
        Self {
            kind: Box::new(WidgetKind::GridItem(w)),
        }
    }
}
impl From<Checkbox> for Widget {
    fn from(w: Checkbox) -> Self {
        Self {
            kind: Box::new(WidgetKind::Checkbox(w)),
        }
    }
}
impl From<Switch> for Widget {
    fn from(w: Switch) -> Self {
        Self {
            kind: Box::new(WidgetKind::Switch(w)),
        }
    }
}
impl From<Radio> for Widget {
    fn from(w: Radio) -> Self {
        Self {
            kind: Box::new(WidgetKind::Radio(w)),
        }
    }
}
impl From<SafeArea> for Widget {
    fn from(w: SafeArea) -> Self {
        Self {
            kind: Box::new(WidgetKind::SafeArea(w)),
        }
    }
}
impl From<Composite> for Widget {
    fn from(w: Composite) -> Self {
        Self {
            kind: Box::new(WidgetKind::Composite(w)),
        }
    }
}
impl From<Positioned> for Widget {
    fn from(w: Positioned) -> Self {
        Self {
            kind: Box::new(WidgetKind::Positioned(w)),
        }
    }
}
impl From<Spacer> for Widget {
    fn from(w: Spacer) -> Self {
        Self {
            kind: Box::new(WidgetKind::Spacer(w)),
        }
    }
}
impl From<Slider> for Widget {
    fn from(w: Slider) -> Self {
        Self {
            kind: Box::new(WidgetKind::Slider(w)),
        }
    }
}
impl From<LazyColumn> for Widget {
    fn from(w: LazyColumn) -> Self {
        Self {
            kind: Box::new(WidgetKind::LazyColumn(w)),
        }
    }
}
impl From<Icon> for Widget {
    fn from(w: Icon) -> Self {
        Self {
            kind: Box::new(WidgetKind::Icon(w)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InternalRenderNode {
    pub debug_tag: String,
    #[serde(skip)]
    pub lowerer: Option<Arc<dyn InternalLowerer>>,
    /// Optional render object that participates in hit-testing, event handling,
    /// and painting.  When `None`, the node behaves exactly as before (lowering
    /// only via `InternalLowerer`).
    #[serde(skip)]
    pub render_object: Option<Arc<dyn CustomRenderObject>>,
}

pub type CustomWidget = InternalRenderNode;

impl From<CustomWidget> for Widget {
    fn from(node: CustomWidget) -> Self {
        Widget::custom(node)
    }
}
