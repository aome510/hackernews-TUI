use super::html::HTMLTextParsedResult;
use super::rcdom::{Handle, NodeData, RcDom};
use crate::parser::html::HTMLTableParsedResult;
use crate::prelude::*;
use crate::utils::decode_html;
use html5ever::tendril::TendrilSink;
use html5ever::*;
use once_cell::sync::Lazy;
use regex::Regex;

/// a regex that matches whitespace character(s)
static WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

#[derive(Debug, Clone)]
/// Additional arguments of the article parse function [`Article::parse()`]
struct ArticleParseArgs {
    /// A value indicates whether the current node is inside a `<pre>` tag.
    pub in_pre_node: bool,
    /// A value indicates whether a node is the first element of a block tag.
    /// This is mostly used to add newlines separating two consecutive elements in a block node.
    pub is_first_element_in_block: bool,
    /// A prefix string appended to each line of the current node's inner text.
    /// This is mostly used to decorate or indent elements inside specific nodes.
    pub prefix: String,
}

impl Default for ArticleParseArgs {
    fn default() -> Self {
        Self {
            in_pre_node: false,
            is_first_element_in_block: true,
            prefix: String::new(),
        }
    }
}

impl Article {
    /// Parses the article's HTML content
    ///
    /// # Arguments:
    /// * `max_width`: the maximum width of the parsed content. This is mostly used
    /// to construct a HTML table using `comfy_table`.
    pub fn parse(&self, max_width: usize) -> Result<HTMLTextParsedResult> {
        debug!("parse article ({:?})", self);

        // parse HTML content into DOM node(s)
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut (self.content.as_bytes()))?;

        let (mut result, _) = Self::parse_dom_node(
            dom.document,
            max_width,
            0,
            Style::default(),
            ArticleParseArgs::default(),
        );

        // process the links
        result.links = result
            .links
            .into_iter()
            .map(|l| {
                match url::Url::parse(&l) {
                    // Failed to parse the link, possibly because it's a relative link, (e.g `/a/b`).
                    // Try to convert the relative link into an absolute link.
                    Err(err) => {
                        debug!("failed to parse url {l}: {err}");
                        match url::Url::parse(&self.url).unwrap().join(&l) {
                            Ok(url) => url.to_string(),
                            Err(_) => l,
                        }
                    }
                    Ok(_) => l,
                }
            })
            .collect();

