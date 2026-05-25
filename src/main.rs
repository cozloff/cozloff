mod catalog; 
mod domain; 

fn main() {
    let earth = catalog::earth::earth_model();

    println!(
        "loaded Earth model: {} shells, {} shell specs",
        earth.shells.len(),
        earth.specs.len()
    );
}
