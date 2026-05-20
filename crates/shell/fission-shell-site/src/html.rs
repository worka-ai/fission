use anyhow::{anyhow, bail, Result};
use fission_ir::op::{
    decode_inline_widget_marker, AlignItems, BoxShadow, Color, Fill, FlexDirection, FlexWrap,
    FontStyle, GridPlacement, GridTrack, ImageFit, JustifyContent, LayoutOp, LineCap, LineJoin, Op,
    PaintOp, Stroke, TextAlign, TextOverflow, TextRun,
};
use fission_ir::{CoreIR, CoreNode, NodeId, Role};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct HtmlRenderOptions {
    pub document_title: String,
    pub description: Option<String>,
    pub stylesheet_href: String,
    pub root_class: String,
    pub current_route_path: String,
}

impl Default for HtmlRenderOptions {
    fn default() -> Self {
        Self {
            document_title: "Static site".to_string(),
            description: None,
            stylesheet_href: "/site.css".to_string(),
            root_class: "fission-site-root".to_string(),
            current_route_path: "/".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedHtml {
    pub html: String,
    pub body_html: String,
}

pub fn render_ir_to_html(ir: &CoreIR, options: &HtmlRenderOptions) -> Result<RenderedHtml> {
    validate_static_ir(ir)?;
    let root = ir
        .root
        .ok_or_else(|| anyhow!("site render failed: Core IR has no root node"))?;
    let mut renderer = HtmlRenderer { ir, options };
    let body = renderer.render_node(root)?;
    let body_html = format!(
        "<div class=\"{}\">{body}</div>",
        escape_attr(&options.root_class)
    );
    let html = render_document(&body_html, options);
    Ok(RenderedHtml { html, body_html })
}

fn render_document(body_html: &str, options: &HtmlRenderOptions) -> String {
    let description = options
        .description
        .as_ref()
        .map(|value| {
            format!(
                "\n    <meta name=\"description\" content=\"{}\">",
                escape_attr(value)
            )
        })
        .unwrap_or_default();
    format!(
        "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\">\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">{description}\n    <title>{}</title>\n    <link rel=\"stylesheet\" href=\"{}\">\n  </head>\n  <body>\n    {body_html}\n  </body>\n</html>\n",
        escape_text(&options.document_title),
        escape_attr(&options.stylesheet_href)
    )
}

fn validate_static_ir(ir: &CoreIR) -> Result<()> {
    for node in ir.nodes.values() {
        match &node.op {
            Op::Semantics(semantics) => {
                if !semantics.actions.entries.is_empty() {
                    bail!(
                        "static site renderer cannot lower interactive actions on node {}; use a web target or add explicit static enhancement support",
                        node.id
                    );
                }
                match semantics.role {
                    Role::TextInput
                    | Role::Checkbox
                    | Role::Switch
                    | Role::Slider
                    | Role::Input => {
                        bail!(
                            "static site renderer cannot lower interactive semantic role {:?} on node {}; provide static content or move this route to a web target",
                            semantics.role,
                            node.id
                        );
                    }
                    _ => {}
                }
            }
            Op::Layout(LayoutOp::Embed { kind, .. }) => {
                bail!(
                    "static site renderer cannot lower embedded surface {:?} on node {}; provide a static fallback widget for the site target",
                    kind,
                    node.id
                );
            }
            _ => {}
        }
    }
    Ok(())
}

struct HtmlRenderer<'a> {
    ir: &'a CoreIR,
    options: &'a HtmlRenderOptions,
}

impl HtmlRenderer<'_> {
    fn render_node(&mut self, node_id: NodeId) -> Result<String> {
        let node = self
            .ir
            .nodes
            .get(&node_id)
            .ok_or_else(|| anyhow!("site render failed: missing IR node {node_id}"))?;
        match &node.op {
            Op::Structural(_) => self.render_element("div", node, "fission-site-node", Vec::new()),
            Op::Layout(layout) => self.render_layout(node, layout),
            Op::Paint(paint) => self.render_paint(node, paint),
            Op::Semantics(_) => self.render_semantics(node),
        }
    }

    fn render_children(&mut self, children: &[NodeId], skip: &HashSet<NodeId>) -> Result<String> {
        let mut out = String::new();
        for child in children {
            if skip.contains(child) {
                continue;
            }
            out.push_str(&self.render_node(*child)?);
        }
        Ok(out)
    }

    fn render_element(
        &mut self,
        tag: &str,
        node: &CoreNode,
        class_name: &str,
        mut style: Vec<String>,
    ) -> Result<String> {
        let mut skip = HashSet::new();
        style.extend(self.coalesced_paint_style(node, &mut skip)?);
        style.extend(composite_style(node));
        let children = self.render_children(&node.children, &skip)?;
        Ok(format!(
            "<{tag} class=\"{}\" style=\"{}\" data-fission-node=\"{}\">{children}</{tag}>",
            escape_attr(class_name),
            escape_attr(&style.join(";")),
            node.id
        ))
    }

    fn render_layout(&mut self, node: &CoreNode, layout: &LayoutOp) -> Result<String> {
        match layout {
            LayoutOp::Box {
                width,
                height,
                min_width,
                max_width,
                min_height,
                max_height,
                padding,
                flex_grow,
                flex_shrink,
                aspect_ratio,
            } => {
                let mut style = vec!["display:block".to_string(), "position:relative".to_string()];
                push_box_constraints(
                    &mut style,
                    *width,
                    *height,
                    *min_width,
                    *max_width,
                    *min_height,
                    *max_height,
                );
                push_padding(&mut style, *padding);
                push_flex_item(&mut style, *flex_grow, *flex_shrink);
                if let Some(aspect_ratio) = aspect_ratio {
                    style.push(format!("aspect-ratio:{aspect_ratio}"));
                }
                self.render_element("div", node, "fission-site-node fission-site-box", style)
            }
            LayoutOp::Flex {
                direction,
                wrap,
                flex_grow,
                flex_shrink,
                padding,
                gap,
                align_items,
                justify_content,
            } => {
                let mut style = vec![
                    "display:flex".to_string(),
                    format!("flex-direction:{}", flex_direction(*direction)),
                    format!("flex-wrap:{}", flex_wrap(*wrap)),
                    format!("align-items:{}", align_items_css(*align_items)),
                    format!("justify-content:{}", justify_content_css(*justify_content)),
                ];
                if let Some(gap) = gap {
                    style.push(format!("gap:{}px", px(*gap)));
                }
                push_padding(&mut style, *padding);
                push_flex_item(&mut style, *flex_grow, *flex_shrink);
                let class_name = match direction {
                    FlexDirection::Column => "fission-site-node fission-site-column",
                    FlexDirection::Row => "fission-site-node fission-site-row",
                };
                self.render_element("div", node, class_name, style)
            }
            LayoutOp::Grid {
                columns,
                rows,
                column_gap,
                row_gap,
                padding,
            } => {
                let mut style = vec!["display:grid".to_string()];
                if !columns.is_empty() {
                    style.push(format!("grid-template-columns:{}", grid_tracks(columns)));
                }
                if !rows.is_empty() {
                    style.push(format!("grid-template-rows:{}", grid_tracks(rows)));
                }
                if let Some(gap) = column_gap {
                    style.push(format!("column-gap:{}px", px(*gap)));
                }
                if let Some(gap) = row_gap {
                    style.push(format!("row-gap:{}px", px(*gap)));
                }
                push_padding(&mut style, *padding);
                self.render_element("div", node, "fission-site-node fission-site-grid", style)
            }
            LayoutOp::GridItem {
                row_start,
                row_end,
                col_start,
                col_end,
            } => {
                let mut style = Vec::new();
                push_grid_placement(&mut style, "grid-row-start", *row_start);
                push_grid_placement(&mut style, "grid-row-end", *row_end);
                push_grid_placement(&mut style, "grid-column-start", *col_start);
                push_grid_placement(&mut style, "grid-column-end", *col_end);
                self.render_element("div", node, "fission-site-node fission-site-grid-item", style)
            }
            LayoutOp::Scroll {
                direction,
                show_scrollbar: _,
                width,
                height,
                min_width,
                max_width,
                min_height,
                max_height,
                padding,
                flex_grow,
                flex_shrink,
            } => {
                let mut style = vec![
                    "display:flex".to_string(),
                    format!("flex-direction:{}", flex_direction(*direction)),
                    "overflow:auto".to_string(),
                ];
                push_box_constraints(
                    &mut style,
                    *width,
                    *height,
                    *min_width,
                    *max_width,
                    *min_height,
                    *max_height,
                );
                push_padding(&mut style, *padding);
                push_flex_item(&mut style, *flex_grow, *flex_shrink);
                self.render_element("div", node, "fission-site-node fission-site-scroll", style)
            }
            LayoutOp::Embed { .. } => unreachable!("embed ops are rejected before rendering"),
            LayoutOp::AbsoluteFill => self.render_element(
                "div",
                node,
                "fission-site-node fission-site-absolute-fill",
                vec!["position:absolute".to_string(), "inset:0".to_string()],
            ),
            LayoutOp::Positioned {
                left,
                top,
                right,
                bottom,
                width,
                height,
            } => {
                let mut style = vec!["position:absolute".to_string()];
                push_optional_px(&mut style, "left", *left);
                push_optional_px(&mut style, "top", *top);
                push_optional_px(&mut style, "right", *right);
                push_optional_px(&mut style, "bottom", *bottom);
                push_optional_px(&mut style, "width", *width);
                push_optional_px(&mut style, "height", *height);
                self.render_element("div", node, "fission-site-node fission-site-positioned", style)
            }
            LayoutOp::ZStack => self.render_element(
                "div",
                node,
                "fission-site-node fission-site-zstack",
                vec!["display:grid".to_string(), "position:relative".to_string()],
            ),
            LayoutOp::Align => self.render_element(
                "div",
                node,
                "fission-site-node fission-site-align",
                vec![
                    "display:flex".to_string(),
                    "align-items:center".to_string(),
                    "justify-content:center".to_string(),
                ],
            ),
            LayoutOp::Flyout { .. } => bail!(
                "static site renderer cannot lower flyout layout on node {}; render open dialog content as normal page content for site output",
                node.id
            ),
            LayoutOp::Transform { transform } => self.render_element(
                "div",
                node,
                "fission-site-node fission-site-transform",
                vec![format!("transform:matrix3d({})", matrix3d(transform))],
            ),
            LayoutOp::Clip { path } => {
                let mut style = vec!["overflow:hidden".to_string()];
                if let Some(path) = path {
                    style.push(format!("clip-path:path('{}')", css_string(path)));
                }
                self.render_element("div", node, "fission-site-node fission-site-clip", style)
            }
        }
    }

    fn render_paint(&mut self, node: &CoreNode, paint: &PaintOp) -> Result<String> {
        match paint {
            PaintOp::DrawRect {
                fill,
                stroke,
                corner_radius,
                shadow,
            } => {
                let mut style = draw_rect_style(fill.as_ref(), stroke.as_ref(), *corner_radius, shadow.as_ref());
                style.push("min-height:1px".to_string());
                self.render_element("div", node, "fission-site-node fission-site-rect", style)
            }
            PaintOp::DrawText {
                text,
                size,
                color,
                underline,
                wrap,
                paragraph_style,
                ..
            } => {
                let mut style = text_style(*size, *color, *underline, *wrap);
                push_paragraph_style(&mut style, paragraph_style.as_ref());
                Ok(format!(
                    "<span class=\"fission-site-text\" style=\"{}\" data-fission-node=\"{}\">{}</span>",
                    escape_attr(&style.join(";")),
                    node.id,
                    escape_text(text)
                ))
            }
            PaintOp::DrawRichText {
                runs,
                wrap,
                paragraph_style,
                ..
            } => {
                let mut style = vec![
                    "display:inline".to_string(),
                    format!("white-space:{}", if *wrap { "pre-wrap" } else { "pre" }),
                ];
                push_paragraph_style(&mut style, paragraph_style.as_ref());
                let mut content = String::new();
                for run in runs {
                    if decode_inline_widget_marker(run.style.font_family.as_deref()).is_some() {
                        continue;
                    }
                    content.push_str(&render_text_run(run));
                }
                content.push_str(&self.render_children(&node.children, &HashSet::new())?);
                Ok(format!(
                    "<span class=\"fission-site-rich-text\" style=\"{}\" data-fission-node=\"{}\">{content}</span>",
                    escape_attr(&style.join(";")),
                    node.id
                ))
            }
            PaintOp::DrawImage { source, fit } => Ok(format!(
                "<img class=\"fission-site-img\" src=\"{}\" alt=\"\" style=\"width:100%;height:100%;object-fit:{}\" data-fission-node=\"{}\">",
                escape_attr(source),
                image_fit_css(*fit),
                node.id
            )),
            PaintOp::DrawPath { path, fill, stroke } => Ok(format!(
                "<svg class=\"fission-site-svg\" viewBox=\"0 0 24 24\" aria-hidden=\"true\" data-fission-node=\"{}\"><path d=\"{}\" style=\"{}\"></path></svg>",
                node.id,
                escape_attr(path),
                escape_attr(&svg_paint_style(fill.as_ref(), stroke.as_ref()))
            )),
            PaintOp::DrawSvg {
                content,
                fill: _,
                stroke: _,
            } => Ok(format!(
                "<span class=\"fission-site-svg\" data-fission-node=\"{}\">{}</span>",
                node.id,
                content
            )),
        }
    }

    fn render_semantics(&mut self, node: &CoreNode) -> Result<String> {
        let Op::Semantics(semantics) = &node.op else {
            unreachable!();
        };
        if let Some(identifier) = semantics.identifier.as_deref() {
            if let Some(target) = identifier.strip_prefix("site-route:") {
                return self.render_semantic_link(
                    node,
                    target,
                    semantics.label.as_deref(),
                    "fission-site-route-link",
                );
            }
            if let Some(anchor) = identifier.strip_prefix("site-heading:") {
                return self.render_semantic_link(
                    node,
                    &format!("#{anchor}"),
                    semantics.label.as_deref(),
                    "fission-site-heading-link",
                );
            }
            if let Some(target) = identifier.strip_prefix("markdown-link:") {
                return self.render_semantic_link(
                    node,
                    target,
                    semantics.label.as_deref(),
                    "fission-site-markdown-link",
                );
            }
        }
        let tag = match semantics.role {
            Role::Button => "button",
            Role::Image => "figure",
            Role::List => "ul",
            Role::ListItem => "li",
            Role::Dialog => "section",
            Role::Text | Role::Generic => "div",
            Role::TextInput | Role::Checkbox | Role::Switch | Role::Slider | Role::Input => {
                unreachable!("interactive roles are rejected before rendering")
            }
        };
        let tag = semantics
            .identifier
            .as_deref()
            .and_then(markdown_heading_tag)
            .unwrap_or(tag);
        let mut attrs = String::new();
        if let Some(label) = &semantics.label {
            attrs.push_str(&format!(" aria-label=\"{}\"", escape_attr(label)));
        }
        if let Some(identifier) = &semantics.identifier {
            attrs.push_str(&format!(
                " data-fission-semantics=\"{}\"",
                escape_attr(identifier)
            ));
            if let Some(anchor) = markdown_heading_anchor(identifier) {
                attrs.push_str(&format!(" id=\"{}\"", escape_attr(anchor)));
            }
        }
        if tag == "button" {
            attrs.push_str(" type=\"button\" disabled");
        }
        let children = self.render_children(&node.children, &HashSet::new())?;
        Ok(format!(
            "<{tag} class=\"fission-site-node fission-site-semantics\"{attrs} data-fission-node=\"{}\">{children}</{tag}>",
            node.id
        ))
    }

    fn render_semantic_link(
        &mut self,
        node: &CoreNode,
        target: &str,
        label: Option<&str>,
        link_class: &str,
    ) -> Result<String> {
        let mut attrs = format!(" href=\"{}\"", escape_attr(&self.resolve_link_href(target)));
        if let Some(label) = label {
            attrs.push_str(&format!(" aria-label=\"{}\"", escape_attr(label)));
        }
        attrs.push_str(&format!(
            " data-fission-current-route=\"{}\"",
            escape_attr(&self.options.current_route_path)
        ));
        let children = self.render_children(&node.children, &HashSet::new())?;
        Ok(format!(
            "<a class=\"fission-site-node fission-site-link {link_class}\"{attrs} data-fission-node=\"{}\">{children}</a>",
            node.id
        ))
    }

    fn resolve_link_href(&self, target: &str) -> String {
        if target.starts_with('#')
            || target.starts_with('/')
            || target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("mailto:")
            || target.starts_with("tel:")
        {
            target.to_string()
        } else {
            target.to_string()
        }
    }

    fn coalesced_paint_style(
        &self,
        node: &CoreNode,
        skip: &mut HashSet<NodeId>,
    ) -> Result<Vec<String>> {
        let mut style = Vec::new();
        for child_id in &node.children {
            let Some(child) = self.ir.nodes.get(child_id) else {
                continue;
            };
            if let Op::Paint(PaintOp::DrawRect {
                fill,
                stroke,
                corner_radius,
                shadow,
            }) = &child.op
            {
                style.extend(draw_rect_style(
                    fill.as_ref(),
                    stroke.as_ref(),
                    *corner_radius,
                    shadow.as_ref(),
                ));
                skip.insert(*child_id);
            }
        }
        Ok(style)
    }
}

fn draw_rect_style(
    fill: Option<&Fill>,
    stroke: Option<&Stroke>,
    corner_radius: f32,
    shadow: Option<&BoxShadow>,
) -> Vec<String> {
    let mut style = Vec::new();
    if let Some(fill) = fill {
        style.push(format!("background:{}", fill_css(fill)));
    }
    if let Some(stroke) = stroke {
        style.push(format!(
            "border:{}px solid {}",
            px(stroke.width),
            stroke_css(stroke)
        ));
    }
    if corner_radius > 0.0 {
        style.push(format!("border-radius:{}px", px(corner_radius)));
    }
    if let Some(shadow) = shadow {
        style.push(format!(
            "box-shadow:{}px {}px {}px {}",
            px(shadow.offset.0),
            px(shadow.offset.1),
            px(shadow.blur_radius),
            color_css(shadow.color)
        ));
    }
    style
}

fn markdown_heading_tag(identifier: &str) -> Option<&'static str> {
    let level = identifier
        .strip_prefix("markdown-heading-")?
        .split_once(':')
        .map(|(level, _)| level)
        .unwrap_or_else(|| identifier.strip_prefix("markdown-heading-").unwrap_or(""));
    match level {
        "1" => Some("h1"),
        "2" => Some("h2"),
        "3" => Some("h3"),
        "4" => Some("h4"),
        "5" => Some("h5"),
        "6" => Some("h6"),
        _ => None,
    }
}

