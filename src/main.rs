extern crate argparse;

use std::path::Path;

use argparse::{ArgumentParser, Store, StoreTrue};
use nickgit::{GitRepository, repo_create};

fn main() {
    let mut verbose = false;
    let mut name = "World".to_string();
    let mut command = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Greet somebody.");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");
        ap.refer(&mut name)
            .add_option(&["--name"], Store, "Name for the greeting");
        ap.refer(&mut command)
            .add_argument("command", Store, "Command to run");
        ap.parse_args_or_exit();
    }

    if verbose {
        println!("name is {}", name);
    }
    println!("Hello {}!", name);
    println!("Command {}!", command);
    if command == "init" {
        repo_create(Path::new("./")).unwrap();
    } else {
        let repo_path = Path::new("./");
        match GitRepository::build(&repo_path, false) {
            Ok(repo) => {
                println!("Worktree: {}", repo.worktree.display());
                println!("Gitdir: {}", repo.gitdir.display());
                println!("{:?}", repo.conf.get_map().unwrap())
            }
            Err(err) => {
                eprintln!("Error building repository: {}", err);
            }
        }
    }
}
