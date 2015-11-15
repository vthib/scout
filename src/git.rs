use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub type BranchRef = Rc<RefCell<Branch>>;

// {{{ Branch

#[derive(Debug)]
pub struct Branch {
    name: String,
    inherits: Vec<BranchRef>,
}

impl Branch {
    pub fn new(name: String) -> Branch {
        Branch {
            name: name,
            inherits: Vec::new(),
        }
    }

    pub fn inherits_from(&mut self, child: &BranchRef) {
        self.inherits.push(child.clone());
    }
}

// }}}
// {{{ Repo

#[derive(Debug)]
pub struct Repo {
    name: String,
    branches: HashMap<String, BranchRef>,
}

impl Repo {
    pub fn new(name: String) -> Repo {
        Repo {
            name: name,
            branches: HashMap::new(),
        }
    }

    pub fn add_branch(&mut self, branch: Branch) {
        self.branches.insert(branch.name.to_string(),
                             Rc::new(RefCell::new(branch)));
    }

    pub fn find_branch(&self, branch_name: &str) -> Option<&BranchRef> {
        self.branches.get(branch_name)
    }
}

// }}}
// {{{ Context

#[derive(Debug)]
pub struct Context {
    repos: HashMap<String, Repo>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            repos: HashMap::new(),
        }
    }

    pub fn add_repo(&mut self, repo: Repo) {
        self.repos.insert(repo.name.to_string(), repo);
    }
}

// }}}
