extern crate toml;

use std::io::{self, Read};
use std::collections::VecDeque;
use toml::Value as Value;
use toml::value::Table as Table;

struct Var<'a> {
    key: Vec<&'a str>,
    value: &'a str,
}

fn walk(config: &Table) -> Vec<Var> {
    let mut queue: VecDeque<(Vec<&str>, &Table)> = VecDeque::new();
    let mut vars: Vec<Var> = Vec::new();
    queue.push_back((Vec::new(), config));
    while let Some((prefix, value)) = queue.pop_front() {
        for (k, v) in value {
            let mut prefix = prefix.to_owned();
            prefix.push(&k);
            match v {
                Value::String(s) => vars.push(Var { key: prefix, value: s.as_str() }),
                Value::Table(t) => queue.push_back((prefix, t)),
                _ => ()
            };
        };
    };
    vars
}

fn format_vars(vars: Vec<Var>) -> Vec<String> {
    vars.iter()
        .map(|var| format!("{0}={1}; export {0}",
                           var.key.join("_").to_uppercase(),
                           var.value))
        .collect()
}

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let val = buffer.parse::<Value>().unwrap();
    let vars = walk(val.as_table().unwrap());
    let formatted = format_vars(vars);
    for s in formatted {
        println!("{}", s);
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let config = r#"somevar = "value1"
[section]
nestedvar = "value2"
[section.nested]
doublynestedvar = "value3"
"#.parse::<Value>().unwrap();
        let vars = walk(config.as_table().unwrap());
        let formatted = format_vars(vars);
        assert_eq!(formatted,
                   [
                       "SOMEVAR=value1; export SOMEVAR",
                       "SECTION_NESTEDVAR=value2; export SECTION_NESTEDVAR",
                       "SECTION_NESTED_DOUBLYNESTEDVAR=value3; export SECTION_NESTED_DOUBLYNESTEDVAR"
                   ]);
    }
}
