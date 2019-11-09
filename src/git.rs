use crate::config;
use git2;
use std::env;
use std::io::{self, Write};
use std::path;
use std::process::Command;
use std::str;

struct Git<'a> {
    //save user git cred
    cred: Option<git2::Cred>,
    signature: Option<git2::Signature<'static>>,
    config: &'a config::GitProps,
}

impl<'a> Git<'a> {
    pub fn new(config: &'a config::GitProps) -> Self {
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
            config,
        }
    }
    //need provide git project and local project path
    #[allow(dead_code)]
    pub fn pull_projects<'b> (
        &self,
        project: &'b str,
        local_project_path: &'b std::path::Path,
    ) -> Result<(), git2::Error> {
        let remote_git_path = format!("{}/{}.git", self.config.prefix, project);
        println!("remote git path:{}", remote_git_path);
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_user: &str, _user_from_url: Option<&str>, _cred| {
            if _cred.contains(git2::CredentialType::USERNAME) {
                return git2::Cred::userpass_plaintext(
                    self.config
                        .username
                        .as_ref()
                        .expect("the git server need provide username"),
                    self.config
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
                _ => {
                    let mut ssh_path = env::home_dir().unwrap();
                    ssh_path.push(".ssh/id_rsa");
                    git2::Cred::ssh_key(user, None, &ssh_path, None)
                    //     Err(git2::Error::from_str(
                    //     "unable to get private key from AUTO_DEPLOY_SSH_KEY and",
                    // ))
                }
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
                .find_remote(&self.config.remote)
                .or_else(|_| (&repo).remote_anonymous(&self.config.remote))?;
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
            builder.branch(&self.config.branch);

            builder.clone(&remote_git_path, local_project_path)?;
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
    #[ignore]
    fn pull_projects_from_not_exists_project_should_return_false() {
        Git::new(&config::GitProps {
            remote: String::from("origin"),
            branch: String::from("master"),
            prefix: String::from("git@github.com:zidoshare"),
            name: Some(String::from("zido")),
            email: Some(String::from("wuhongxu1208@gmail.com")),
            username: None,
            password: None,
        })
        .pull_projects("zicode-script.js", std::path::Path::new("./test"))
        .unwrap();
        std::fs::remove_dir(std::path::Path::new("./test")).unwrap();
    }
}
