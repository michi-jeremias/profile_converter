use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use inquire::{Text, Select};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
struct Profiles {
    docName: String,
    meAddress: String,
    profiles: Vec<Profile>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Profile {
    name: String,
    parameters: Vec<Parameter>,
}

impl Profile {
    fn append_name(&mut self, s: &str) {
        self.name.push_str(s);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Parameter {
    providerNo: i8,
    shortName: String,
    edifactNo: u16,
}

impl Parameter {
    fn set_provider(&mut self, id: i8) {
        self.providerNo = id;
    }

    fn set_short_name(&mut self, new_name: &str) {
        self.shortName = String::from(new_name);
    }
}

fn main() {
    let dir_entry = fs::read_dir("./").unwrap();
    let files = dir_entry
        .filter_map(|entry|
            {
                entry.ok()
                .and_then(|e| e.path().file_name()
                    .and_then(|n| n.to_str().map(|s| String::from(s)))
                )
            })
        .collect::<Vec<String>>();

    let json_options: Vec<&String> = files.iter()
        .filter(|element| element.contains(".json")).collect();
    let translator_options: Vec<&String> = files.iter()
        .filter(|element| element.contains(".trans")).collect();
    let inquire_file_select = Select::new("Select profiles.json to translate:", json_options)
        .prompt();
    let inquire_translator_select = Select::new("Select target translation:", translator_options)
        .prompt();


    let parameter_map: HashMap<String, String>;
    match inquire_file_select {
        Ok(path) => parameter_map = load_parameter_map(path),
        Err(error) => panic!("Error: {error}"),
    };
    // = load_parameter_map(&inquire_translator_select.ok());
    // let path = String::from("C:/Users/michaeljeremias/Documents/GitHub/rust_doc/hashmap_test/src/assets/profiles_enml.json");
    // let json_string = fs::read_to_string(path).expect("could not read file");

    let json_string: String;
    match inquire_translator_select {
        Ok(path) => parameter_map = load_parameter_map(path),
        Err()
    let json_string = fs::read_to_string(inquire_file_select).expect("could not read file");
    // let mut profiles: Profiles = deserialize_profiles(&json_string);
    // update_profiles(&mut profiles, &parameter_map);
    // serialize_profiles(&profiles);
}

fn deserialize_profiles(json_string: &String) -> Profiles {
    let profiles: Profiles = serde_json::from_str(&json_string).expect("JSON was not properly formatted.");
    profiles
}

fn update_profiles(profiles: &mut Profiles, parameter_map: &HashMap<String, String>) {

    for profile in &mut profiles.profiles {

        for parameter in &mut profile.parameters {
            parameter.set_provider(13);
            let new_name = &parameter_map.get(&parameter.edifactNo.to_string());

            match new_name {
                None => println!("Edifact {:?}: no match found (profile: {:?})", &parameter.edifactNo, &profile.name),
                Some(t) => {
                    parameter.set_short_name(t);
                    // println!("{}", t)
                },
            };
        }
    };

}

fn serialize_profiles(profiles: &Profiles) {
    let outstring = serde_json::to_string(&profiles);
    match outstring {
        Ok(outstring) => {
            let path = "C:/Users/michaeljeremias/Documents/GitHub/rust_doc/hashmap_test/src/assets/profiles_new.json";
            match fs::write(path, outstring) {
                Ok(()) => (),
                Err(error) => panic!("Could not write file: {:?}", error),
            };
        },
        Err(error) => println!("Could not serialize: {:?}", error),
    }
}

fn load_parameter_map(path: &String) -> HashMap<String, String> {
    let mut path = String::from(path);
    // match lab {
    //     "enml" => {
    //         path.push_str("enml.txt");
    //         println!("enml")
    //     },
    //     "imcl" => {
    //         path.push_str("imcl.txt");
    //         println!("imcl")
    //     },
    //     _ => println!("other"),
    // }

    let mut hashmap: HashMap<String, String> = HashMap::new();
    let f = File::open(path).expect("Unable to open file.");
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let s = line
            .unwrap()
            .to_string();
        let tokens: Vec<&str> = s.split(",").collect();
        hashmap.insert(tokens[0].to_string(), tokens[1].to_string());
    }
    return hashmap;
}