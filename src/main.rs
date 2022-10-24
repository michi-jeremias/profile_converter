#[warn(non_snake_case)]
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use inquire::{Select};

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
    fn set_provider(&mut self, id: &i8) {
        self.providerNo = *id;
    }

    fn set_short_name(&mut self, new_name: &str) {
        self.shortName = String::from(new_name);
    }
}

fn main() {
    let dir_entry = fs::read_dir("./").unwrap();
    // let files = dir_entry
    //     .filter_map(|entry|
    //         {
    //             entry.ok()
    //             .and_then(|e| e.path().file_name()
    //                 .and_then(|n| n.to_str().map(|s| String::from(s)))
    //             )
    //         })
    //     .collect::<Vec<String>>();

    let files: Vec<&str> = dir_entry
        .filter_map(|entry|
            {
                entry.ok()
                .and_then(|e| e.path().file_name()
                    .and_then(|n| n.to_str())
                )
            })
        .collect();


    let option_quit = "Quit";
    let mut json_options: Vec<&str> = files.into_iter()
        .filter(|element| element.contains(".json")).collect();
    json_options.push(&option_quit);

    // Select profile
    let json_string: String;
    let inquire_file_select = Select::new("Select profiles.json to translate:", json_options)
        .prompt();
    match inquire_file_select {
        Ok(selection) => {
            if selection.eq("Quit") {
                std::process::exit(0);
            } else {
                json_string = fs::read_to_string(selection).expect("could not read file");
            };
        },
        Err(error) => panic!("Error: {error}"),
    };

    // Select translator
    let mut translator_options: Vec<&str> = files.into_iter()
        .filter(|element| element.contains(".trans"))
        .collect();
    translator_options.push(&option_quit);

    let inquire_translator_select = Select::new("Select target translation:", translator_options)
        .prompt();

    let parameter_map: HashMap<String, String>;
    match inquire_translator_select {
        Ok(selection) => {
            if selection.eq("Quit") {
                std::process::exit(0);
            } else {
                parameter_map = load_parameter_map(selection);
            }
        },
        Err(error) => panic!("Error: {error}"),
    };

    // Select provider
    let mut provider_options: Vec<&str> = files.into_iter()
        .filter(|element| element.contains(".prov"))
        .collect();
    provider_options.push(&option_quit);
    let inquire_provider_select = Select::new("Select target provider: ", provider_options).prompt();
    let target_provider: Result<i8, ParseIntError>;
    match inquire_provider_select {
        Ok("Quit") => std::process::exit(0),
        Ok(choice) => target_provider = load_provider(choice),
        Err(error) => panic!("Error: {error}"),
    };

    let mut profiles: Profiles = deserialize_profiles(&json_string);
    update_profiles(&mut profiles, &parameter_map, &target_provider.unwrap());
    serialize_profiles(&profiles);
}

fn deserialize_profiles(json_string: &String) -> Profiles {
    let profiles: Profiles = serde_json::from_str(&json_string).expect("JSON was not properly formatted.");
    profiles
}

fn update_profiles(profiles: &mut Profiles, parameter_map: &HashMap<String, String>, provider: &i8) {

    for profile in &mut profiles.profiles {

        for parameter in &mut profile.parameters {
            parameter.set_provider(provider);
            let new_name = parameter_map.get(&parameter.edifactNo.to_string());

            match new_name {
                None => println!("Edifact {:?}: no match found (profile: {:?})", &parameter.edifactNo, &profile.name),
                Some(t) => {
                    parameter.set_short_name(t);
                },
            };
        }
    };

}

fn serialize_profiles(profiles: &Profiles) {
    let outstring = serde_json::to_string(&profiles);
    match outstring {
        Ok(outstring) => {
            let path = "./profiles_new.json";
            match fs::write(path, outstring) {
                Ok(()) => (),
                Err(error) => panic!("Could not write file: {:?}", error),
            };
        },
        Err(error) => println!("Could not serialize: {:?}", error),
    }
}

fn load_parameter_map(path: &str) -> HashMap<String, String> {
    let path = String::from(path);
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

fn load_provider(path: &str) -> Result<i8, ParseIntError > {
    let provider = std::fs::read_to_string(path).unwrap_or(String::from("0"));
    return parse_provider(&provider);
}

fn parse_provider(s: &String) -> Result<i8, ParseIntError> {
    s.parse::<i8>()
}