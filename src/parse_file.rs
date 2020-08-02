use crate::{parse_fnames::convert_to_url, Markdown};
use gm_docs_parser::{GmManualFunction, GmManualFunctionParameter, GmManualVariable};
use log::*;
use scraper::{html::Select, Html, Node, Selector};
use std::ops::Deref;
use std::path::Path;

#[derive(Debug)]
pub enum DocEntry {
    Function(GmManualFunction),
    Variable(GmManualVariable),
}

pub fn parse_function_file(fpath: &Path, base: &Path) -> Option<DocEntry> {
    trace!("{:?}", fpath);
    let doc = Html::parse_document(&std::fs::read_to_string(fpath).unwrap());
    let h1_sel = Selector::parse("h1").unwrap();
    let h4_sel = Selector::parse("h4").unwrap();

    let name_description = parse_name_and_description(&doc, &h1_sel);
    let mut h4_select = doc.select(&h4_sel);
    let parameters = parse_parameters(&mut h4_select).unwrap_or_else(|| Data::Function {
        parameters: Default::default(),
        required_parameters: 0,
        is_variadic: false,
    });
    let returns = parse_returns(&mut h4_select);
    let example = parse_example(&mut h4_select);

    // did we fuckin nail it?
    let all_success = name_description.is_some() && example.is_some() && returns.is_some();
    if all_success {
        let (name, description) = name_description.unwrap();
        let link = convert_to_url(base, fpath);

        let output = match parameters {
            Data::Function {
                parameters,
                required_parameters,
                is_variadic,
            } => DocEntry::Function(GmManualFunction {
                name,
                parameters,
                is_variadic,
                required_parameters,
                example: example.unwrap(),
                description,
                returns: returns.unwrap(),
                link,
            }),
            Data::Variable => DocEntry::Variable(GmManualVariable {
                name,
                example: example.unwrap(),
                description,
                returns: returns.unwrap(),
                link,
            }),
        };

        Some(output)
    } else {
        error!(
            "FAIL! {:?}\n..name_desc [{}], example [{}], returns [{}]",
            fpath,
            if name_description.is_some() { "X" } else { " " },
            if example.is_some() { "X" } else { " " },
            if returns.is_some() { "X" } else { " " },
        );
        None
    }
}

fn parse_name_and_description(doc: &Html, h1_sel: &Selector) -> Option<(String, String)> {
    let title = doc.select(h1_sel).next()?;
    let f_child = title.first_child()?;
    let name = f_child.value().as_text()?.to_string();

    let mut sibling_iterator = title.next_siblings();
    sibling_iterator.next(); // skip over the `\n`

    let desc = sibling_iterator.next()?;
    let description = Markdown::convert_to_markdown(&desc);

    Some((name, description))
}

enum Data {
    Function {
        parameters: Vec<GmManualFunctionParameter>,
        required_parameters: usize,
        is_variadic: bool,
    },
    Variable,
}