        Ok(result)
    }

    /// Parses a HTML DOM node.
    ///
    /// # Returns
    /// The function returns a HTML parsed result and a boolean value
    /// indicating whether the current node has a non-whitespace text.
    fn parse_dom_node(
        node: Handle,
        max_width: usize,
        base_link_id: usize,
        mut style: Style,
        mut args: ArticleParseArgs,
    ) -> (HTMLTextParsedResult, bool) {
        // TODO: handle parsing <ol> tags correctly

        debug!(
            "parse dom node: {:?}, style: {:?}, args: {:?}",
            node, style, args
        );

        let mut result = HTMLTextParsedResult::default();
        let mut suffix = StyledString::new();

        let mut visit_block_element_cb = || {
            if !args.is_first_element_in_block {
                result.content.append_plain("\n\n");
                result.content.append_styled(&args.prefix, style);
            }
            args.is_first_element_in_block = true;
        };

        let mut has_non_ws_text = false;

        match &node.data {
            NodeData::Text { contents } => {
                let content = contents.borrow().to_string();

                let text = if args.in_pre_node {
                    // add `prefix` to each line of the text inside the `<pre>` tag
                    content.replace('\n', &format!("\n{}", args.prefix))
                } else {
                    // Otherwise, consecutive whitespaces are ignored for non-pre elements.
                    // This is to prevent reader-mode engine from adding unneccesary line wraps/indents in a paragraph.
                    WS_RE.replace_all(&content, " ").to_string()
                };
                let text = decode_html(&text);
                debug!("visit text: {}", text);

                has_non_ws_text |= !text.trim().is_empty();

                result.content.append_styled(text, style);
            }
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                debug!("visit element: name={:?}, attrs: {:?}", name, attrs);

                let component_style = &config::get_config_theme().component_style;

                match name.expanded() {
                    expanded_name!(html "h1")
                    | expanded_name!(html "h2")
                    | expanded_name!(html "h3")
                    | expanded_name!(html "h4")
                    | expanded_name!(html "h5")
                    | expanded_name!(html "h6") => {
                        visit_block_element_cb();

                        style = style.combine(component_style.header);
                    }
                    expanded_name!(html "br") => {
                        result
                            .content
                            .append_styled(format!("\n{}", args.prefix), style);
                    }
                    expanded_name!(html "p") => visit_block_element_cb(),
                    expanded_name!(html "code") => {
                        if !args.in_pre_node {
                            // this assumes that `<code>` element that is not inside a pre node
                            // is a single-line code block.
                            style = style.combine(component_style.single_code_block);
                        }
                    }
                    expanded_name!(html "pre") => {
                        visit_block_element_cb();

                        args.in_pre_node = true;
                        args.prefix = format!("{}  ", args.prefix);

                        style = style.combine(component_style.multiline_code_block);

                        result.content.append_styled("  ", style);
                    }
                    expanded_name!(html "blockquote") => {
                        visit_block_element_cb();

                        args.prefix = format!("{}▎ ", args.prefix);
                        style = style.combine(component_style.quote);

                        result.content.append_styled("▎ ", style);
                    }
                    expanded_name!(html "table") => {
                        let mut table_result = HTMLTableParsedResult::default();
                        Self::parse_html_table(
                            node.clone(),
                            max_width,
                            base_link_id + result.links.len(),
                            style,
                            false,
                            &mut table_result,
                        );

                        result.links.append(&mut table_result.links);

                        let mut table = comfy_table::Table::new();
                        table
                            .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
                            .set_width(max_width as u16)
                            .load_preset(comfy_table::presets::UTF8_FULL)
                            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
                            .apply_modifier(comfy_table::modifiers::UTF8_SOLID_INNER_BORDERS)
                            .set_header(
                                table_result
                                    .headers
                                    .into_iter()
                                    .map(|h| comfy_table::Cell::new(h.source()))
                                    .collect::<Vec<_>>(),
                            );

                        for row in table_result.rows {
                            table.add_row(row.into_iter().map(|c| c.source().to_owned()));
                        }

                        result.content.append_styled(format!("\n\n{table}"), style);

                        return (result, true);
                    }
                    expanded_name!(html "menu")
                    | expanded_name!(html "ul")
                    | expanded_name!(html "ol") => {
                        // currently, <ol> tag is treated the same as <ul> tag
                        args.prefix = format!("{}  ", args.prefix);
                    }
                    expanded_name!(html "li") => {
                        args.is_first_element_in_block = true;

                        result
                            .content
                            .append_styled(format!("\n{}• ", args.prefix), style);
                    }
                    expanded_name!(html "img") => {
                        let img_desc = if let Some(attr) = attrs
                            .borrow()
                            .iter()
                            .find(|&attr| attr.name.expanded() == expanded_name!("", "alt"))
                        {
                            attr.value.to_string()
                        } else {
                            String::new()
                        };

                        if !args.is_first_element_in_block {
                            result.content.append_plain("\n\n");
                        }
                        result.content.append_styled(&img_desc, style);
                        result
                            .content
                            .append_styled(" (image)", component_style.metadata);
                    }
                    expanded_name!(html "a") => {
                        // find `href` attribute of an <a> tag
                        if let Some(attr) = attrs
                            .borrow()
                            .iter()
                            .find(|&attr| attr.name.expanded() == expanded_name!("", "href"))
                        {
                            result.links.push(attr.value.clone().to_string());

                            suffix.append_styled(" ", style);
                            suffix.append_styled(
                                format!("[{}]", result.links.len() + base_link_id),
                                component_style.link_id,
                            );
                        }

                        style = style.combine(component_style.link);
                    }
                    expanded_name!(html "strong") => {
                        style = style.combine(component_style.bold);
                    }
                    expanded_name!(html "em") => {
                        style = style.combine(component_style.italic);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        node.children.borrow().iter().for_each(|node| {
            let (child_result, child_has_non_ws_text) = Self::parse_dom_node(
                node.clone(),
                max_width,
                base_link_id + result.links.len(),
                style,
                args.clone(),
            );

            result.merge(child_result);
            has_non_ws_text |= child_has_non_ws_text;
            if has_non_ws_text {
                args.is_first_element_in_block = false;
            }
        });

        result.content.append(suffix);
        (result, has_non_ws_text)
    }

    fn parse_html_table(
        node: Handle,
        max_width: usize,
        base_link_id: usize,
        style: Style,
        mut is_header: bool,
        result: &mut HTMLTableParsedResult,
    ) {
        debug!("parse html table: {:?}", node);

        if let NodeData::Element { name, .. } = &node.data {
            match name.expanded() {
                expanded_name!(html "thead") => {
                    is_header = true;
                }
                expanded_name!(html "tbody") => {
                    is_header = false;
                }
                expanded_name!(html "tr") => {
                    if !is_header {
                        result.rows.push(vec![]);
                    }
                }
                expanded_name!(html "td") | expanded_name!(html "th") => {
                    let mut s = StyledString::new();

                    node.children.borrow().iter().for_each(|node| {
                        let (mut child_result, _) = Self::parse_dom_node(
                            node.clone(),
                            max_width,
                            base_link_id + result.links.len(),
                            style,
                            ArticleParseArgs::default(),
                        );

                        result.links.append(&mut child_result.links);
                        s.append(child_result.content);
                    });

                    if !is_header {
                        result.rows.last_mut().unwrap().push(s);
                    } else {
                        result.headers.push(s);
                    }

                    return;
                }
                _ => {}
            }
        }

        node.children.borrow().iter().for_each(|node| {
            Self::parse_html_table(
                node.clone(),
                max_width,
                base_link_id + result.links.len(),
                style,
                is_header,
                result,
            );
        });
    }
}