fn markdown_heading_anchor(identifier: &str) -> Option<&str> {
    identifier
        .strip_prefix("markdown-heading-")?
        .split_once(':')
        .map(|(_, anchor)| anchor)
        .filter(|anchor| !anchor.is_empty())
}

fn text_style(size: f32, color: Color, underline: bool, wrap: bool) -> Vec<String> {
    let mut style = vec![
        format!("font-size:{}px", px(size)),
        format!("color:{}", color_css(color)),
        format!("white-space:{}", if wrap { "pre-wrap" } else { "pre" }),
    ];
    if underline {
        style.push("text-decoration:underline".to_string());
    }
    style
}

fn render_text_run(run: &TextRun) -> String {
    let mut style = vec![
        format!("font-size:{}px", px(run.style.font_size)),
        format!("color:{}", color_css(run.style.color)),
        format!("font-weight:{}", run.style.font_weight),
        format!("letter-spacing:{}px", px(run.style.letter_spacing)),
    ];
    if run.style.underline {
        style.push("text-decoration:underline".to_string());
    }
    if let Some(family) = &run.style.font_family {
        style.push(format!("font-family:{}", css_string(family)));
    }
    if let Some(line_height) = run.style.line_height {
        style.push(format!("line-height:{}px", px(line_height)));
    }
    if run.style.font_style == FontStyle::Italic {
        style.push("font-style:italic".to_string());
    }
    if let Some(background) = run.style.background_color {
        style.push(format!("background:{}", color_css(background)));
        style.push("border-radius:0.35em".to_string());
        style.push("padding:0.1em 0.3em".to_string());
    }
    format!(
        "<span style=\"{}\">{}</span>",
        escape_attr(&style.join(";")),
        escape_text(&run.text)
    )
}

