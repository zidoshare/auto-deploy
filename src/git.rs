use crate::config;
use git2;
use std::env;
use std::io::{self, Write};
use std::path;
use std::process::Command;
use std::str;

struct Git {
    cred: Option<git2::Cred>,
    signature: Option<git2::Signature<'static>>,
}

impl Git {
    pub fn new(config: &config::GitProps) -> Self {
        let cred: Option<git2::Cred> = if config.username.is_some() && config.password.is_some() {
            Some(
                git2::Cred::userpass_plaintext(
                    config.username.as_ref().unwrap(),
                    config.password.as_ref().unwrap(),
                )
                .unwrap(),
            )
        } else {
            None
        };
        Git {
            cred,
            signature: None,
        }
    }
    //need provide git path and local project path
    #[allow(dead_code)]
    pub fn pull_projects(
        &self,
        remote_git_path: &str,
        local_project_path: &std::path::Path,
        config: &config::DeployConfig,
    ) -> Result<(), git2::Error> {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_user: &str, _user_from_url: Option<&str>, _cred| {
            if _cred.contains(git2::CredentialType::USERNAME) {
                return git2::Cred::userpass_plaintext(
                    config
                        .git
                        .username
                        .as_ref()
                        .expect("the git server need provide username"),
                    config
                        .git
                        .password
                        .as_ref()
                        .expect("the git server need provide password"),
                );
            }

            let user = _user_from_url.unwrap_or("git");
            match env::var("AUTO_DEPLOY_SSH_KEY") {
                Ok(k) => {
                    println!(
                        "authenticate with user {} and private key located in {}",
                        user, k
                    );
                    git2::Cred::ssh_key(user, None, path::Path::new(&k), None)
                }
                _ => Err(git2::Error::from_str(
                    "unable to get private key from AUTO_DEPLOY_SSH_KEY",
                )),
            }
        });
        callbacks.sideband_progress(|data| {
            print!("remote:{}", str::from_utf8(data).unwrap());
            io::stdout().flush().unwrap();
            true
        });

        callbacks.update_tips(|refname, a, b| {
            if a.is_zero() {
                println!("[new]    {:20} {}", b, refname);
            } else {
                println!("[update] {:10}..{:10} {}", a, b, refname);
            }
            true
        });

        callbacks.transfer_progress(|stats| {
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

        if local_project_path.exists() {
            let repo = git2::Repository::open(local_project_path)?;
            let mut remote = repo
                .find_remote(&config.git.remote)
                .or_else(|_| (&repo).remote_anonymous(&config.git.remote))?;
            let mut fo = git2::FetchOptions::new();
            fo.remote_callbacks(callbacks);
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
            let mut opts = git2::FetchOptions::new();
            opts.remote_callbacks(callbacks);
            opts.download_tags(git2::AutotagOption::None);

            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(opts);
            builder.branch(&config.git.branch);

            builder.clone(remote_git_path, local_project_path)?;
        }
        Ok(())
    }
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
    use crate::config;
    use crate::git::Git;

    #[test]
    fn pull_projects_from_not_exists_project_should_return_false() {
        Git::new(&config::get_config("").git)
            .pull_projects(
                "git@github.com:/zidoshare/zicoder",
                std::path::Path::new("./test"),
                &config::get_config("example/example.toml"),
            )
            .unwrap();
    }
}
