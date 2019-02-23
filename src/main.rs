extern crate toml;

use std::io::{self, Read};
use toml::Value as Value;
use toml::value::Table as Table;

struct Var<'a> {
    key: Vec<&'a str>,
    value: &'a str,
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
                Value::String(s) => vars.push(Var { key: prefix, value: s.as_str() }),
                Value::Table(t) => stack.push((prefix, t)),
                _ => ()
            };
        };
    };
    vars
}

fn format_vars(vars: &Vec<Var>) -> Vec<String> {
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
    let formatted = format_vars(&vars);
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
[section2]
nestedvar = "value4"
"#.parse::<Value>().unwrap();
        let vars = walk(config.as_table().unwrap());
        let formatted = format_vars(&vars);
        assert_eq!(formatted,
                   [
                       "SOMEVAR=value1; export SOMEVAR",
                       "SECTION2_NESTEDVAR=value4; export SECTION2_NESTEDVAR",
                       "SECTION_NESTEDVAR=value2; export SECTION_NESTEDVAR",
                       "SECTION_NESTED_DOUBLYNESTEDVAR=value3; export SECTION_NESTED_DOUBLYNESTEDVAR",
                   ]);
    }
}
