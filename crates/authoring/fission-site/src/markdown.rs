use rushdown::ast::{
    Arena, CodeBlock, Heading, HtmlBlock, Image, KindData, Link, NodeRef, RawHtml, TableCell, Text,
    TextQualifier,
};
use rushdown::parser::{self, Parser};
use rushdown::text::BasicReader;
use serde::Serialize;
use std::collections::BTreeSet;

use crate::utils::{escape_attr, escape_html, normalize_markdown_href, slugify};

#[derive(Debug)]
pub struct ParsedMarkdown {
    pub source: String,
    pub arena: Arena,
    pub document: NodeRef,
}

#[derive(Clone, Debug, Default)]
pub struct MarkdownHtml {
    pub html: String,
    pub h1_links: Vec<HeadingLink>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HeadingLink {
    pub id: String,
    pub title: String,
    pub level: u8,
}

pub(crate) fn parse_markdown(source: &str) -> ParsedMarkdown {
    let parser = Parser::with_extensions(
        parser::Options::default(),
        parser::gfm(parser::GfmOptions::default()),
    );
    let mut reader = BasicReader::new(source);
    let (arena, document) = parser.parse(&mut reader);
    ParsedMarkdown {
        source: source.to_string(),
        arena,
        document,
    }
}

pub fn markdown_to_html(source: &str) -> MarkdownHtml {
    markdown_to_html_with_base(source, None)
}

pub fn markdown_to_html_with_route(source: &str, route_path: &str) -> MarkdownHtml {
    markdown_to_html_with_base(source, Some(route_path))
}

fn markdown_to_html_with_base(source: &str, route_path: Option<&str>) -> MarkdownHtml {
    let parsed = parse_markdown(source);
    let mut renderer = MarkdownHtmlRenderer::new(&parsed.source, &parsed.arena, route_path);
    renderer.render_document(parsed.document);
    MarkdownHtml {
        html: renderer.html,
        h1_links: renderer.headings,
    }
}

struct MarkdownHtmlRenderer<'a> {
    source: &'a str,
    arena: &'a Arena,
    html: String,
    headings: Vec<HeadingLink>,
    used_ids: BTreeSet<String>,
    route_path: Option<&'a str>,
}

impl<'a> MarkdownHtmlRenderer<'a> {
    fn new(source: &'a str, arena: &'a Arena, route_path: Option<&'a str>) -> Self {
        Self {
            source,
            arena,
            html: String::new(),
            headings: Vec::new(),
            used_ids: BTreeSet::new(),
            route_path,
        }
    }

    fn render_document(&mut self, node_ref: NodeRef) {
        for child in self.arena[node_ref].children(self.arena) {
            self.block(child);
        }
    }

    fn block(&mut self, node_ref: NodeRef) {
        match self.arena[node_ref].kind_data() {
            KindData::Document(_) => self.render_document(node_ref),
            KindData::Paragraph(_) => {
                self.html.push_str("<p>");
                self.inline_children(node_ref);
                self.html.push_str("</p>\n");
            }
            KindData::Heading(heading) => self.heading(node_ref, heading),
            KindData::ThematicBreak(_) => self.html.push_str("<hr />\n"),
            KindData::CodeBlock(code) => self.code_block(code),
            KindData::Blockquote(_) => {
                self.html.push_str("<blockquote>");
                self.render_document(node_ref);
                self.html.push_str("</blockquote>\n");
            }
            KindData::List(list) => {
                let tag = if list.is_ordered() { "ol" } else { "ul" };
                if list.is_ordered() && list.start() > 1 {
                    self.html
                        .push_str(&format!("<{tag} start=\"{}\">", list.start()));
                } else {
                    self.html.push_str(&format!("<{tag}>"));
                }
                for child in self.arena[node_ref].children(self.arena) {
                    self.block(child);
                }
                self.html.push_str(&format!("</{tag}>\n"));
            }
            KindData::ListItem(_) => {
                self.html.push_str("<li>");
                self.render_document(node_ref);
                self.html.push_str("</li>");
            }
            KindData::HtmlBlock(html) => self.html_block(html),
            KindData::Table(_) => {
                self.html.push_str("<table>");
                for child in self.arena[node_ref].children(self.arena) {
                    self.block(child);
                }
                self.html.push_str("</table>\n");
            }
            KindData::TableHeader(_) => {
                self.html.push_str("<thead>");
                for child in self.arena[node_ref].children(self.arena) {
                    self.block(child);
                }
                self.html.push_str("</thead>");
            }
            KindData::TableBody(_) => {
                self.html.push_str("<tbody>");
                for child in self.arena[node_ref].children(self.arena) {
                    self.block(child);
                }
                self.html.push_str("</tbody>");
            }
            KindData::TableRow(_) => {
                self.html.push_str("<tr>");
                for child in self.arena[node_ref].children(self.arena) {
                    self.block(child);
                }
                self.html.push_str("</tr>");
            }
            KindData::TableCell(cell) => self.table_cell(node_ref, cell),
            KindData::LinkReferenceDefinition(_) => {}
            _ => {
                self.html.push_str("<p>");
                self.inline(node_ref);
                self.html.push_str("</p>\n");
            }
        }
    }

