use ego_tree::NodeRef;
use log::error;
use scraper::{node::Element, Node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Markdown {
    Hyperlink(String),
    Tooltip(String),
    Plain,
    Bold,
    Italic,
    CodeSnippet,
    CodeFull,
}

impl Default for Markdown {
    fn default() -> Self {
        Markdown::Plain
    }
}

impl Markdown {
    pub fn convert_to_markdown(container: &NodeRef<Node>) -> String {
        let mut st = String::new();
        Self::to_md(container, &mut st);

        st
    }

    fn to_md(container: &NodeRef<Node>, output: &mut String) {
        if let Some(txt) = container.value().as_text() {
            output.push_str(txt);
            return;
        }

        let this_container = container.value().as_element().unwrap();
        let md = match this_container.name() {
            "i" | "em" => Markdown::Italic,
            "b" | "strong" | "h4" => Markdown::Bold,
            "a" => {
                if let Some(val) = this_container.attr("href") {
                    Markdown::Hyperlink(val.to_string())
                } else if let Some(val) = this_container.attr("class") {
                    if val == "tooltip" {
                        Markdown::Tooltip(val.to_string())
                    } else {
                        Markdown::Plain
                    }
                } else {
                    // like what is going on here...
                    Markdown::Plain
                }
            }
            "img" => {
                if let Some(val) = this_container.attr("src") {
                    Markdown::Hyperlink(val.to_string())
                } else {
                    error!("We had an <img> with no src!");
                    Markdown::Plain
                }
            }
            "p" => {
                if this_container.attr("class") == Some("code") {
                    Markdown::CodeFull
                } else {
                    Markdown::Plain
                }
            }
            "tt" => Markdown::CodeSnippet,
            "td" | "br" | "span" => Markdown::Plain,
            o => {
                error!("Unknown tag encountered {}", o);
                Markdown::Plain
            }
        };

        let mut wrote = false;

        for child in container.children() {
            match child.value() {
                Node::Text(txt) => {
                    Self::write_in_md(md.clone(), txt, output);
                    wrote = true;
                }
                Node::Element(_) => {
                    let mut buff = String::new();
                    Self::to_md(&child, &mut buff);
                    Self::write_in_md(md.clone(), &buff, output);
                    wrote = true;
                }
                _ => continue,
            }
        }

        if wrote == false {
            Self::emergency_write_in_md(md, this_container, output);
        }
    }

    fn write_in_md(txt_desc: Markdown, txt: &str, buf: &mut String) {
        match txt_desc {
            Markdown::Hyperlink(dest) => {
                buf.push_str(&format!("[{}]({})", txt, dest));
            }
            Markdown::Tooltip(dest) => buf.push_str(&format!("{} ({})", txt, dest)),
            Markdown::Plain => {
                buf.push_str(txt);
            }
            Markdown::Bold => {
                buf.push('*');
                buf.push('*');
                buf.push_str(txt);
                buf.push('*');
                buf.push('*');
            }
            Markdown::Italic => {
                buf.push('*');
                buf.push_str(txt);
                buf.push('*');
            }
            Markdown::CodeSnippet => {
                buf.push('`');
                buf.push_str(txt);
                buf.push('`');
            }
            Markdown::CodeFull => {
                buf.push_str("```\n");
                buf.push_str(txt.trim());
                buf.push_str("\n```");
            }
        }
    }

    fn emergency_write_in_md(txt_desc: Markdown, e: &Element, buf: &mut String) {
        if let Markdown::Hyperlink(dest) = txt_desc {
            if let Some(txt) = e.attr("alt") {
                buf.push_str(&format!("[{}]({})", txt, dest));
            }
        }
    }
}
