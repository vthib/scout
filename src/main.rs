extern crate scout;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        panic!("missing config file");
    }
    let ctx = match scout::Context::new(&args[1]) {
        Ok(ctx) => ctx,
        Err(e) => match e {
            scout::Error::TomlError(err) => {
                panic!("error while parsing `{}`: {}", &args[1], err);
            }
        }
    };

    for repo in ctx.repos.values() {
        println!("repo name: {}", repo.name);
        for branch in repo.branches.values() {
            println!("  branch name: {}", branch.name);
        }
    }
}
