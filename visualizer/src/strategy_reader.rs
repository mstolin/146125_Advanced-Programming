use serde::{Serialize, Deserialize};
use std::fs::File;
use std::fs;
use std::io::Write;


#[derive(Debug,Deserialize,Serialize)]
pub(crate) struct Balance{
    id :f64,
    possession : f64
}
impl Balance{
    pub(crate) fn get_id(&self) -> f64{
        self.id
    }
    pub(crate) fn get_possession(&self) -> f64{
        self.possession
    }
}

#[allow(dead_code)]
/// # Strategy saver
/// Serializes the activities in a location, with the desired name;
/// to be able to work for the visualizer it needs a log (vector) of activities in the form of a vector as indicated
///
pub fn save(activities : &[(i32,f64)],filename : &str){
    let balances: Vec<Balance>= activities.iter()
        .map(|op| Balance{id : op.0 as f64,possession : op.1})
        .collect();

    let json_ops = serde_json::to_string(&balances).unwrap();

    //need to change filepath after merge to main
    let mut file = File::create(format!("src/trades/{filename}.json")).expect("Could not save");
    file.write_all(json_ops.as_bytes()).unwrap();
}


/// # File reader
/// * `path` - The location of the strategy's log
pub(crate) fn read(filename :&str) -> Result<Vec<Balance>,serde_json::Error>{
    let path = format!("src/trades/{filename}");
    let file = File::open(path).expect("File not found in the folder src/trades");
    serde_json::from_reader(file)
}


pub(crate) fn find_all_available() -> Vec<String>{
    let directory = "src/trades";
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
