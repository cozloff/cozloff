use burn::backend::Cuda;
use burn::backend::cuda::CudaDevice;

mod matmul;

fn main() {
    let mut x: i16 = 0;
    loop {
        match x {
            0 => println!("Press Enter for matmul..."),
            1 => {
                let device: CudaDevice = Default::default();
                matmul::matmul::<Cuda>(&device);
            }
            2 => {
                println!("Oh look you did it! Here is some dopamine.");
                println!("Press enter to hyperfix down the rabbit hole through autistic space...");
            },
            _ => (),
        }
        x+=1;    

        std::io::stdin().read_line(&mut String::new()).unwrap();
        
        if x > 2 {
                println!("Goodbye!");
                break;
        }
    }
}
