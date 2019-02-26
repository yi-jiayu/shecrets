extern crate toml;

use std::fmt::{Display, Error, Formatter};
use std::io::{self, Read};
use toml::value::{Array, Table};
use toml::Value;

const DEFAULT_ARRAY_SEP: &str = ":";

struct Var<'a> {
    key: Vec<&'a str>,
    value: Box<Display + 'a>,
}

#[derive(Debug, PartialEq)]
struct ArrayValue<'a> {
    sep: &'a str,
    elems: Vec<&'a str>,
}

impl<'a> ArrayValue<'a> {
    pub fn from_array(array: &'a Array, sep: &'a str) -> Option<ArrayValue<'a>> {
        array
            .first()
            .and_then(|val| {
                if val.is_str() {
                    Some(array.iter().map(|v| v.as_str().unwrap()).collect())
                } else {
                    None
                }
            })
            .map(|elems| ArrayValue { sep, elems })
    }
}

impl<'a> Display for ArrayValue<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.elems.join(self.sep))
    }
}

fn walk(config: &Table) -> Vec<Var> {
    let mut stack: Vec<(Vec<&str>, &Table)> = Vec::new();
    let mut vars: Vec<Var> = Vec::new();
    stack.push((Vec::new(), config));
    while let Some((prefix, table)) = stack.pop() {
        for (k, v) in table {
            if k.starts_with('_') {
                continue;
            }
            let mut prefix = prefix.to_owned();
            prefix.push(k);
            match v {
                Value::String(s) => vars.push(Var {
                    key: prefix,
                    value: Box::new(s.as_str()),
                }),
                Value::Integer(i) => vars.push(Var {
                    key: prefix,
                    value: Box::new(i),
                }),
                Value::Float(f) => vars.push(Var {
                    key: prefix,
                    value: Box::new(f),
                }),
                Value::Boolean(b) => vars.push(Var {
                    key: prefix,
                    value: Box::new(b),
                }),
                Value::Array(a) => {
                    let sep = table
                        .get(format!("_{}_separator", k).as_str())
                        .and_then(|v| v.as_str())
                        .unwrap_or(DEFAULT_ARRAY_SEP);
                    if let Some(av) = ArrayValue::from_array(a, sep) {
                        vars.push(Var {
                            key: prefix,
                            value: Box::new(av),
                        })
                    }
                }
                Value::Table(t) => stack.push((prefix, t)),
                _ => (),
            };
        }
    }
    vars
}

fn format_posix(var: &Var) -> String {
    format!(
        "{0}={1}; export {0}",
        var.key.join("_").to_uppercase(),
        var.value
    )
}

fn format_vars(vars: &[Var]) -> Vec<String> {
    vars.iter().map(|var| format_posix(var)).collect()
}

fn main() -> Result<(), String> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| format!("error reading standard input: {}", e))?;
    let val = buffer
        .parse::<Value>()
        .map_err(|e| format!("invalid toml: {}", e))?;
    // valid toml should always be a table at the top level
    let vars = walk(val.as_table().unwrap());
    let formatted = format_vars(&vars);
    for s in formatted {
        println!("{}", s);
    }
    Ok(())
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
    fn test_array_value_display() {
        let arr = ArrayValue {
            sep: "X",
            elems: vec!["/bin", "/usr/bin", "/usr/local/bin"],
        };
        let actual = format!("{}", arr);
        let expected = "/binX/usr/binX/usr/local/bin";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_array_value_from_array() {
        let val = r#"arr = ["/bin", "/usr/bin", "/usr/local/bin"]"#.parse::<Value>().unwrap();
        let arr = val.get("arr").unwrap().as_array().unwrap();
        let actual = ArrayValue::from_array(arr, DEFAULT_ARRAY_SEP).unwrap();
        let expected = ArrayValue {
            sep: ":",
            elems: vec!["/bin", "/usr/bin", "/usr/local/bin"],
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test() {
        let toml_str = r#"somefloatvar = 1.2
someintvar = 1
someboolval = true
somearrayval = ["abc","def"]
[section]
pipeseperatedarrayval = ["abc","def"]
_pipeseperatedarrayval_separator = "|"
nestedvar = "value2"
[section.nested]
doublynestedvar = "value3"
[section2]
nestedvar = "value4"
"#;
        let config = toml_str.parse::<Value>().unwrap();
        let vars = walk(config.as_table().unwrap());
        let formatted = format_vars(&vars);
        assert_eq!(
            formatted,
            [
                "SOMEARRAYVAL=abc:def; export SOMEARRAYVAL",
                "SOMEBOOLVAL=true; export SOMEBOOLVAL",
                "SOMEFLOATVAR=1.2; export SOMEFLOATVAR",
                "SOMEINTVAR=1; export SOMEINTVAR",
                "SECTION2_NESTEDVAR=value4; export SECTION2_NESTEDVAR",
                "SECTION_NESTEDVAR=value2; export SECTION_NESTEDVAR",
                "SECTION_PIPESEPERATEDARRAYVAL=abc|def; export SECTION_PIPESEPERATEDARRAYVAL",
                "SECTION_NESTED_DOUBLYNESTEDVAR=value3; export SECTION_NESTED_DOUBLYNESTEDVAR",
            ]
        );
    }
}
