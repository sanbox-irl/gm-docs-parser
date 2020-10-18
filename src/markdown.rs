use ego_tree::NodeRef;
use log::error;
use scraper::{node::Element, Node};
use std::{fmt, path::Path, path::PathBuf};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Markdown {
    txt: String,
    style: Style,
}

impl fmt::Display for Markdown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let txt = self.txt.trim();
        match &self.style {
            Style::Hyperlink(dest) => write!(
                f,
                " [{}]({})",
                txt,
                super::parse_fnames::convert_to_url(dest)
            ),
            Style::Tooltip(dest) => write!(f, "{} ({})", txt, dest),
            Style::Plain => write!(f, "{}", txt),
            Style::Bold => write!(f, "**{}**", txt),
            Style::Italic => write!(f, "*{}*", txt),
            Style::CodeSnippet => write!(f, "`{}`", txt),
            Style::CodeFull => write!(f, "```\n{}\n```", txt),
        }
    }
}

impl Markdown {
    fn new(txt: String, style: Style) -> Markdown {
        Markdown { txt, style }
    }

    pub fn convert_to_markdown(directory: &Path, container: &NodeRef<Node>) -> String {
        let mut st = Vec::new();
        Self::flatten_container(container, directory, &mut st);

        let st = Self::simplify_markdown(st);
        let mut output = String::new();

        for md in st {
            output.push_str(&md.to_string());
        }

        output
    }

    fn simplify_markdown(input: Vec<Markdown>) -> Vec<Markdown> {
        if input.is_empty() {
            return input;
        }

        let mut output = Vec::with_capacity(input.capacity());

        let mut initial = true;
        let mut current_markdown = Markdown::new(String::new(), Style::Plain);
        for md in input {
            if initial {
                current_markdown.style = md.style.clone();
            }

            if current_markdown.style == md.style {
                if md.style.is_combinatorial() == false {
                    if initial {
                        current_markdown.txt = md.txt.clone();
                    } else {
                        output.push(current_markdown);
                        current_markdown = md;
                    }
                } else {
                    current_markdown.txt.push_str(&md.txt);
                }
            } else {
                output.push(current_markdown);
                current_markdown = md;
            }
            initial = false;
        }
        output.push(current_markdown);

        output
    }

    fn flatten_container(container: &NodeRef<Node>, directory: &Path, output: &mut Vec<Markdown>) {
        if let Some(txt) = container.value().as_text() {
            output.push(Markdown::new(txt.to_string(), Style::Plain));
            return;
        }

        let this_container = container.value().as_element().unwrap();
        let style = match this_container.name() {
            "i" | "em" => Style::Italic,
            "b" | "strong" | "h4" => Style::Bold,
            "a" => {
                if let Some(val) = this_container.attr("href") {
                    Style::Hyperlink(directory.join(val))
                } else if let Some(val) = this_container.attr("class") {
                    if val == "tooltip" {
                        Style::Tooltip(val.to_string())
                    } else {
                        Style::Plain
                    }
                } else {
                    // like what is going on here...
                    Style::Plain
                }
            }
            "img" => {
                if let Some(val) = this_container.attr("src") {
                    Style::Hyperlink(directory.join(val))
                } else {
                    error!("We had an <img> with no src!");
                    Style::Plain
                }
            }
            "p" => {
                if this_container.attr("class") == Some("code") {
                    Style::CodeFull
                } else {
                    Style::Plain
                }
            }
            "tt" => Style::CodeSnippet,
            "td" | "br" | "span" | "font" => Style::Plain,
            o => {
                error!("Unknown tag encountered {}", o);
                Style::Plain
            }
        };

        let mut wrote = false;

        for child in container.children() {
            match child.value() {
                Node::Text(txt) => {
                    output.push(Markdown::new(txt.to_string(), style.clone()));
                    wrote = true;
                }
                Node::Element(_) => {
                    let mut buff = Vec::new();
                    Self::flatten_container(&child, directory, &mut buff);
                    output.append(&mut buff);
                    wrote = true;
                }
                _ => continue,
            }
        }

        if wrote == false {
            if let Some(yup) = Self::flat_make_md(style, this_container) {
                output.push(yup);
            }
        }
    }

    fn flat_make_md(txt_desc: Style, e: &Element) -> Option<Markdown> {
        if let Style::Hyperlink(dest) = txt_desc {
            e.attr("alt").map(|txt| {
                Markdown::new(
                    format!("[{}]({})", txt, super::parse_fnames::convert_to_url(&dest)),
                    Style::Plain,
                )
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
enum Style {
    Hyperlink(PathBuf),
    Tooltip(String),
    Plain,
    Bold,
    Italic,
    CodeSnippet,
    CodeFull,
}

impl Style {
    pub fn is_combinatorial(&self) -> bool {
        match self {
            Style::Hyperlink(_) | Style::Tooltip(_) => false,
            Style::Plain | Style::Bold | Style::Italic | Style::CodeSnippet | Style::CodeFull => {
                true
            }
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::Plain
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_whatever() {
        fn harness(input: Vec<Markdown>, output: Vec<Markdown>) {
            let simp = Markdown::simplify_markdown(input);

            assert_eq!(simp, output);
        }

        harness(
            vec![
                Markdown::new("a".to_string(), Style::CodeFull),
                Markdown::new("b".to_string(), Style::CodeFull),
            ],
            vec![Markdown::new("ab".to_string(), Style::CodeFull)],
        );

        harness(
            vec![
                Markdown::new("a".to_string(), Style::CodeFull),
                Markdown::new("b".to_string(), Style::Bold),
                Markdown::new("c".to_string(), Style::CodeFull),
            ],
            vec![
                Markdown::new("a".to_string(), Style::CodeFull),
                Markdown::new("b".to_string(), Style::Bold),
                Markdown::new("c".to_string(), Style::CodeFull),
            ],
        );

        harness(
            vec![
                Markdown::new("a".to_string(), Style::CodeFull),
                Markdown::new("b".to_string(), Style::Bold),
                Markdown::new("b".to_string(), Style::Bold),
                Markdown::new("c".to_string(), Style::CodeFull),
            ],
            vec![
                Markdown::new("a".to_string(), Style::CodeFull),
                Markdown::new("bb".to_string(), Style::Bold),
                Markdown::new("c".to_string(), Style::CodeFull),
            ],
        );

        harness(
            vec![
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
            ],
            vec![
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
            ],
        );

        harness(
            vec![
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
                Markdown::new("b".to_string(), Style::Italic),
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
            ],
            vec![
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
                Markdown::new("b".to_string(), Style::Italic),
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
            ],
        );

        harness(
            vec![
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
                Markdown::new("b".to_string(), Style::Italic),
                Markdown::new("hello".to_string(), Style::Italic),
            ],
            vec![
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
                Markdown::new(
                    "a".to_string(),
                    Style::Hyperlink(Path::new("hey").to_owned()),
                ),
                Markdown::new("bhello".to_string(), Style::Italic),
            ],
        );
    }
}
