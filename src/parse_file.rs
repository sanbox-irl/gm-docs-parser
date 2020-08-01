use crate::{FunctionDoc, GmFunctionParameter};
use ego_tree::NodeRef;
use log::*;
use scraper::{html::Select, node::Element, Html, Node, Selector};
// use selectors::attr::CaseSensitivity;
use gm_docs_parser::{ConstantDoc, VariableDoc};
use std::ops::Deref;
use std::path::Path;

pub enum DocEntry {
    Function(FunctionDoc),
    Variable(VariableDoc),
}

pub fn parse_function_file(fpath: &Path, constants: &mut Vec<ConstantDoc>) -> Option<DocEntry> {
    trace!("{:?}", fpath);
    let doc = Html::parse_document(&std::fs::read_to_string(fpath).unwrap());
    let h1_sel = Selector::parse("h1").unwrap();
    let h4_sel = Selector::parse("h4").unwrap();

    let name_description = parse_name_and_description(&doc, &h1_sel);
    let mut h4_select = doc.select(&h4_sel);
    let parameters = parse_parameters(&mut h4_select);
    let returns = parse_returns(&mut h4_select);
    let example = parse_example(&mut h4_select);

    // attempt to parse for constants -- this is basically unrelated to anything else
    let table_sel = Selector::parse("table").unwrap();
    parse_constants(&doc, &table_sel, constants);

    // did we fuckin nail it?
    let all_success = name_description.is_some() && example.is_some() && returns.is_some();
    if all_success {
        let (name, description) = name_description.unwrap();

        let output = match parameters.unwrap() {
            Data::Function {
                parameters,
                required_parameters,
                is_variadic,
            } => DocEntry::Function(FunctionDoc {
                name,
                parameters,
                is_variadic,
                required_parameters,
                example: example.unwrap(),
                description,
                returns: returns.unwrap(),
                link: fpath.to_path_buf(),
            }),
            Data::Variable => DocEntry::Variable(VariableDoc {
                name,
                example: example.unwrap(),
                description,
                returns: returns.unwrap(),
                link: fpath.to_path_buf(),
            }),
        };

        Some(output)
    } else {
        error!(
            "FAIL! {:?}\n..name_desc [{}], parameters [{}], example [{}], returns [{}]",
            fpath,
            if name_description.is_some() { "X" } else { " " },
            if parameters.is_some() { "X" } else { " " },
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
    let mut description = String::new();
    flatten_element_into_markdown(&desc, &mut description);

    Some((name, description))
}

enum Data {
    Function {
        parameters: Vec<GmFunctionParameter>,
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
                    let mut syntax_output = String::new();
                    flatten_element_into_markdown(&child, &mut syntax_output);
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

            let mut sig = String::new();
            flatten_element_into_markdown(&signature, &mut sig);
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

                    for tr in table.children().nth(1)?.children() {
                        if let Node::Element(_) = tr.value() {
                            let is_table_data = tr.children().all(|v| match v.value() {
                                Node::Element(e) => e.name() == "td",
                                // for the silly `\n` children
                                Node::Text(_) => true,
                                _ => false,
                            });

                            if is_table_data {
                                let mut gm_parameter = GmFunctionParameter::default();
                                let mut td = tr.children();
                                td.next(); // newline
                                           // gm_parameter.parameter =
                                           //     td.next()?.first_child()?.value().as_text()?.to_string();

                                flatten_element_into_markdown(
                                    &td.next()?,
                                    &mut gm_parameter.parameter,
                                );

                                td.next(); // newline

                                flatten_element_into_markdown(
                                    &td.next()?,
                                    &mut gm_parameter.description,
                                );

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
                    let mut example_output = String::new();
                    flatten_element_into_markdown(&v, &mut example_output);
                    example_output.make_ascii_lowercase();

                    example_output.contains("example")
                })
                .unwrap_or_default()
        })
        .and_then(|example| {
            let mut example_siblings = example.next_siblings();
            example_siblings.next(); // skip newline

            let example = example_siblings.next()?;
            let mut gm_example = String::new();

            flatten_element_into_markdown(&example, &mut gm_example);
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
                        let mut next_one = String::new();
                        flatten_element_into_markdown(&ex, &mut next_one);

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
                    let mut example_output = String::new();
                    flatten_element_into_markdown(&v, &mut example_output);
                    example_output.make_ascii_lowercase();

                    example_output.contains("returns")
                })
                .unwrap_or_default()
        })
        .and_then(|returns| {
            let mut returns_siblings = returns.next_siblings();
            returns_siblings.next(); // skip newline
            let returns = returns_siblings.next()?;

            let mut output = String::new();

            flatten_element_into_markdown(&returns, &mut output);

            Some(output)
        })
}

