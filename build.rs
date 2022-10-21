use std::{fs, path::Path};

fn main() {
    let mut cf_template = fs::read_to_string("src/templates/launchTemplate.json")
        .expect("Something went wrong reading the CloudFormation template.");
    cf_template.pop();
    
    println!("{}", cf_template);


    let cf_template_const = format!("{}{}{}", r#"pub const LAUNCH_TEMPLATE: &'static str = r##""#, cf_template, r###""##;"###);
    let dest_path = Path::new("src").join("template.rs");

    fs::write(&dest_path, cf_template_const).unwrap();
    println!("cargo:rerun-if-changed=src/templates/launchTemplate.json");
    println!("cargo:rerun-if-changed=build.rs");
}
