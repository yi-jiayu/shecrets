# shecrets
Define shell environment variables in a config file

## Introduction

`shecrets` takes a TOML file specifying environment variables and generates some code to export
them.

For example, given the following file content in `.shecrets.toml`:

```toml
TRAVIS_TOKEN = "secrettravistoken"

[datamall]
	account_key = "secretaccountkey"

[pipenv]
	venv_in_project = 1
```

`shecrets` will output:

```console
$ shecrets < .shecrets.toml
TRAVIS_TOKEN=secrettravistoken; export TRAVIS_TOKEN
PIPENV_VENV_IN_PROJECT=1; export PIPENV_VENV_IN_PROJECT
DATAMALL_ACCOUNT_KEY=secretaccountkey; export DATAMALL_ACCOUNT_KEY
```

The output of `shecrets` can be `eval`ed inside your shell startup script:

```sh
eval "$(shecrets < $HOME/.shecrets.toml)"
```