fn flatten_element_into_markdown(container: &NodeRef<Node>, output: &mut String) {
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
                write_in_md(md.clone(), txt, output);
                wrote = true;
            }
            Node::Element(_) => {
                let mut buff = String::new();
                flatten_element_into_markdown(&child, &mut buff);
                write_in_md(md.clone(), &buff, output);
                wrote = true;
            }
            _ => continue,
        }
    }

    if wrote == false {
        emergency_write_in_md(md, this_container, output);
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

fn parse_constants(
    doc: &Html,
    constant_selector: &Selector,
    constants: &mut Vec<ConstantDoc>,
) -> Option<()> {
    for table in doc.select(constant_selector) {
        let table_body = table.children().nth(1).unwrap();

        let mut trs = table_body.children();

        // skip the newline
        trs.next(); // bye bye bitch

        // find the header...
        let is_constant = trs
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
                        let mut header = String::new();
                        flatten_element_into_markdown(&header_v, &mut header);

                        header.make_ascii_lowercase();

                        header.contains("constant")
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        if is_constant == false {
            continue;
        }

        for tr in trs {
            if let Node::Element(_) = tr.value() {
                let mut constant_doc = ConstantDoc::default();
                let mut td = tr.children();
                td.next(); // newline
                           // gm_parameter.parameter =
                           //     td.next()?.first_child()?.value().as_text()?.to_string();

                flatten_element_into_markdown(&td.next()?, &mut constant_doc.name);

                td.next(); // newline

                let nxt = &td.next()?;
                println!("{:#?}", nxt.value());
                flatten_element_into_markdown(nxt, &mut constant_doc.description);

                println!("{:#?}", constant_doc);
            }
        }

        // for tr in table_body.children() {
        // }
    }

    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Markdown {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signature_test() {
        fn harness(sig: &str, optionals: Vec<bool>, is_variadic: bool) {
            let (options, variadic, _) = parse_signature(sig);
            assert_eq!(options, optionals);
            assert_eq!(variadic, is_variadic);
        }

        harness(
            "instance_destroy([id, execute_event_flag]);",
            vec![true, true],
            false,
        );

        harness(
            "shader_set_uniform_f(handle, value1 [, value2, value3, value4]);",
            vec![false, false, true, true, true],
            false,
        );

        harness(
            "choose(val0, val1, val2... max_val);",
            vec![false, false, false],
            true,
        );

        harness(
            "place_empty(x, y, [object_id]);",
            vec![false, false, true],
            false,
        );

        harness(
            "ds_list_add(id, val1 [, val2, ... max_val]);",
            vec![false, false, true, true],
            true,
        );

        harness(
            "ds_list_add(id, val1 [, val2, ... max_val]);
            ",
            vec![false, false, true, true],
            true,
        );

        harness(
            "display_set_gui_maximise(<i>xscale, yscale, xoffset, yoffset</i>);",
            vec![false, false, false, false],
            false,
        );
    }

    #[test]
    fn consts() {
        fn harness(p: &str) {
            let doc = Html::parse_document(&std::fs::read_to_string(&p).unwrap());

            let table_sel = Selector::parse("table").unwrap();
            let mut consts = vec![];

            parse_constants(&doc, &table_sel, &mut consts).unwrap();
        }

        harness("data\\GameMaker_Language\\GML_Reference\\Drawing\\Text\\draw_get_halign.htm");
    }
}