fn push_paragraph_style(
    style: &mut Vec<String>,
    paragraph: Option<&fission_ir::op::TextParagraphStyle>,
) {
    if let Some(paragraph) = paragraph {
        style.push(format!(
            "text-align:{}",
            text_align_css(paragraph.text_align)
        ));
        if let Some(lines) = paragraph.max_lines {
            style.push("display:-webkit-box".to_string());
            style.push("-webkit-box-orient:vertical".to_string());
            style.push(format!("-webkit-line-clamp:{lines}"));
        }
        match paragraph.overflow {
            TextOverflow::Clip => style.push("overflow:hidden".to_string()),
            TextOverflow::Ellipsis => {
                style.push("overflow:hidden".to_string());
                style.push("text-overflow:ellipsis".to_string());
            }
            TextOverflow::Fade => style.push("overflow:hidden".to_string()),
            TextOverflow::Visible => {}
        }
    }
}

fn push_box_constraints(
    style: &mut Vec<String>,
    width: Option<f32>,
    height: Option<f32>,
    min_width: Option<f32>,
    max_width: Option<f32>,
    min_height: Option<f32>,
    max_height: Option<f32>,
) {
    push_optional_px(style, "width", width);
    push_optional_px(style, "height", height);
    push_optional_px(style, "min-width", min_width);
    push_optional_px(style, "max-width", max_width);
    push_optional_px(style, "min-height", min_height);
    push_optional_px(style, "max-height", max_height);
}

