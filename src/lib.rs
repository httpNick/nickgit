use configparser::ini::Ini;

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub struct GitRepository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
    pub conf: Ini,
}

impl GitRepository {
    pub fn build(path: &Path, force: bool) -> Result<GitRepository, io::Error> {
        let worktree = PathBuf::from(path);
        let gitdir = worktree.join(".git");

        if !(force || gitdir.is_dir()) {
            let formatted_err = format!("Not a Git Repository: {}", path.display());
            return Err(io::Error::new(io::ErrorKind::Other, formatted_err));
        }

        let mut conf = Ini::new();
        if let Some(repo_config) = repo_file(&gitdir, &["config"], false)? {
            conf.load(repo_config).unwrap();
        }
        Ok(GitRepository {
            worktree,
            gitdir,
            conf,
        })
    }
}

fn repo_path(gitdir: &PathBuf, paths: &[&str]) -> PathBuf {
    let mut final_path = PathBuf::from(gitdir);
    for path_seg in paths {
        final_path = final_path.join(path_seg);
    }
    final_path
}

fn repo_dir(gitdir: &PathBuf, paths: &[&str], mkdir: bool) -> Result<Option<PathBuf>, io::Error> {
    let path = repo_path(gitdir, paths);

    if path.exists() {
        if path.is_dir() {
            Ok(Some(path))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Not a directory: {}", path.display()),
            ))
        }
    } else if mkdir {
        fs::create_dir_all(&path)?;
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

fn repo_file(gitdir: &PathBuf, paths: &[&str], mkdir: bool) -> Result<Option<PathBuf>, io::Error> {
    if repo_dir(gitdir, &paths[..paths.len() - 1], mkdir)?.is_some() {
        Ok(Some(repo_path(gitdir, paths)))
    } else {
        Ok(None)
    }
}

pub fn repo_create(path: &Path) -> Result<GitRepository, io::Error> {
    let repo = GitRepository::build(path, true).unwrap();

    if repo.worktree.exists() {
        if !repo.worktree.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Not a directory: {}", path.display()),
            ));
        }
        if repo.gitdir.exists() && fs::read_dir(&repo.gitdir).unwrap().count() > 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Directory not empty: {}", path.display()),
            ));
        }
    } else {
        fs::create_dir(&repo.worktree).unwrap();
    }

    repo_dir(&repo.gitdir, &["branches"], true).unwrap();
    repo_dir(&repo.gitdir, &["objects"], true).unwrap();
    repo_dir(&repo.gitdir, &["refs", "tags"], true).unwrap();
    repo_dir(&repo.gitdir, &["refs", "heads"], true).unwrap();

    let mut head_file =
        fs::File::create(repo_file(&repo.gitdir, &["HEAD"], false).unwrap().unwrap())?;
    head_file.write_all(b"ref: refs/heads/master\n")?;

    Ok(repo)
}
