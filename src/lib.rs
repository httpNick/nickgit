use configparser::ini::Ini;

use std::{
    fs, io,
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

fn create_repo_file(
    gitdir: &PathBuf,
    filename: &str,
    initial_content: &str,
) -> Result<PathBuf, io::Error> {
    let paths = &[filename];
    let filepath = repo_path(gitdir, paths);

    if let Some(parent) = filepath.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&filepath, initial_content)?;

    Ok(filepath)
}

fn default_config_content() -> Ini {
    let mut config = Ini::new();
    config.set("core", "repositoryformatversion", Some(String::from("0")));
    config.set("core", "filemode", Some(String::from("false")));
    config.set("core", "bare", Some(String::from("false")));

    config
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

    repo_dir(&repo.gitdir, &["branches"], true)?;
    repo_dir(&repo.gitdir, &["objects"], true)?;
    repo_dir(&repo.gitdir, &["refs", "tags"], true)?;
    repo_dir(&repo.gitdir, &["refs", "heads"], true)?;

    create_repo_file(
        &repo.gitdir,
        "description",
        "Unnamed repository; edit this file 'description' to name the repository.\n",
    )?;

    create_repo_file(&repo.gitdir, "HEAD", "ref: refs/heads/master\n")?;

    let config_file_path = create_repo_file(&repo.gitdir, "config", "")?;
    let config = default_config_content();
    config.write(config_file_path.to_str().unwrap())?;

    Ok(repo)
}

fn repo_find(path: &PathBuf, required: bool) -> Result<Option<GitRepository>, io::Error> {
    if path.join(".git").is_dir() {
        return match GitRepository::build(path, false) {
            Ok(repo) => Ok(Some(repo)),
            Err(e) => Err(e),
        };
    }

    let parent = path.join("..");

    if parent.eq(path) {
        if required {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No git directory."));
        } else {
            return Ok(None);
        }
    }

    repo_find(&parent, required)
}
