use java_properties;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use yaml_rust;
//// validate project path,eg. application-${env}.properties
///  and set application.profiles to ${env}.
///
/// # Example:
/// ```rust
/// let project_path = "/data/some-server";
/// validateProject(project_path,"test");
/// ```
///
/// # with submodule starter
/// ```rust
/// let project_path = "/data/parent-module/sub-module";
/// validateProject(project_path,"test");
/// ```
pub fn validateProject(project_path: &str, env: &str) {
    //find deploy application.project,application-${env}.properties
    println!("validate project...");
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
    let pom = File::open(format!("{}/pom.xml", project_path))
        .expect("the pom file: pom.xml is not exists");
}

pub fn fixPackageName<'a>(pomFile: &'a File, packageName: &'a str) {
    let mut reader = Reader::from_reader(BufReader::new(pomFile));
    reader.trim_text(true);
    let mut vec: Vec<Event> = vec![];
    let stream = BufWriter::new(pomFile);
    let mut writer = Writer::new(stream);
    let mut buf = Vec::new();
    let mut needWrite = false;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(ref e) => {
                if let Event::Start(ref x) = e {
                    if x.name() == b"finalName" {
                        let finalName = &reader.read_text(x.name(), &mut Vec::new()).expect(
                            "cannot decode project final name in tag: <finalName>xxx</finalName>",
                        );
                        if finalName != packageName {
                            println!(
                                "this current finalName is {},and fix to {}",
                                finalName, packageName
                            );
                            needWrite = true;
                        } else {
                            println!("finalName is correct and does not need to be fixed");
                        }
                    }
                }
                vec.push(e.to_owned());
                buf.clear();
            }
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }
    if (needWrite) {}
}
