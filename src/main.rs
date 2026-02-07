use std::io;

mod lib;

use lib::tui; 

fn main() -> io::Result<()> {
    let data = tui::run_app()?;
    
    print!("{}", data);
    
    Ok(())
}
