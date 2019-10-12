use git2;
use std::env;
use std::io::{self, Write};
use std::path;
use std::process::Command;
use std::str;
use url::Url;

fn git_credentials_callback(
    _user: &str,
    _user_from_url: Option<&str>,
    _cerd: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    let user = _user_from_url.unwrap_or("git");

    if _cerd.contains(git2::CredentialType::USERNAME) {
        return git2::Cred::username(user);
    }

    match env::var("GPM_SSH_KEY") {
        Ok(k) => {
            println!(
                "authenticate with user {} and private key located in {}",
                user, k
            );
            git2::Cred::ssh_key(user, None, path::Path::new(&k), None)
        }
        _ => Err(git2::Error::from_str(
            "unable to get private key from GPM_SSH_KEY",
        )),
    }
}

//need provide git path and local project path
#[allow(dead_code)]
pub fn pull_projects(
    remote_git_path: &str,
    local_project_path: &std::path::Path,
    remote: &str,
    branch: &str,
) -> Result<(), git2::Error> {
    println!("pulling {}...", remote_git_path);
    let data_url = match Url::parse(remote_git_path) {
        Ok(data_url) => data_url,
        Err(e) => panic!("failed to parse url:{}", e),
    };

    let path = local_project_path
        .join(data_url.host_str().unwrap())
        .join(&data_url.path()[1..]);

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(git_credentials_callback);

    let mut opts = git2::FetchOptions::new();
    opts.remote_callbacks(callbacks);
    opts.download_tags(git2::AutotagOption::None);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(opts);
    builder.branch(branch);

    if path.exists() {
        let path_str = &path.to_str().unwrap();
        println!("git project path exists:{}", path_str);
        let repo = git2::Repository::open(&path)?;
        println!("fetching {} fro repo:{}", remote, path_str);
        let mut cb = git2::RemoteCallbacks::new();
        let mut remote = repo
            .find_remote(remote)
            .or_else(|_| (&repo).remote_anonymous(remote))?;

        cb.sideband_progress(|data| {
            print!("remote:{}", str::from_utf8(data).unwrap());
            io::stdout().flush().unwrap();
            true
        });

        cb.update_tips(|refname, a, b| {
            if a.is_zero() {
                println!("[new]    {:20} {}", b, refname);
            } else {
                println!("[update] {:10}..{:10} {}", a, b, refname);
            }
            true
        });

        cb.transfer_progress(|stats| {
            if stats.received_objects() == stats.total_objects() {
                print!(
                    "Resolving deltas {}/{}\r",
                    stats.indexed_deltas(),
                    stats.total_deltas()
                );
            } else if stats.total_objects() > 0 {
                print!(
                    "Received {}/{} objects ({}) in {} bytes\r",
                    stats.received_objects(),
                    stats.total_objects(),
                    stats.indexed_objects(),
                    stats.received_bytes()
                );
            }
            io::stdout().flush().unwrap();
            true
        });
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(cb);
        remote.download(&[], Some(&mut fo))?;
        let stats = remote.stats();

        if stats.local_objects() > 0 {
            println!(
                "\rReceived {}/{} objects in {} bytes (used {} local \

                                                                     objects)",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes(),
                stats.local_objects()
            );
        } else {
            println!(
                "\rReceived {}/{} objects in {} bytes",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes()
            );
        }
        remote.disconnect();
        remote.update_tips(None, true, git2::AutotagOption::Unspecified, None)?;
    } else {
        builder.clone(remote, &path)?;
    }
    Ok(())
}

// execute git config --global user.email
#[allow(dead_code)]
pub fn assert_name(name: &str) {
    let output = Command::new("git")
        .arg("config")
        .arg("--global")
        .arg("--get")
        .arg("user.name")
        .output()
        .expect(
            "fail to get user.name from git, the command: \"git config --global --get user.name\"",
        );
    if output.status.success() && name.as_bytes() != output.stdout.as_slice() {
        println!("set git user.name to {}", name);
        Command::new("git")
            .arg("config")
            .arg("--global")
            .arg("user.name")
            .arg(name)
            .spawn()
            .expect(&format!("fail to set user.name to {}", name));
    }
}

// execute git config --global user.email
#[allow(dead_code)]
pub fn assert_email(email: &str) {
    let output = Command::new("git")
        .arg("config")
        .arg("--global")
        .arg("--get")
        .arg("user.email")
        .output()
        .expect(
            "fail to get user.email from git, the command: \"git config --global --get user.email\"",
        );
    if output.status.success() && (email.as_bytes() != output.stdout.as_slice()) {
        println!("set git user.email to {}", email);
        Command::new("git")
            .arg("config")
            .arg("--global")
            .arg("user.email")
            .arg(email)
            .spawn()
            .expect(&format!("fail to set user.email to {}", email));
    }
}

#[cfg(test)]
mod test {
    use crate::git::pull_projects;

    #[test]
    fn pull_projects_from_not_exists_project_should_return_false() {
        pull_projects(
            "https://github.com/xxx/xxfewrgfe",
            std::path::Path::new("./test"),
            "origin",
            "master",
        )
        .unwrap();
    }
}
