extern crate toml;

use std::fmt::Display;
use std::io::{self, Read};
use toml::Value as Value;
use toml::value::Table as Table;

struct Var<'a> {
    key: Vec<&'a str>,
    value: Box<Display + 'a>,
}

fn walk(config: &Table) -> Vec<Var> {
    let mut stack: Vec<(Vec<&str>, &Table)> = Vec::new();
    let mut vars: Vec<Var> = Vec::new();
    stack.push((Vec::new(), config));
    while let Some((prefix, value)) = stack.pop() {
        for (k, v) in value {
            let mut prefix = prefix.to_owned();
            prefix.push(&k);
            match v {
                Value::String(s) => vars.push(Var { key: prefix, value: Box::new(s.as_str()) }),
                Value::Integer(i) => vars.push(Var { key: prefix, value: Box::new(i) }),
                Value::Float(f) => vars.push(Var { key: prefix, value: Box::new(f) }),
                Value::Table(t) => stack.push((prefix, t)),
                _ => ()
            };
        };
    };
    vars
}

fn format_posix(var: &Var) -> String {
    format!("{0}={1}; export {0}",
            var.key.join("_").to_uppercase(),
            var.value)
}

fn format_vars(vars: &Vec<Var>) -> Vec<String> {
    vars.iter()
        .map(|var| format_posix(var))
        .collect()
}

fn die(msg: &str) {
    eprintln!("{}", msg);
    std::process::exit(1);
}

fn main() {
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        return die("shecrets: error reading standard input");
    }
    let val = match buffer.parse::<Value>() {
        Ok(val) => val,
        Err(_) => return die("shecrets: invalid toml"),
    };
    // valid toml should always be a table at the top level
    let vars = walk(val.as_table().unwrap());
    let formatted = format_vars(&vars);
    for s in formatted {
        println!("{}", s);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_posix() {
        let var = Var {
            key: vec!["pipenv", "venv_in_project"],
            value: Box::new(1),
        };
        let actual = format_posix(&var);
        let expected = "PIPENV_VENV_IN_PROJECT=1; export PIPENV_VENV_IN_PROJECT";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test() {
        let config = r#"somefloatvar = 1.2
someintvar = 1
[section]
nestedvar = "value2"
[section.nested]
doublynestedvar = "value3"
[section2]
nestedvar = "value4"
"#.parse::<Value>().unwrap();
        let vars = walk(config.as_table().unwrap());
        let formatted = format_vars(&vars);
        assert_eq!(formatted,
                   [
                       "SOMEFLOATVAR=1.2; export SOMEFLOATVAR",
                       "SOMEINTVAR=1; export SOMEINTVAR",
                       "SECTION2_NESTEDVAR=value4; export SECTION2_NESTEDVAR",
                       "SECTION_NESTEDVAR=value2; export SECTION_NESTEDVAR",
                       "SECTION_NESTED_DOUBLYNESTEDVAR=value3; export SECTION_NESTED_DOUBLYNESTEDVAR",
                   ]);
    }
}
