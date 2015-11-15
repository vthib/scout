extern crate scout;

use scout::git::Context;

use std::io::Write;

macro_rules! exit_on_err(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        };
        return 1;
    )
);

fn _main() -> i32 {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        exit_on_err!("missing config file");
    }
    let ctx = match Context::from_config(&args[1]) {
        Ok(ctx) => ctx,
        Err(e) => match e {
            scout::Error::TomlError(err) => {
                exit_on_err!("error while parsing `{}`: {}", &args[1], err);
            }
            scout::Error::StructuralError(err) => {
                exit_on_err!("`{}` is illformed: {}", &args[1], err);
            }
        }
    };

    println!("context: {:?}", ctx);
    0
}

fn main() {
    let exit_code = _main();

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}
