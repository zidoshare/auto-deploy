use java_properties;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Error, Reader, Writer};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Cursor};
use std::str;
use yaml_rust;
static INVALID_END_PATH_VEC: &[char] = &['/', '\\'];
//// validate project path,eg. application-${env}.properties
///  and set application.profiles to ${env}.
///
/// # Example:
/// ```rust
/// let project_path = "/data/some-server";
/// validate_project(project_path,"test");
/// ```
///
/// # with submodule starter
/// ```rust
/// let project_path = "/data/parent-module/sub-module";
/// validate_project(project_path,"test");
/// ```
pub fn validate_project<'a>(project_path: &'a str, env: &'a str) {
    //find deploy application.project,application-${env}.properties
    println!("validate project...");

    let project_path = project_path.trim_end_matches(INVALID_END_PATH_VEC);
    let mut project_name: Vec<char> = Vec::new();
    for c in project_path.chars().rev() {
        if c == '/' || c == '\\' {
            break;
        }
        project_name.push(c);
    }
    project_name.reverse();
    let project_name: String = project_name.into_iter().collect();
    let propertiesFile = File::open(format!(
        "{}/src/main/resources/application-{}.properties",
        project_path, env
    ))
    .unwrap_or_else(|_| {
        File::open(format!(
            "{}/src/main/resources/application-{}.yml",
            project_path, env
        ))
        .expect("the properties file: application.properties or application.yml is not exists")
    });
    fix_package_name(&format!("{}/pom.xml", project_path), &project_name);
}

fn fix_package_name<'a>(pom_file: &'a str, package_name: &'a str) {
    let content = fix_package_name_from_str(&fs::read_to_string(pom_file).unwrap(), package_name);
    fs::write(pom_file, content).unwrap();
}
fn fix_package_name_from_str<'a>(content: &'a str, package_name: &'a str) -> String {
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut need_write = false;
    let mut finded_final_name = false;
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(ref e) => {
                if let Event::Start(ref x) = e {
                    if x.name() == b"finalName" {
                        finded_final_name = true;
                        let final_name = &reader.read_text(x.name(), &mut Vec::new()).expect(
                            "cannot decode project final name in tag: <finalName>xxx</finalName>",
                        );
                        if final_name != package_name {
                            println!(
                                "this current finalName is {},and fix to {}",
                                final_name, package_name
                            );
                            need_write = true;
                            writer.write_event(e).unwrap();
                            writer
                                .write_event(Event::Text(BytesText::from_plain_str(package_name)))
                                .unwrap();
                            writer
                                .write_event(Event::End(BytesEnd::owned(x.name().to_vec())))
                                .unwrap();
                            continue;
                        } else {
                            println!("finalName is correct and does not need to be fixed");
                        }
                    }
                } else if let Event::End(ref x) = e {
                    if x.name() == b"build" && finded_final_name == false {
                        writer
                            .write_event(Event::Start(BytesStart::owned(
                                b"finalName".to_vec(),
                                "finalName".len(),
                            )))
                            .unwrap();
                        writer
                            .write_event(Event::Text(BytesText::from_plain_str(package_name)))
                            .unwrap();
                        writer
                            .write_event(Event::End(BytesEnd::owned(b"finalName".to_vec())))
                            .unwrap();
                    }
                }
                writer.write_event(e).unwrap();
            }
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
        buf.clear();
    }
    if need_write {
        String::from_utf8(writer.into_inner().into_inner()).unwrap()
    } else {
        content.to_owned()
    }
}

#[test]
fn when_final_name_not_match_then_fix_it() {
    let content = fs::read_to_string("./tests/pom.xml").unwrap();
    assert_eq!(557, content.find("demo-test").unwrap());
    let content = fix_package_name_from_str(&content, "test_1");
    assert_eq!(493, content.find("test_1").unwrap());
}
#[test]
fn when_final_name_not_exists_then_fix_it() {
    let content = fs::read_to_string("./tests/pom-with-no-final-name.xml").unwrap();
    let content = fix_package_name_from_str(&content, "test_1");
    println!("{}", content);
    assert_eq!(493, content.find("test_1").unwrap());
}

#[test]
fn when_final_name_matches_then_dont_fix_it() {
    let content = fs::read_to_string("./tests/pom.xml").unwrap();
    assert_eq!(557, content.find("demo-test").unwrap());
    let content = fix_package_name_from_str(&content, "demo-test");
    assert_eq!(557, content.find("demo-test").unwrap());
}
