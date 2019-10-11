use std::process::Command;

use crate::tools::path_exists;

//need provide git path and local project path
pub fn pull_projects(
    remote_git_path: &str,
    local_project_path: &str,
    remote: &str,
    branch: &str,
) -> bool {
    
    println!("pulling {}...", remote_git_path);
    if !path_exists(local_project_path) {
        println!("git project path exists:{}", local_project_path);
        if let Ok(mut child) = Command::new("git")
            .arg("clone")
            .arg(remote_git_path)
            .arg(local_project_path)
            .spawn()
        {
            if let Ok(status) = child.wait() {
                return status.success();
            }
        }
    } else {
        if let Ok(mut child) = Command::new("git").arg("fetch").arg("--all").spawn() {
            if let Ok(status) = child.wait() {
                if status.success() {
                    if let Ok(mut child) = Command::new("git")
                        .arg("reset")
                        .arg("--hard")
                        .arg(format!("{}/{}", remote, branch))
                        .spawn()
                    {
                        if let Ok(status) = child.wait() {
                            return status.success();
                        }
                    }
                }
            }
        }
    }
    false
}

// execute git config --global user.email
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
        let result = pull_projects(
            "https://github.com/xxx/xxfewrgfe",
            "./test",
            "origin",
            "master",
        );
        assert!(!result);
    }
}
