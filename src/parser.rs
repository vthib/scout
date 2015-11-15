extern crate toml;

use core::Error;

use git::{Branch, Repo, Context};

use std::collections::HashMap;
use std::fs;
use std::io::Read;

// {{{ Helpers

macro_rules! try_toml {
    ($expr:expr, $err:expr) => (match $expr {
        Some(t) => t,
        None => return Err(Error::TomlError($err.to_string())),
    })
}

macro_rules! throw_err {
    ($($arg:tt)*) => (
        return Err(Error::StructuralError(format!($($arg)*)));
    )
}

trait FromToml {
    fn from_toml(&toml::Table) -> Result<Self, Error>;
}

// }}}
// {{{ TOML tables to structures
// {{{ Branch

#[derive(Debug)]
pub struct ParsedBranch {
    pub name: String,
    pub inherits: Vec<String>,
}

impl FromToml for ParsedBranch {
    fn from_toml(table: &toml::Table) -> Result<ParsedBranch, Error> {
        let name = try_toml!(table.get("name").and_then(|v| v.as_str()),
                             "table `branch` should have a `name` attribute");

        let mut branch = ParsedBranch {
            name: name.to_string(),
            inherits: Vec::new(),
        };
        match table.get("inherits") {
            None => Ok(branch),
            Some(v) => {
                let branches = try_toml!(v.as_slice(),
                                      "value `inherits` should be an array");
                for b in branches {
                    let s = try_toml!(b.as_str(),
                                      "`inherits` values should be strings");
                    branch.inherits.push(s.to_string());
                }
                Ok(branch)
            }
        }
    }
}

// }}}
// {{{ Repo

#[derive(Debug)]
pub struct ParsedRepo {
    pub name: String,
    pub branches: HashMap<String, ParsedBranch>,
}

impl FromToml for ParsedRepo {
    fn from_toml(table: &toml::Table) -> Result<ParsedRepo, Error> {
        let name = try_toml!(table.get("name").and_then(|v| v.as_str()),
                             "table `repo` should have a `name` attribute");

        let branches = try_toml!(table.get("branch")
                                      .and_then(|v| v.as_slice()),
                                 "table `repo` should have an array \
                                  `branch`");
        let mut repo = ParsedRepo {
            name: name.to_string(),
            branches: HashMap::new(),
        };
        for branch in branches {
            let brc_table = try_toml!(branch.as_table(),
                                      "value `branch` should be a table");
            let b = try!(ParsedBranch::from_toml(brc_table));

            repo.branches.insert(b.name.to_string(), b);
        }

        Ok(repo)
    }
}

impl FromToml for Repo {
    fn from_toml(table: &toml::Table) -> Result<Repo, Error> {
        let parsed = try!(ParsedRepo::from_toml(table));
        let mut repo = Repo::new(parsed.name.to_string());

        // create Branch objects for each parsed branch
        for name in parsed.branches.keys() {
            repo.add_branch(Branch::new(name.to_string()));
        }

        // add parents for each branch
        for parsed_branch in parsed.branches.values() {
            let mut child = repo.find_branch(&parsed_branch.name)
                                .unwrap().borrow_mut();

            for parent_name in &parsed_branch.inherits {
                if parent_name == &parsed_branch.name {
                    throw_err!("branch `{}` in repo `{}` cannot inherit \
                                from itself", parent_name, parsed.name);
                }
                match repo.find_branch(parent_name) {
                    Some(parent_branch) => child.inherits_from(parent_branch),
                    None => throw_err!("unknown branch `{}` in repo `{}`",
                                       parent_name, parsed.name),
                }
            }
        }

        Ok(repo)
    }
}

// }}}
// {{{ Context

impl FromToml for Context {
    fn from_toml(table: &toml::Table) -> Result<Context, Error> {
        let repos = try_toml!(table.get("repo").and_then(|v| v.as_slice()),
                              "value `repo` should be an array");
        let mut ctx = Context::new();

        for repo in repos {
            let repo_table = try_toml!(repo.as_table(),
                                       "value `repo` should be a table");
            let r = try!(Repo::from_toml(repo_table));

            ctx.add_repo(r);
        }

        Ok(ctx)
    }
}

// }}}
// }}}

impl Context {
    pub fn from_config(cfgfile_path: &str) -> Result<Context, Error> {
        let mut f = fs::File::open(cfgfile_path).unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();

        let mut parser = toml::Parser::new(&buf);
        let table = try_toml!(parser.parse(),
                              format!("error while parsing `{}`: {:?}",
                                      &cfgfile_path, parser.errors));

        Context::from_toml(&table)
    }
}

// {{{ Tests

#[cfg(test)]
mod test {
    extern crate toml;
    use super::*;
    use super::FromToml;
    use core::Error;
    use git::{Repo, Context};

    use std::fmt::Debug;

    fn test_err_from_toml_string<T>(toml: &str, expected: &str)
        where T: FromToml + Debug
    {
        let table = toml::Parser::new(toml).parse().unwrap();
        let res = T::from_toml(&table);

        match res.unwrap_err() {
            Error::TomlError(e) => assert_eq!(e, expected),
            Error::StructuralError(e) => assert_eq!(e, expected),
        }
    }

    #[test]
    fn test_branch_from_toml() {
        let mut toml = String::from("");

        test_err_from_toml_string::<ParsedBranch>(&toml,
            "table `branch` should have a `name` attribute");

        toml.push_str("name = \"pnl\"\n");
        toml.push_str("inherits = 5\n");
        test_err_from_toml_string::<ParsedBranch>(&toml,
            "value `inherits` should be an array");

        toml = String::from("name = \"pnl\"\n");
        toml.push_str("inherits = [5]\n");
        test_err_from_toml_string::<ParsedBranch>(&toml,
            "`inherits` values should be strings");
    }

    #[test]
    fn test_repo_from_toml() {
        let mut toml = String::from("");

        test_err_from_toml_string::<Repo>(&toml,
            "table `repo` should have a `name` attribute");

        toml.push_str("name = \"cr\"\n");
        test_err_from_toml_string::<Repo>(&toml,
            "table `repo` should have an array `branch`");

        toml.push_str("branch = [1]\n");
        test_err_from_toml_string::<Repo>(&toml,
            "value `branch` should be a table");
    }

    #[test]
    fn test_ctx_from_toml() {
        test_err_from_toml_string::<Context>("",
            "value `repo` should be an array");

        let toml = "repo = 3";
        test_err_from_toml_string::<Context>(&toml,
            "value `repo` should be an array");

        let toml = "repo = [3]";
        test_err_from_toml_string::<Context>(&toml,
            "value `repo` should be a table");

        let toml = r#"
            [[repo]]
            name = "a"

            [[repo.branch]]
            name = "b"
            inherits = ["b"]
        "#;
        test_err_from_toml_string::<Context>(&toml,
            "branch `b` in repo `a` cannot inherit from itself");

        let toml = r#"
            [[repo]]
            name = "a"

            [[repo.branch]]
            name = "b"
            inherits = ["c"]
        "#;
        test_err_from_toml_string::<Context>(&toml,
            "unknown branch `c` in repo `a`");
    }
}

// }}}
