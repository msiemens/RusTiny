use std::fs::File;
use std::io::Read;
use std::path::Path;


/// Read a file and return it's contents
pub fn read_file(input_path: &str) -> String {
    let mut file = match File::open(&Path::new(input_path)) {
        Ok(f) => f,
        Err(err) => panic!("Can't open {}: {}", input_path, err)
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(contents) => contents,
        Err(_) => panic!("Can't read {}", input_path)
    };

    contents
}
