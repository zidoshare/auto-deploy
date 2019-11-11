use clap::crate_version;
use clap::{App, Arg};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

static ARG_CONFIG: &str = "config";
static ARG_LOCATION_PROJECTS: &str = "location-projects";
static ARG_LOCATION_BIN: &str = "location-bin";
static ARG_LOCATION_LOG: &str = "location-log";
static ARG_LOCATION_TMP: &str = "location-tmp";
static ARG_LOCATION_JAVA: &str = "location-java";
static ARG_GIT_REMOTE: &str = "git-remote";
static ARG_GIT_BRANCH: &str = "git-branch";
static ARG_GIT_PREFIX: &str = "git-prefix";
static ARG_GIT_NAME: &str = "git-name";
static ARG_GIT_EMAIL: &str = "git-password";
static ARG_GIT_USERNAME: &str = "git-username";
static ARG_GIT_PASSWORD: &str = "git-password";
static ARG_MAVEN_BIN: &str = "maven-bin";
static ARG_MAVEN_REPOSITORY: &str = "maven-repository";
static ARG_PACKAGE_ENV: &str = "package-env";
static ARG_PACKAGE_TARGET: &str = "package-target";
static ARG_DEPENDENCIES_UPDATE: &str = "dependencies-update";

static CONSTANTS_PROJECTS: &str = "PROJECTS";

#[derive(Debug, Deserialize)]
pub struct DeployConfig {
    pub location: LocationProps,
    pub git: GitProps,
    pub maven: MavenProps,
    pub package: PackageProps,
    pub dependencies: DependenciesProps,
    pub projects: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LocationProps {
    pub projects: String,
    pub bin: String,
    pub log: String,
    pub tmp: String,
    pub java: String,
}

#[derive(Debug, Deserialize)]
pub struct GitProps {
    pub remote: String,
    pub branch: String,
    pub prefix: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MavenProps {
    pub bin: String,
    pub repository: String,
}

#[derive(Debug, Deserialize)]
pub struct PackageProps {
    pub env: String,
    pub target: String,
}

#[derive(Debug, Deserialize)]
pub struct DependenciesProps {
    pub update: Vec<String>,
}

// 获取发布配置
//
// 配置规则：可通过全局配置和命令行参数进行配置，命令行参数优先。
// 全局配置会从 命令行参数 -c / 全局环境变量 / 默认配置地址 default_config_path 读取，优先级从高到低
pub fn get_config(default_config_path: &str) -> DeployConfig {
    let matches = get_app_args();
    let config_path = match matches.value_of(ARG_CONFIG) {
        Some(config_path) => String::from(config_path),
        None => env::var(ARG_CONFIG).unwrap_or(String::from(default_config_path)),
    };
    let global_config = get_config_from_toml(&config_path);

    DeployConfig {
        location: LocationProps {
            projects: (&matches)
                .value_of(ARG_LOCATION_PROJECTS)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.location.projects),
            bin: (&matches)
                .value_of(ARG_LOCATION_BIN)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.location.bin),
            log: (&matches)
                .value_of(ARG_LOCATION_LOG)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.location.log),
            tmp: (&matches)
                .value_of(ARG_LOCATION_TMP)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.location.tmp),
            java: (&matches)
                .value_of(ARG_LOCATION_JAVA)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.location.java),
        },
        git: GitProps {
            remote: (&matches)
                .value_of(ARG_GIT_REMOTE)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.git.remote),
            branch: (&matches)
                .value_of(ARG_GIT_BRANCH)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.git.branch),
            prefix: (&matches)
                .value_of(ARG_GIT_PREFIX)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.git.prefix),
            name: (&matches)
                .value_of(ARG_GIT_NAME)
                .and_then(|s| Some(String::from(s)))
                .or(global_config.git.username),
            email: (&matches)
                .value_of(ARG_GIT_EMAIL)
                .and_then(|s| Some(String::from(s)))
                .or(global_config.git.name),
            username: (&matches)
                .value_of(ARG_GIT_USERNAME)
                .and_then(|s| Some(String::from(s)))
                .or(global_config.git.email),
            password: (&matches)
                .value_of(ARG_GIT_PASSWORD)
                .and_then(|s| Some(String::from(s)))
                .or(global_config.git.password),
        },
        maven: MavenProps {
            bin: (&matches)
                .value_of(ARG_MAVEN_BIN)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.maven.bin),
            repository: (&matches)
                .value_of(ARG_MAVEN_REPOSITORY)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.maven.repository),
        },
        package: PackageProps {
            env: (&matches)
                .value_of(ARG_PACKAGE_ENV)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.package.env),
            target: (&matches)
                .value_of(ARG_PACKAGE_TARGET)
                .and_then(|s| Some(String::from(s)))
                .unwrap_or(global_config.package.target),
        },
        dependencies: DependenciesProps {
            update: if let Some(dependencies) = (&matches).values_of(ARG_DEPENDENCIES_UPDATE) {
                dependencies.map(|s| String::from(s)).collect()
            } else {
                global_config.dependencies.update
            },
        },
        projects: if let Some(projects) = (&matches).values_of(CONSTANTS_PROJECTS) {
            projects.map(|s| String::from(s)).collect()
        } else {
            panic!("need provide projects")
        },
    }
}

