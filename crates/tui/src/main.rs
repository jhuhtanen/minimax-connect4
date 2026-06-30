use std::error::Error;

mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    ui::run()
}