fn parse_parameters(select: &mut Select) -> Option<Data> {
    select
        .find(|v| {
            v.first_child()
                .map(|child| {
                    let mut syntax_output = Markdown::convert_to_markdown(&child);
                    syntax_output.make_ascii_lowercase();
                    syntax_output.contains("syntax")
                })
                .unwrap_or_default()
        })
        .and_then(|syntax| {
            let mut syntax_siblings = syntax.next_siblings();

            // let mut parameter_status = vec![];

            syntax_siblings.next(); // skip newline

            // parse the signature for optionals...
            let signature = syntax_siblings.next()?;

            let sig = Markdown::convert_to_markdown(&signature);
            let (mut param_guesses, mut variadic, is_function) = parse_signature(&sig);
            if is_function == false {
                return Some(Data::Variable);
            }

            syntax_siblings.next(); // skip newline

            syntax_siblings
                .find(|table| {
                    table
                        .value()
                        .as_element()
                        .map(|v| v.name() == "table")
                        .unwrap_or_default()
                })
                .and_then(|table| {
                    let mut parameters = vec![];
                    let mut trs = table.children().nth(1)?.children();
                    trs.next(); // newline by bitch

                    // find the header...
                    let contains_argument = trs
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
                        .map(|th| {
                            th.first_child()
                                .map(|header_v| {
                                    let mut header = Markdown::convert_to_markdown(&header_v);
                                    header.make_ascii_lowercase();

                                    header.contains("argument")
                                })
                                .unwrap_or_default()
                        })
                        .unwrap_or_default();

                    if contains_argument {
                        for tr in trs.into_iter().skip(1) {
                            if tr.value().is_element() {
                                let mut gm_parameter = GmManualFunctionParameter::default();

                                let mut td = tr.children();
                                td.next(); // newline
                                gm_parameter.parameter = Markdown::convert_to_markdown(&td.next()?);

                                td.next(); // newline

                                gm_parameter.description =
                                    Markdown::convert_to_markdown(&td.next()?);

                                let is_optional = gm_parameter.parameter.contains("optional")
                                    || gm_parameter.parameter.contains("Optional")
                                    || gm_parameter.description.contains("optional")
                                    || gm_parameter.description.contains("Optional");

                                let is_variadic = gm_parameter.parameter.contains("..")
                                    || gm_parameter.description.contains("..");

                                if is_variadic && variadic == false {
                                    variadic = true;
                                }

                                if param_guesses.len() <= parameters.len() {
                                    param_guesses.push(false);
                                }

                                if param_guesses[parameters.len()] == false && is_optional {
                                    param_guesses[parameters.len()] = is_optional;
                                }

                                parameters.push(gm_parameter);
                            }
                        }
                    }

                    let minimum_parameters =
                        param_guesses.iter().position(|v| *v).unwrap_or_default();

                    Some(Data::Function {
                        parameters,
                        required_parameters: minimum_parameters,
                        is_variadic: variadic,
                    })
                })
        })
}

fn parse_example(select: &mut Select) -> Option<String> {
    select
        .find(|v| {
            v.first_child()
                .map(|v| {
                    let mut example_output = Markdown::convert_to_markdown(&v);
                    example_output.make_ascii_lowercase();

                    example_output.contains("example")
                })
                .unwrap_or_default()
        })
        .and_then(|example| {
            let mut example_siblings = example.next_siblings();
            example_siblings.next(); // skip newline

            let example = example_siblings.next()?;
            let mut gm_example = Markdown::convert_to_markdown(&example);
            for ex in example_siblings {
                match ex.value() {
                    Node::Text(txt) => {
                        if txt.deref().trim().is_empty() {
                            gm_example.push('\n');
                        } else {
                            gm_example.push_str(txt.deref().trim());
                        }
                    }
                    Node::Element(_) => {
                        let next_one = Markdown::convert_to_markdown(&ex);

                        if next_one.trim().is_empty() {
                            break;
                        }

                        gm_example.push_str(&next_one);
                    }
                    _ => break,
                }
            }

            Some(gm_example)
        })
}

fn parse_returns(select: &mut Select) -> Option<String> {
    select
        .find(|v| {
            v.first_child()
                .map(|v| {
                    let mut example_output = Markdown::convert_to_markdown(&v);
                    example_output.make_ascii_lowercase();

                    example_output.contains("returns")
                })
                .unwrap_or_default()
        })
        .and_then(|returns| {
            let mut returns_siblings = returns.next_siblings();
            returns_siblings.next(); // skip newline
            let returns = returns_siblings.next()?;

            let output = Markdown::convert_to_markdown(&returns);

            Some(output)
        })
}

fn parse_signature(sig: &str) -> (Vec<bool>, bool, bool) {
    let start = sig.find('(');
    let end = sig.find(')');

    let succeeded = start.is_some() && end.is_some();
    if succeeded == false {
        return (vec![], false, false);
    }
    let start = start.unwrap();
    let end = end.unwrap();

    let mut output = vec![];
    let mut variadic = false;

    // for no param args
    if end - start > 2 {
        let parameters = &sig[start + 1..end];

        let mut running_optional = false;

        for param in parameters.split(',') {
            let param = param.trim();
            if param.is_empty() == false {
                if param.starts_with('[') {
                    running_optional = true;
                }
                output.push(running_optional);

                if param.contains('[') {
                    running_optional = true;
                }

                if param.contains(']') {
                    running_optional = false;
                }

                if param.contains("..") {
                    variadic = true;
                }
            }
        }
    }

    (output, variadic, true)
}
