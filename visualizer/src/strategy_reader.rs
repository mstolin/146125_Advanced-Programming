use serde::{Serialize, Deserialize};
use std::fs::File;
use std::fs;
use std::io::Write;


#[derive(Debug,Deserialize,Serialize)]
pub(crate) struct Balance{
    pub day:f64,
    pub eur:f64,
    pub usd:f64,
    pub yen:f64,
    pub yuan:f64,
}
impl Balance{
    pub(crate) fn get_day(&self) -> f64{
        self.day
    }
    pub(crate) fn get_eur(&self) -> f64{
        self.eur
    }
    pub(crate) fn get_usd(&self) -> f64{
        self.usd
    }
    pub(crate) fn get_yen(&self) -> f64{
        self.yen
    }
    pub(crate) fn get_yuan(&self) -> f64{
        self.yuan
    }
}

#[allow(dead_code)]
/// # Strategy saver
/// Serializes the activities in a location, with the desired name;
/// to be able to work for the visualizer it needs a log (vector) of activities in the form of a vector as indicated
///
pub fn save(activities : &[(i32,f64,f64,f64,f64)],filename : &str){
    let balances: Vec<Balance>= activities.iter()
        .map(|op| Balance{
            day : op.0 as f64,
            eur : op.1,
            usd: op.2,
            yen : op.3,
            yuan : op.4
        })
        .collect();

    let json_ops = serde_json::to_string(&balances).unwrap();
    println!("json was created");

    let mut file = File::create(format!("visualizer/src/trades/{filename}.json")).expect("Could not save");
    file.write_all(json_ops.as_bytes()).unwrap();
}


/// # File reader
/// * `path` - The location of the strategy's log
pub(crate) fn read(filename :&str) -> Result<Vec<Balance>,serde_json::Error>{
    //test only works with src/trades while the program works with visualizer/src/trades ?? let's check
    let path = format!("visualizer/src/trades/{filename}");
    let file = File::open(path).expect("File not found in the folder src/trades");
    serde_json::from_reader(file)
}


pub(crate) fn find_all_available() -> Vec<String>{
    let directory = "C:/Users/Farid/CLionProject/trade-agent/visualizer/src/trades";
    let mut strategies = vec![];
    let saved_files = fs::read_dir(directory).expect("Nothing was found");
    for file in saved_files{
        let curr_file = file.unwrap();
        if curr_file.file_type().unwrap().is_file(){
            strategies.push(curr_file.file_name().into_string().unwrap());
        }
    }
    strategies
}
