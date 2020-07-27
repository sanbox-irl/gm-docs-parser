use crate::{GmExample, GmFunction, GmParameter};
use ego_tree::NodeRef;
use scraper::{html::Select, ElementRef, Html, Node, Selector};
use selectors::attr::CaseSensitivity;
use std::ops::Deref;
use std::path::Path;

pub fn parse_function_file(fpath: &Path) -> Option<GmFunction> {
    println!("{:?}", fpath);
    let doc = Html::parse_document(&std::fs::read_to_string(fpath).unwrap());
    let h1_sel = Selector::parse("h1").unwrap();
    let h4_sel = Selector::parse("h4").unwrap();

    let mut malformed = vec![];
    let name_description = parse_name_and_description(&doc, &h1_sel);
    let mut h4_select = doc.select(&h4_sel);
    let parameters = parse_parameters(&mut h4_select, &mut malformed);
    let returns =
        parse_returns(&mut h4_select, &mut malformed).unwrap_or_else(|| "N/A".to_string());
    let example = parse_example(&mut h4_select, &mut malformed);

    // did we fuckin nail it?
    if malformed.is_empty() {
        if name_description.is_some() && parameters.is_some() && example.is_some() {
            let (parameters, required_parameters, is_variadic) = parameters.unwrap();
            let (name, description) = name_description.unwrap();
            let f = GmFunction {
                name,
                parameters,
                is_variadic,
                required_parameters,
                example: example.unwrap(),
                description,
                returns,
                link: fpath.to_path_buf(),
            };
            Some(f)
        } else {
            println!("failed to parse, but no errors?");
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

fn parse_parameters(
    select: &mut Select,
    malformed: &mut Vec<String>,
) -> Option<(Vec<GmParameter>, usize, bool)> {
    select.next().and_then(|syntax| {
        if syntax.first_child()?.value().as_text()?.deref() != "Syntax:" {
            malformed.push("couldn't find syntax h4 tag".to_string());
            return None;
        }

        let mut syntax_siblings = syntax.next_siblings();

        // let mut parameter_status = vec![];

        syntax_siblings.next(); // skip newline

        // parse the signature for optionals...
        let signature = syntax_siblings.next()?;
        if signature
            .value()
            .as_element()?
            .has_class("code", CaseSensitivity::AsciiCaseInsensitive)
            == false
        {
            malformed.push("couldn't find the signature line...".to_string());
            return None;
        }

        let mut sig = String::new();
        flatten_element_into_markdown(&signature, &mut sig);
        let (mut param_guesses, mut variadic) = parse_signature(&sig);

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
                                   // gm_parameter.parameter =
                                   //     td.next()?.first_child()?.value().as_text()?.to_string();

                        flatten_element_into_markdown(&td.next()?, &mut gm_parameter.parameter);

                        td.next(); // newline

                        flatten_element_into_markdown(&td.next()?, &mut gm_parameter.description);

                        let is_optional = gm_parameter.parameter.contains("optional")
                            || gm_parameter.parameter.contains("Optional")
                            || gm_parameter.description.contains("optional")
                            || gm_parameter.description.contains("Optional");

                        let is_variadic = gm_parameter.parameter.contains("..")
                            || gm_parameter.description.contains("..");

                        if is_variadic && variadic == false {
                            variadic = true;
                        }

                        if param_guesses.len() < parameters.len() {
                            param_guesses.push(false);
                        }

                        if param_guesses[parameters.len()] == false && is_optional {
                            param_guesses[parameters.len()] = is_optional;
                        }

                        parameters.push(gm_parameter);
                    }
                }
            }

            let min_parameters = param_guesses.iter().position(|v| *v).unwrap_or_default();

            Some((parameters, min_parameters, variadic))
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

fn flatten_element_into_markdown(container: &NodeRef<Node>, output: &mut String) {
    let md = match container.value().as_element().unwrap().name() {
        "i" => Markdown::Italic,
        "b" => Markdown::Bold,
        "a" => {
            if let Some(val) = container.value().as_element().unwrap().attr("href") {
                Markdown::Hyperlink(val.to_string())
            } else {
                Markdown::Plain
            }
        }
        _ => Markdown::Plain,
    };

    for child in container.children() {
        match child.value() {
            Node::Text(txt) => {
                write_in_md(md.clone(), txt, output);
            }
            Node::Element(_) => {
                let mut buff = String::new();
                flatten_element_into_markdown(&child, &mut buff);
                write_in_md(md.clone(), &buff, output);
            }
            _ => continue,
        }
    }
}

fn write_in_md(txt_desc: Markdown, txt: &str, buf: &mut String) {
    match txt_desc {
        Markdown::Hyperlink(dest) => {
            buf.push_str(&format!("[{}]({})", txt, dest));
        }
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
    }
}

fn parse_signature(sig: &str) -> (Vec<bool>, bool) {
    let start = sig.find('(').unwrap();
    let end = sig.find(')').unwrap();

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

    (output, variadic)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Markdown {
    Hyperlink(String),
    Plain,
    Bold,
    Italic,
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
            let (options, variadic) = parse_signature(sig);
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
    fn full_param_tests() {
        fn harness(path: &str, required_parameters: usize, is_variadic: bool) {
            println!("parsing {}...", path);
            let path = Path::new(path);

            let gm_func = parse_function_file(path).unwrap();
            assert_eq!(gm_func.required_parameters, required_parameters);
            assert_eq!(gm_func.is_variadic, is_variadic);
        }

        harness(
            "data/GameMaker_Language/GML_Reference/Variable_Functions/array_create.htm",
            1,
            false,
        );
        harness("data/GameMaker_Language/GML_Reference/Cameras_And_Display/display_set_gui_maximise.htm", 0, false);
    }
}