fn push_padding(style: &mut Vec<String>, padding: [f32; 4]) {
    if padding.iter().any(|value| *value != 0.0) {
        style.push(format!(
            "padding:{}px {}px {}px {}px",
            px(padding[2]),
            px(padding[1]),
            px(padding[3]),
            px(padding[0])
        ));
    }
}

fn push_flex_item(style: &mut Vec<String>, flex_grow: f32, flex_shrink: f32) {
    if flex_grow != 0.0 {
        style.push(format!("flex-grow:{flex_grow}"));
    }
    if (flex_shrink - 1.0).abs() > f32::EPSILON {
        style.push(format!("flex-shrink:{flex_shrink}"));
    }
}

fn push_optional_px(style: &mut Vec<String>, name: &str, value: Option<f32>) {
    if let Some(value) = value {
        style.push(format!("{name}:{}px", px(value)));
    }
}

fn push_grid_placement(style: &mut Vec<String>, name: &str, value: GridPlacement) {
    match value {
        GridPlacement::Auto => {}
        GridPlacement::Line(line) => style.push(format!("{name}:{line}")),
        GridPlacement::Span(span) => style.push(format!("{name}:span {span}")),
    }
}

fn composite_style(node: &CoreNode) -> Vec<String> {
    let mut style = Vec::new();
    if let Some(opacity) = node.composite.opacity.as_ref() {
        style.push(format!("opacity:{}", opacity.base));
    }
    if node.composite.clip_to_bounds {
        style.push("overflow:hidden".to_string());
    }
    let translate_x = node
        .composite
        .translate_x
        .as_ref()
        .map(|v| v.base)
        .unwrap_or(0.0);
    let translate_y = node
        .composite
        .translate_y
        .as_ref()
        .map(|v| v.base)
        .unwrap_or(0.0);
    let scale = node.composite.scale.as_ref().map(|v| v.base).unwrap_or(1.0);
    let rotation = node
        .composite
        .rotation
        .as_ref()
        .map(|v| v.base)
        .unwrap_or(0.0);
    if translate_x != 0.0
        || translate_y != 0.0
        || (scale - 1.0).abs() > f32::EPSILON
        || rotation != 0.0
    {
        style.push(format!(
            "transform:translate({}px,{}px) scale({}) rotate({}rad)",
            px(translate_x),
            px(translate_y),
            scale,
            rotation
        ));
    }
    style
}

