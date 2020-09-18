use std::io::{BufRead, BufReader, Write};
use std::fs::{File, OpenOptions};

pub fn read_lines(filename : &String) -> std::io::Result<Vec<String>> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines().map(|l| l.unwrap()).collect())
}

pub fn append_lines(filename : &String, lines : &Vec<String>) -> std::io::Result<()> {
    let mut open_options = OpenOptions::new();
    open_options.append(true);
    let mut file = open_options.open(filename).or_else(|_| File::create(filename))?;
    for line in lines {
        let output = [line, "\n"].concat();
        file.write_all(output.as_bytes()).unwrap();
    }
    Ok(())
}

pub fn clear_file(filename : &String) -> std::io::Result<()> {
    OpenOptions::new().write(true).truncate(true).open(filename)?;
    Ok(())
}