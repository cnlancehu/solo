use std::path::PathBuf;

use definition::ConfigFile;
use lazy_static::lazy_static;
use reader::get_config_list;

pub mod definition;
pub mod reader;

lazy_static! {
    pub static ref CONFIG_DETECTION_PATH: PathBuf = {
        #[cfg(debug_assertions)]
        {
            PathBuf::from("./conf/")
        }
        #[cfg(not(debug_assertions))]
        {
            use std::env;

            let mut path = env::current_exe().unwrap();
            path.pop();
            path.push("conf");
            path
        }
    };
    pub static ref CONFIG_LIST: Vec<ConfigFile> = get_config_list();
    pub static ref CONFIG_LIST_NAMES: Vec<String> =
        CONFIG_LIST.iter().map(|f| f.name.clone()).collect();
}

pub fn get_config_path(name: &str) -> Option<PathBuf> {
    let file_name = CONFIG_LIST
        .iter()
        .find(|f| f.name == name)
        .map(|f| f.filename.clone());
    file_name.map(|f| {
        let mut path = CONFIG_DETECTION_PATH.clone();
        path.push(f);
        path
    })
}
