extern crate toml;

use std::collections::HashMap;
use std::fs;
use std::io::Read;

#[derive(Debug)]
pub enum Error {
    TomlError(String),
}

#[derive(Debug)]
pub struct Branch {
    pub name: String,
    pub inherits: Vec<Branch>,
}

#[derive(Debug)]
pub struct Repo {
    pub name: String,
    pub branches: HashMap<String, Branch>,
}

#[derive(Debug)]
pub struct Context {
    pub repos: HashMap<String, Repo>,
}

// {{{ TOML Parsing
// {{{ Helpers

macro_rules! try_toml {
    ($expr:expr, $err:expr) => (match $expr {
        Some(t) => t,
        None => return Err(Error::TomlError($err.to_string())),
    })
}

trait FromToml {
    fn from_toml(&toml::Table) -> Result<Self, Error>;
}

// }}}
// {{{ TOML tables to structures

impl FromToml for Branch {
    fn from_toml(table: &toml::Table) -> Result<Branch, Error> {
        let name = try_toml!(table.get("name").and_then(|v| v.as_str()),
                             "table `branch` should have a `name` attribute");

        Ok(Branch {
            name: name.to_string(),
            inherits: Vec::new(),
        })
    }
}

impl FromToml for Repo {
    fn from_toml(table: &toml::Table) -> Result<Repo, Error> {
        let name = try_toml!(table.get("name").and_then(|v| v.as_str()),
                             "table `repo` should have a `name` attribute");

        let branches = try_toml!(table.get("branch")
                                      .and_then(|v| v.as_slice()),
                                 "table `repo` should have an array \
                                  `branch`");
        let mut repo = Repo {
            name: name.to_string(),
            branches: HashMap::new(),
        };
        for branch in branches {
            let brc_table = try_toml!(branch.as_table(),
                                      "value `branch` should be a table");
            let b = try!(Branch::from_toml(brc_table));

            repo.branches.insert(b.name.to_string(), b);
        }
            
        Ok(repo)
    }
}

impl FromToml for Context {
    fn from_toml(table: &toml::Table) -> Result<Context, Error> {
        let repos = try_toml!(table.get("repo").and_then(|v| v.as_slice()),
                              "value `repo` should be an array");

        let mut ctx = Context {
            repos: HashMap::new()
        };

        for repo in repos {
            let repo_table = try_toml!(repo.as_table(),
                                       "value `repo` should be a table");
            let r = try!(Repo::from_toml(repo_table));

            ctx.repos.insert(r.name.to_string(), r);
        }

        Ok(ctx)
    }
}

// }}}

impl Context {
    pub fn new(cfgfile_path: &str) -> Result<Context, Error> {
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

// }}}
// {{{ Tests

#[cfg(test)]
mod test {
    extern crate toml;
    use super::*;
    use super::FromToml;
    use std::fmt::Debug;

    fn test_err_from_toml_string<T>(toml: &str, expected: &str)
        where T: FromToml + Debug
    {
        let table = toml::Parser::new(toml).parse().unwrap();
        let res = T::from_toml(&table);

        match res.unwrap_err() {
            Error::TomlError(e) => assert_eq!(e, expected),
        }
    }

    #[test]
    fn test_branch_from_toml() {
        test_err_from_toml_string::<Branch>("",
            "table `branch` should have a `name` attribute");
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
        let mut toml = String::from("");

        test_err_from_toml_string::<Context>(&toml,
            "value `repo` should be an array");

        toml.push_str("repo = 3");
        test_err_from_toml_string::<Context>(&toml,
            "value `repo` should be an array");

        toml = String::from("repo = [3]\n");
        test_err_from_toml_string::<Context>(&toml,
            "value `repo` should be a table");
    }
}

// }}}
