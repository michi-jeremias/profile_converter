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
    edifactNo: Option<u16>,
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
    let files: Vec<String> = get_files_in_dir("./");

    // Select profile
    let mut files_json: Vec<&str> = files.iter()
        .filter(|s| s.contains(".json"))
        .map(|s| &**s)
        .collect();
    files_json.push("Quit");

    let json_string: String;
    let inquire_file_select = Select::new("Select profiles.json to translate:", files_json)
        .prompt();
    match inquire_file_select {
        Ok("Quit") => std::process::exit(0),
        Ok(selection) => json_string = fs::read_to_string(selection).expect("could not read file"),
        Err(error) => panic!("Error: {}", error),
    };

    // Select translator
    let mut files_trans: Vec<&str> = files.iter()
        .filter(|s| s.contains(".trans"))
        .map(|s| &**s)
        .collect();
    files_trans.push("Quit");
    let inquire_translator_select = Select::new("Select source translation:", files_trans)
        .prompt();

    let source_map_edi_name: HashMap<String, String>;
    let source_map_name_edi: HashMap<String, String>;
    match inquire_translator_select {
        Ok("Quit") => std::process::exit(0),
        Ok(selection) => (source_map_edi_name, source_map_name_edi) = load_parameter_maps(selection),
        Err(error) => panic!("Error: {}", error),
    };

    let mut files_trans: Vec<&str> = files.iter()
        .filter(|s| s.contains(".trans"))
        .map(|s| &**s)
        .collect();
    files_trans.push("Quit");
    let inquire_translator_select = Select::new("Select target translation:", files_trans)
        .prompt();

    let target_map_edi_name: HashMap<String, String>;
    let target_map_name_edi: HashMap<String, String>;
    match inquire_translator_select {
        Ok("Quit") => std::process::exit(0),
        Ok(selection) => (target_map_edi_name, target_map_name_edi) = load_parameter_maps(selection),
        Err(error) => panic!("Error: {}", error),
    };

    // Select provider
    let mut files_prov: Vec<&str> = files.iter()
        .filter(|s| s.contains(".prov"))
        .map(|s| &**s)
        .collect();
    files_prov.push("Quit");
    let inquire_provider_select = Select::new("Select target provider: ", files_prov)
        .prompt();
    let target_provider: Result<i8, ParseIntError>;
    match inquire_provider_select {
        Ok("Quit") => std::process::exit(0),
        Ok(choice) => target_provider = load_provider(choice),
        Err(error) => panic!("Error: {}", error),
    };


    match deserialize_profiles(&json_string) {
        Ok(mut profiles) => {
            update_profiles(&mut profiles, &source_map_name_edi, &target_map_edi_name, &target_provider.unwrap());
            serialize_profiles(&profiles);
        },
        Err(error) => println!("Error: {}", error),
    };
}


fn get_files_in_dir(path: &str) -> Vec<String> {
    let paths = fs::read_dir(path)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect();
    return paths;
}

fn deserialize_profiles(json_string: &String) -> Result<Profiles, serde_json::Error> {
    serde_json::from_str(json_string)
}

fn update_profiles(profiles: &mut Profiles, source_map: &HashMap<String, String>, target_map: &HashMap<String, String>, provider: &i8) {

    for profile in &mut profiles.profiles {

        for parameter in &mut profile.parameters {
            parameter.set_provider(provider);
            let param_edi = source_map.get(&parameter.shortName);
            let mut found_edi: String = String::from("-1");
            match param_edi {
                None => println!("Edifact {:?}, parameter: {:?} not found in source translation", param_edi, parameter.shortName),
                Some(edi) => found_edi = edi.to_string(),
            };

            let new_name = target_map.get(&found_edi);

            match new_name {
                None => println!("Edifact: {:?}, name: {:?}: no match found (profile: {:?})", &found_edi, &parameter.shortName, &profile.name),
                Some(name) => {
                    parameter.set_short_name(name);
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

fn load_parameter_maps(path: &str) -> (HashMap<String, String>, HashMap<String, String>) {
    let path = String::from(path);
    let mut hashmap_edi_name: HashMap<String, String> = HashMap::new();
    let mut hashmap_name_edi: HashMap<String, String> = HashMap::new();
    let f = File::open(path).expect("Unable to open file.");
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let s = line
            .unwrap()
            .to_string();
        let tokens: Vec<&str> = s.split(",").collect();
        hashmap_edi_name.insert(tokens[0].to_string(), tokens[1].to_string());
        hashmap_name_edi.insert(tokens[1].to_string(), tokens[0].to_string());
    }
    return (hashmap_edi_name, hashmap_name_edi);
}

fn load_provider(path: &str) -> Result<i8, ParseIntError > {
    let provider = std::fs::read_to_string(path).unwrap_or(String::from("0"));
    return parse_provider(&provider);
}

fn parse_provider(s: &String) -> Result<i8, ParseIntError> {
    s.parse::<i8>()
}