fn get_config_from_toml<'a, P: AsRef<Path>>(path: P) -> DeployConfig {
    if let Ok(contents) = fs::read_to_string(path.as_ref()) {
        toml::from_str::<DeployConfig>(&contents).unwrap()
    } else {
        panic!("配置文件错误:无法访问{}", path.as_ref().display())
    }
}

fn get_app_args<'a>() -> clap::ArgMatches<'a> {
    let absolute_path = "绝对路径";
    let relative_path = "相对路径";
    App::new("auto-deploy")
        .version(crate_version!())
        .author("zido. <wuhongxu1208@gmail.com>")
        .about("自动发布项目到服务器，包含从git拉取->校验并构建项目->备份原版本->发布/回滚新版本")
        .arg(Arg::with_name(ARG_CONFIG)
            .short("c")
            .long(ARG_CONFIG)
            .value_name("配置文件路径")
            .takes_value(true)
            .help("设置配置文件路径"))
        .arg(Arg::with_name(ARG_LOCATION_PROJECTS)
            .long(ARG_LOCATION_PROJECTS)
            .value_name(absolute_path)
            .help("设置项目所在目录")
            .takes_value(true))
        .arg(Arg::with_name(ARG_LOCATION_BIN)
            .long(ARG_LOCATION_BIN)
            .value_name(absolute_path)
            .takes_value(true)
            .help("项目可执行文件所在目录"))
        .arg(Arg::with_name(ARG_LOCATION_LOG)
            .long(ARG_LOCATION_LOG)
            .value_name(absolute_path)
            .takes_value(true)
            .help("日志文件所在目录"))
        .arg(Arg::with_name(ARG_LOCATION_TMP)
            .long(ARG_LOCATION_TMP)
            .value_name(absolute_path)
            .takes_value(true)
            .help("原可执行文件的备份文件所在路径"))
        .arg(Arg::with_name(ARG_LOCATION_JAVA)
            .long(ARG_LOCATION_JAVA)
            .value_name(absolute_path)
            .takes_value(true)
            .help("java bin执行路径"))
        .arg(Arg::with_name(ARG_GIT_REMOTE)
            .long(ARG_GIT_REMOTE)
            .value_name("仓库名")
            .takes_value(true)
            .help("git远程仓库名"))
        .arg(Arg::with_name(ARG_GIT_BRANCH)
            .long(ARG_GIT_BRANCH)
            .value_name("分支名")
            .help("设置git远程分支名"))
        .arg(Arg::with_name(ARG_GIT_PREFIX)
            .long(ARG_GIT_PREFIX)
            .value_name("url前缀")
            .help("设置git的url前缀，例如 git@github.com/github.com/xxx"))
        .arg(Arg::with_name(ARG_GIT_USERNAME)
            .long(ARG_GIT_USERNAME)
            .value_name("username")
            .help("Sets username for git"))
        .arg(Arg::with_name(ARG_GIT_PASSWORD)
            .long(ARG_GIT_PASSWORD)
            .value_name("password")
            .help("Sets password for git"))
        .arg(Arg::with_name(ARG_MAVEN_BIN)
            .long(ARG_MAVEN_BIN)
            .value_name(absolute_path)
            .help("maven可执行文件路径"))
        .arg(Arg::with_name(ARG_MAVEN_REPOSITORY)
            .long(ARG_MAVEN_REPOSITORY)
            .value_name(absolute_path)
            .help("maven仓库目录"))
        .arg(Arg::with_name(ARG_PACKAGE_ENV)
            .long(ARG_PACKAGE_ENV)
            .value_name("环境名")
            .help("当前执行环境名,对应spring.profiles.active"))
        .arg(Arg::with_name(ARG_PACKAGE_TARGET)
            .long(ARG_PACKAGE_TARGET)
            .value_name(relative_path)
            .help("项目/模块内构建结果目录"))
        .arg(Arg::with_name(ARG_DEPENDENCIES_UPDATE)
            .long(ARG_DEPENDENCIES_UPDATE)
            .value_name("依赖集合")
            .required(false)
            .help("项目所需要强制更新的依赖,采用gradle形式版本,\
            多个依赖使用逗号隔开,形如:\n site.zido:demo:-1.0.1,site.zido:demo2:0.0.2"))
        .arg(Arg::with_name(CONSTANTS_PROJECTS)
            .value_name("项目名")
            .required(true)
            .multiple(true)
            .help("设置发布项目名，支持多模块项目,多级项目情况下需要指定到具体模块名,\n形如: app/starter"))
        .get_matches()
}

