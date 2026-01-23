use clap_mangen::Man;
use clap::CommandFactory;
use tatl::cli::commands::Cli;
use std::fs;
use std::path::PathBuf;

fn main() {
    let app = Cli::command();
    let mut buffer: Vec<u8> = Default::default();
    Man::new(app).render(&mut buffer).unwrap();
    
    // Get project root
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let man_dir = project_root.join("man").join("man1");
    
    // Create directory if it doesn't exist
    fs::create_dir_all(&man_dir).expect("Failed to create man directory");
    
    let man_path = man_dir.join("tatl.1");
    fs::write(&man_path, buffer).expect("Failed to write man page");
    
    println!("Man page generated at: {}", man_path.display());
    println!("To view: man -l {}", man_path.display());
    println!("To install: sudo cp {} /usr/local/share/man/man1/ && sudo mandb", man_path.display());
}
