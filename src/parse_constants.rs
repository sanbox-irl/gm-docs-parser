use super::Markdown;
use fehler::throws;
use gm_docs_parser::*;
use scraper::{ElementRef, Html, Node, Selector};
use std::path::Path;

#[throws(Box<dyn std::error::Error>)]
pub fn parse_constants(base_path: &Path, constants: &mut Vec<GmManualConstant>) {
    for file in std::fs::read_dir(base_path)? {
        let file = file?;
        let file_type = file.file_type()?;

        if file_type.is_dir() {
            parse_constants(&file.path(), constants)?;
        } else if file_type.is_file() {
            let path = file.path();
            if path.extension().map(|e| e == "htm").unwrap_or_default() {
                parse_constant(&file.path(), constants);
            }
        }
    }
}

#[allow(dead_code)]
fn parse_constant(fpath: &Path, constants: &mut Vec<GmManualConstant>) {
    let doc = Html::parse_document(&std::fs::read_to_string(fpath).unwrap());

    for table in doc.select(&Selector::parse("table").unwrap()) {
        parse_inner(table, constants);
    }

    fn parse_inner(table: ElementRef, constants: &mut Vec<GmManualConstant>) -> Option<()> {
        let table_body = table.children().nth(1).unwrap();

        let mut trs = table_body.children();

        // skip the newline
        trs.next(); // bye bye bitch

        // find the header...
        let order = trs
            .next()
            .and_then(|header| {
                header.children().find(|c| {
                    if let Some(e) = c.value().as_element() {
                        e.name() == "th"
                    } else {
                        false
                    }
                })
            })
            .and_then(|th| {
                let is_constant = th
                    .first_child()
                    .map(|header_v| {
                        let mut header = Markdown::convert_to_markdown(&header_v);

                        header.make_ascii_lowercase();

                        header.contains("constant")
                    })
                    .unwrap_or_default();

                if is_constant {
                    let mut order = vec![Order::Constant];

                    let mut th = th;
                    while let Some(sibling) = th.next_sibling() {
                        if let Node::Element(e) = sibling.value() {
                            if e.name() == "th" {
                                if let Some(next_header) = sibling.first_child().map(|v| {
                                    let output = Markdown::convert_to_markdown(&v);

                                    output
                                }) {
                                    if next_header.to_lowercase().contains("description") {
                                        order.push(Order::Description);
                                    } else {
                                        order.push(Order::Other(next_header));
                                    }
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        th = sibling;
                    }

                    Some(order)
                } else {
                    None
                }
            });

        if let Some(order) = order {
            for tr in trs {
                let mut caret = 0;
                let mut constant_doc = GmManualConstant::default();

                if tr.value().is_element() {
                    for td in tr.children() {
                        // there are Text(\n) hiddin in the trs
                        if td.value().is_element() {
                            let data = Markdown::convert_to_markdown(&td);

                            match &order[caret] {
                                Order::Constant => {
                                    constant_doc.name = data;
                                }
                                Order::Description => {
                                    constant_doc.description = data;
                                }
                                Order::Other(e) => {
                                    constant_doc
                                        .secondary_description
                                        .get_or_insert_with(|| Default::default())
                                        .insert(e.clone(), data);
                                }
                            }
                            caret += 1;
                        }
                    }

                    // clean up the constant...
                    if constant_doc.name.starts_with('`') && constant_doc.name.ends_with('`') {
                        constant_doc.name =
                            constant_doc.name[1..constant_doc.name.len() - 1].to_owned();
                    }

                    if let Some(inner) = &mut constant_doc.secondary_description {
                        *inner = inner
                            .clone()
                            .into_iter()
                            .filter(|(k, v)| !(k.trim().is_empty() || v.trim().is_empty()))
                            .collect();

                        if constant_doc.description.trim().is_empty() && inner.is_empty() == false {
                            constant_doc.description =
                                inner.remove(&inner.keys().nth(0).unwrap().clone()).unwrap();
                        }

                        if inner.is_empty() {
                            constant_doc.secondary_description = None;
                        }
                    }

                    constants.push(constant_doc);
                }
            }
        }
        Some(())
    }
}
#[derive(Debug)]
enum Order {
    Constant,
    Description,
    Other(String),
}