fn fill_css(fill: &Fill) -> String {
    match fill {
        Fill::Solid(color) => color_css(*color),
        Fill::LinearGradient {
            start: _,
            end,
            stops,
        } => {
            let angle = if end.0.abs() >= end.1.abs() {
                "90deg"
            } else {
                "180deg"
            };
            let stops = stops
                .iter()
                .map(|(offset, color)| {
                    format!("{} {}%", color_css(*color), (offset * 100.0).round())
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("linear-gradient({angle},{stops})")
        }
        Fill::RadialGradient { stops, .. } => {
            let stops = stops
                .iter()
                .map(|(offset, color)| {
                    format!("{} {}%", color_css(*color), (offset * 100.0).round())
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("radial-gradient(circle,{stops})")
        }
    }
}

fn stroke_css(stroke: &Stroke) -> String {
    match &stroke.fill {
        Fill::Solid(color) => color_css(*color),
        fill => fill_css(fill),
    }
}

fn svg_paint_style(fill: Option<&Fill>, stroke: Option<&Stroke>) -> String {
    let mut style = Vec::new();
    if let Some(fill) = fill {
        style.push(format!("fill:{}", fill_css(fill)));
    } else {
        style.push("fill:currentColor".to_string());
    }
    if let Some(stroke) = stroke {
        style.push(format!("stroke:{}", stroke_css(stroke)));
        style.push(format!("stroke-width:{}", px(stroke.width)));
        style.push(format!("stroke-linecap:{}", line_cap_css(stroke.line_cap)));
        style.push(format!(
            "stroke-linejoin:{}",
            line_join_css(stroke.line_join)
        ));
    }
    style.join(";")
}

fn color_css(color: Color) -> String {
    if color.a == 255 {
        format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
    } else {
        format!(
            "rgba({},{},{},{:.3})",
            color.r,
            color.g,
            color.b,
            color.a as f32 / 255.0
        )
    }
}

fn grid_tracks(tracks: &[GridTrack]) -> String {
    tracks
        .iter()
        .map(|track| match track {
            GridTrack::Points(value) => format!("{}px", px(*value)),
            GridTrack::Percent(value) => format!("{}%", px(*value)),
            GridTrack::Fr(value) => format!("{}fr", px(*value)),
            GridTrack::Auto => "auto".to_string(),
            GridTrack::MinContent => "min-content".to_string(),
            GridTrack::MaxContent => "max-content".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn flex_direction(direction: FlexDirection) -> &'static str {
    match direction {
        FlexDirection::Row => "row",
        FlexDirection::Column => "column",
    }
}

fn flex_wrap(wrap: FlexWrap) -> &'static str {
    match wrap {
        FlexWrap::NoWrap => "nowrap",
        FlexWrap::Wrap => "wrap",
        FlexWrap::WrapReverse => "wrap-reverse",
    }
}

fn align_items_css(align: AlignItems) -> &'static str {
    match align {
        AlignItems::Start => "flex-start",
        AlignItems::End => "flex-end",
        AlignItems::Center => "center",
        AlignItems::Stretch => "stretch",
        AlignItems::Baseline => "baseline",
    }
}

fn justify_content_css(justify: JustifyContent) -> &'static str {
    match justify {
        JustifyContent::Start => "flex-start",
        JustifyContent::End => "flex-end",
        JustifyContent::Center => "center",
        JustifyContent::SpaceBetween => "space-between",
        JustifyContent::SpaceAround => "space-around",
        JustifyContent::SpaceEvenly => "space-evenly",
    }
}

fn image_fit_css(fit: ImageFit) -> &'static str {
    match fit {
        ImageFit::Contain => "contain",
        ImageFit::Cover => "cover",
        ImageFit::Fill => "fill",
        ImageFit::None => "none",
    }
}

fn text_align_css(align: TextAlign) -> &'static str {
    match align {
        TextAlign::Left => "left",
        TextAlign::Right => "right",
        TextAlign::Center => "center",
        TextAlign::Justify => "justify",
        TextAlign::Start => "start",
        TextAlign::End => "end",
    }
}

fn line_cap_css(line_cap: LineCap) -> &'static str {
    match line_cap {
        LineCap::Butt => "butt",
        LineCap::Round => "round",
        LineCap::Square => "square",
    }
}