    fn heading(&mut self, node_ref: NodeRef, heading: &Heading) {
        let level = heading.level().clamp(1, 6);
        let text = plain_text(self.source, self.arena, node_ref);
        let id = self.unique_id(&slugify(&text));
        if level == 1 {
            self.headings.push(HeadingLink {
                id: id.clone(),
                title: text.clone(),
                level,
            });
        }
        self.html
            .push_str(&format!("<h{level} id=\"{}\">", escape_attr(&id)));
        self.inline_children(node_ref);
        self.html.push_str(&format!("</h{level}>\n"));
    }

    fn unique_id(&mut self, base: &str) -> String {
        let base = if base.is_empty() { "section" } else { base };
        let mut candidate = base.to_string();
        let mut suffix = 2;
        while self.used_ids.contains(&candidate) {
            candidate = format!("{base}-{suffix}");
            suffix += 1;
        }
        self.used_ids.insert(candidate.clone());
        candidate
    }

    fn code_block(&mut self, code: &CodeBlock) {
        let text = code
            .value()
            .iter(self.source)
            .fold(String::new(), |mut out, line| {
                out.push_str(line.as_ref());
                out
            });
        let class = code
            .language_str(self.source)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|language| format!(" class=\"language-{}\"", escape_attr(language)))
            .unwrap_or_default();
        self.html.push_str(&format!(
            "<pre><code{class}>{}</code></pre>\n",
            escape_html(&text)
        ));
    }

    fn html_block(&mut self, html: &HtmlBlock) {
        let text = html
            .value()
            .iter(self.source)
            .fold(String::new(), |mut out, line| {
                out.push_str(line.as_ref());
                out
            });
        self.html.push_str(&text);
        self.html.push('\n');
    }

    fn table_cell(&mut self, node_ref: NodeRef, cell: &TableCell) {
        let align = match cell.alignment().as_str() {
            "left" | "center" | "right" => {
                format!(" style=\"text-align:{}\"", cell.alignment().as_str())
            }
            _ => String::new(),
        };
        self.html.push_str(&format!("<td{align}>"));
        self.inline_children(node_ref);
        self.html.push_str("</td>");
    }

    fn inline_children(&mut self, node_ref: NodeRef) {
        for child in self.arena[node_ref].children(self.arena) {
            self.inline(child);
        }
    }

    fn inline(&mut self, node_ref: NodeRef) {
        match self.arena[node_ref].kind_data() {
            KindData::Text(text) => self.text(text),
            KindData::CodeSpan(code) => self.html.push_str(&format!(
                "<code>{}</code>",
                escape_html(&code.str(self.source))
            )),
            KindData::Emphasis(_) => {
                self.html.push_str("<em>");
                self.inline_children(node_ref);
                self.html.push_str("</em>");
            }
            KindData::Strong(_) => {
                self.html.push_str("<strong>");
                self.inline_children(node_ref);
                self.html.push_str("</strong>");
            }
            KindData::Strikethrough(_) => {
                self.html.push_str("<del>");
                self.inline_children(node_ref);
                self.html.push_str("</del>");
            }
            KindData::Link(link) => self.link(node_ref, link),
            KindData::Image(image) => self.image(node_ref, image),
            KindData::RawHtml(raw) => self.raw_html(raw),
            _ => self.inline_children(node_ref),
        }
    }

    fn text(&mut self, text: &Text) {
        self.html.push_str(&escape_html(text.str(self.source)));
        if text.has_qualifiers(TextQualifier::HARD_LINE_BREAK) {
            self.html.push_str("<br />");
        } else if text.has_qualifiers(TextQualifier::SOFT_LINE_BREAK) {
            self.html.push(' ');
        }
    }

    fn link(&mut self, node_ref: NodeRef, link: &Link) {
        let href = normalize_markdown_href(link.destination_str(self.source), self.route_path);
        self.html
            .push_str(&format!("<a href=\"{}\">", escape_attr(&href)));
        self.inline_children(node_ref);
        self.html.push_str("</a>");
    }

    fn image(&mut self, node_ref: NodeRef, image: &Image) {
        let alt = plain_text(self.source, self.arena, node_ref);
        self.html.push_str(&format!(
            "<img src=\"{}\" alt=\"{}\" />",
            escape_attr(image.destination_str(self.source)),
            escape_attr(&alt)
        ));
    }

    fn raw_html(&mut self, raw: &RawHtml) {
        self.html.push_str(&raw.str(self.source));
    }
}

fn plain_text(source: &str, arena: &Arena, node_ref: NodeRef) -> String {
    let mut out = String::new();
    match arena[node_ref].kind_data() {
        KindData::Text(text) => out.push_str(text.str(source)),
        KindData::CodeSpan(code) => out.push_str(&code.str(source)),
        _ => {
            for child in arena[node_ref].children(arena) {
                out.push_str(&plain_text(source, arena, child));
            }
        }
    }
    out
}
