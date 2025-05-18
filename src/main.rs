extern crate argparse;

use std::path::Path;

use argparse::{ArgumentParser, Store};
use nickgit::{GitRepository, repo_create};

fn main() {
    let mut command = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Basic implementation of Git using Rust!");
        ap.refer(&mut command)
            .add_argument("command", Store, "Command to run");
        ap.parse_args_or_exit();
    }

    if command == "init" {
        repo_create(Path::new("./")).unwrap();
    } else {
        // Print out worktree, gitdir and config of current git repo.
        match GitRepository::build(Path::new("./"), false) {
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