fn line_join_css(line_join: LineJoin) -> &'static str {
    match line_join {
        LineJoin::Miter => "miter",
        LineJoin::Round => "round",
        LineJoin::Bevel => "bevel",
    }
}

fn matrix3d(values: &[f32; 16]) -> String {
    values
        .iter()
        .map(|value| px(*value))
        .collect::<Vec<_>>()
        .join(",")
}

fn css_string(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn px(value: f32) -> String {
    if (value.fract()).abs() < 0.001 {
        format!("{}", value.round() as i32)
    } else {
        format!("{value:.3}")
    }
}

fn escape_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_attr(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use fission_ir::{ActionEntry, ActionSet, CoreIR, CoreNode, NodeId, Op, Semantics};

    #[test]
    fn renders_text_from_core_ir() {
        let root = NodeId::explicit("root");
        let text = NodeId::explicit("text");
        let mut ir = CoreIR::new();
        ir.add_node(
            text,
            Op::Paint(PaintOp::DrawText {
                text: "Hello <site>".into(),
                size: 16.0,
                color: Color::BLACK,
                underline: false,
                wrap: true,
                caret_index: None,
                caret_color: None,
                caret_width: None,
                caret_height: None,
                caret_radius: None,
                paragraph_style: None,
            }),
            Vec::new(),
        );
        ir.add_node(
            root,
            Op::Structural(fission_ir::StructuralOp::Group { stable_hash: 1 }),
            vec![text],
        );
        ir.set_root(root);

        let rendered = render_ir_to_html(&ir, &HtmlRenderOptions::default()).unwrap();
        assert!(rendered.html.contains("Hello &lt;site&gt;"));
    }

    #[test]
    fn rejects_interactive_actions() {
        let root = NodeId::explicit("root");
        let mut semantics = Semantics::default();
        semantics.actions = ActionSet {
            entries: vec![ActionEntry {
                trigger: fission_ir::semantics::ActionTrigger::Default,
                action_id: 1,
                payload_data: None,
            }],
        };
        let mut ir = CoreIR::new();
        ir.nodes.insert(
            root,
            CoreNode {
                id: root,
                op: Op::Semantics(semantics),
                composite: Default::default(),
                children: Vec::new(),
                parent: None,
                hash: 0,
            },
        );
        ir.set_root(root);
        let error = render_ir_to_html(&ir, &HtmlRenderOptions::default()).unwrap_err();
        assert!(error.to_string().contains("interactive actions"));
    }
}
