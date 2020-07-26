use crate::{GmExample, GmFunction, GmParameter};
use scraper::{html::Select, Html, Node, Selector};
use selectors::attr::CaseSensitivity;
use std::ops::Deref;
use std::path::Path;

pub fn parse_function_file(fpath: &Path) -> Option<GmFunction> {
    let doc = Html::parse_document(&std::fs::read_to_string(fpath).unwrap());
    let h1_sel = Selector::parse("h1").unwrap();
    let h4_sel = Selector::parse("h4").unwrap();

    let mut malformed = vec![];
    let (name, description) = parse_name_and_description(&doc, &h1_sel, &mut malformed);
    let mut h4_select = doc.select(&h4_sel);
    let parameters = parse_parameters(&mut h4_select, &mut malformed);
    let returns =
        parse_returns(&mut h4_select, &mut malformed).unwrap_or_else(|| "N/A".to_string());
    let example = parse_example(&mut h4_select, &mut malformed);

    // did we fuckin nail it?
    if malformed.is_empty() {
        if name.is_some() && description.is_some() && parameters.is_some() && example.is_some() {
            let f = GmFunction {
                name: name.unwrap(),
                parameters: parameters.unwrap(),
                min_parameter: 0,
                max_parameter: 0,
                example: example.unwrap(),
                description: description.unwrap(),
                returns,
                link: fpath.to_path_buf(),
            };
            Some(f)
        } else {
            None
        }
    } else {
        println!("Couldn't parse {:?}", fpath);
        for e in malformed {
            println!("{}", e);
        }
        None
    }
}

fn parse_name_and_description(
    doc: &Html,
    h1_sel: &Selector,
    malformed: &mut Vec<String>,
) -> (Option<String>, Option<String>) {
    if let Some(title) = doc.select(h1_sel).next() {
        let f_child = title.first_child().unwrap();
        let name = Some(f_child.value().as_text().unwrap().to_string());

        let mut sibling_iterator = title.next_siblings();
        sibling_iterator.next(); // skip over the `\n`

        let description = sibling_iterator.next().map(|desc| {
            let mut description = String::new();
            for description_element in desc.children() {
                match description_element.value() {
                    Node::Fragment => {
                        description.push_str(description_element.value().as_text().unwrap());
                    }
                    Node::Text(txt) => {
                        description.push_str(txt);
                    }
                    Node::Element(description_subelement) => {
                        // parse the description subelement, if we *can*
                        let txt_desc: Markdown = match description_subelement.name() {
                            "a" => {
                                if let Some(val) = description_subelement.attr("href") {
                                    Markdown::Hyperlink(val.to_string())
                                } else {
                                    Markdown::Plain
                                }
                            }
                            _ => Markdown::Plain,
                        };

                        // it better have some!
                        let subelement = description_element.first_child().unwrap();
                        match subelement.value() {
                            Node::Text(txt) => write_in_md(txt_desc, txt, &mut description),
                            Node::Element(sub_sub_element) => match sub_sub_element.name() {
                                "tt" => {
                                    let txt = subelement
                                        .first_child()
                                        .unwrap()
                                        .value()
                                        .as_text()
                                        .unwrap();

                                    write_in_md(txt_desc, txt, &mut description);
                                }
                                other => malformed.push(format!(
                                    "unidentified element in description found, tag {:#?}",
                                    other
                                )),
                            },

                            other => malformed.push(format!(
                                "unexpected node type in subelement {}, typeof {:#?}",
                                description_subelement.name(),
                                other
                            )),
                        };
                    }
                    other => {
                        malformed.push(format!(
                            "unexpected node type within description...{:#?}",
                            other
                        ));
                    }
                }
            }

            description
        });

        (name, description)
    } else {
        (None, None)
    }
}

fn parse_parameters(select: &mut Select, malformed: &mut Vec<String>) -> Option<Vec<GmParameter>> {
    select.next().and_then(|syntax| {
        if syntax.first_child()?.value().as_text()?.deref() != "Syntax:" {
            malformed.push("couldn't find syntax h4 tag".to_string());
            return None;
        }

        let mut syntax_siblings = syntax.next_siblings();

        syntax_siblings.next(); // skip newline
        syntax_siblings.next(); // skip signature
        syntax_siblings.next(); // skip newline

        syntax_siblings.next().and_then(|table| {
            if table.value().as_element()?.name() != "table" {
                return None;
            }

            let mut parameters = vec![];

            for tr in table.children().skip(1).next()?.children() {
                if let Node::Element(_) = tr.value() {
                    let is_table_data = tr.children().all(|v| match v.value() {
                        Node::Element(e) => e.name() == "td",
                        // for the silly `\n` children
                        Node::Text(_) => true,
                        _ => false,
                    });

                    if is_table_data {
                        let mut gm_parameter = GmParameter::default();
                        let mut td = tr.children();
                        td.next(); // newline
                        gm_parameter.parameter =
                            td.next()?.first_child()?.value().as_text()?.to_string();
                        td.next(); // newline
                        gm_parameter.documentation =
                            td.next()?.first_child()?.value().as_text()?.to_string();

                        parameters.push(gm_parameter);
                    }
                }
            }

            Some(parameters)
        })
    })
}

fn parse_returns(select: &mut Select, malformed: &mut Vec<String>) -> Option<String> {
    select.next().and_then(|returns| {
        if returns.first_child()?.value().as_text()?.deref() != "Returns:" {
            malformed.push("couldn't find returns h4 tag".to_string());
            return None;
        }

        let mut returns_siblings = returns.next_siblings();
        returns_siblings.next(); // skip newline

        let returns = returns_siblings.next()?;
        if returns
            .value()
            .as_element()?
            .has_class("code", CaseSensitivity::AsciiCaseInsensitive)
        {
            if let Some(child) = returns.first_child() {
                if let Node::Text(txt) = child.value() {
                    return Some(txt.to_string());
                }
            }
        }

        None
    })
}

fn parse_example(select: &mut Select, malformed: &mut Vec<String>) -> Option<GmExample> {
    select.next().and_then(|example| {
        if example.first_child()?.value().as_text()?.deref() != "Example:" {
            malformed.push("couldn't find returns h4 tag".to_string());
            return None;
        }

        let mut example_siblings = example.next_siblings();
        example_siblings.next(); // skip newline

        let example = example_siblings.next()?;
        if example
            .value()
            .as_element()?
            .has_class("code", CaseSensitivity::AsciiCaseInsensitive)
        {
            let mut gm_example = GmExample::default();

            gm_example.code = example.first_child()?.value().as_text()?.to_string();

            example_siblings.next(); // newline
            gm_example.description = example_siblings
                .next()?
                .first_child()?
                .value()
                .as_text()?
                .to_string();

            Some(gm_example)
        } else {
            None
        }
    })
}

fn write_in_md(txt_desc: Markdown, txt: &str, buf: &mut String) {
    match txt_desc {
        Markdown::Hyperlink(dest) => {
            buf.push_str(&format!("[{}]({})", txt, dest));
        }
        Markdown::Plain => {
            buf.push_str(txt);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Markdown {
    Hyperlink(String),
    Plain,
}

impl Default for Markdown {
    fn default() -> Self {
        Markdown::Plain
    }
}
