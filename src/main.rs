use std::{env, path::Path};

mod pull;
mod server;

fn main() {
    let args: Vec<String> = env::args().collect();
    let result = pull::execute_pull_request(Path::new(args.get(1).unwrap()));

    match result {
        Ok(r) => println!("{:?}", r),
        Err(e) => println!("{:?}", e.to_string()),
    }
}