#[cfg(test)]
mod test {
    use crate::config::*;
    use clap;

    #[test]
    #[should_panic(expected = "配置文件错误:无法访问./xxx")]
    fn get_config_from_toml_not_exists_should_panic() {
        get_config_from_toml("./xxx");
    }

    #[test]
    #[should_panic]
    fn get_config_from_toml_missing_some_option_should_not_works() {
        get_config("./example/missing_not_work.toml");
    }

    #[test]
    fn get_config_from_toml_missing_some_options_should_works() {
        let config = get_config("./example/missing_works.toml");
        assert_eq!("test", config.package.env);
    }

    #[test]
    fn clap_with_no_arg_name() {
        let m = App::new("myapp")
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .required(false)
                    .multiple(true)
                    .takes_value(true),
            )
            .get_matches_from(vec!["myapp", "-o", "val1", "val2"]);
        let mut values = m.values_of("output").unwrap();
        assert_eq!(values.next(), Some("val1"));
        assert_eq!(values.next(), Some("val2"));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn get_config_from_toml_works() {
        let config = get_config("./example/example.toml");
        assert_eq!("/home/zido/java/projects", config.location.projects);
        assert_eq!("/home/zido/java/bin", config.location.bin);
        assert_eq!("/home/zido/java/logs", config.location.log);
        assert_eq!("/home/zido/java/bin/.temps", config.location.tmp);
        assert_eq!("java", config.location.java);
        assert_eq!("origin", config.git.remote);
        assert_eq!("test", config.git.branch);
        assert_eq!("git@github.com/xxx", config.git.prefix);
        assert_eq!("wuhongxu1208@gmail.com", config.git.username.unwrap());
        assert_eq!("xxx", config.git.password.unwrap());
        assert_eq!("mvn", config.maven.bin);
        assert_eq!("/home/zido/.m2/repository", config.maven.repository);
        assert_eq!("test", config.package.env);
        assert_eq!("target", config.package.target);
        assert_eq!("site.zido:demo:0.0.1", config.dependencies.update[0]);
    }
}
