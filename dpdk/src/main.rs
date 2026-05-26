mod runtime;

use runtime::DpdkRuntime;

fn main() {
    let eal_args = std::env::args().collect::<Vec<_>>();

    match DpdkRuntime::init(&eal_args) {
        Ok(runtime) => {
            println!("DPDK initialized");
            println!("available Ethernet devices: {}", runtime.port_count());
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
