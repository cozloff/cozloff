use burn::backend::Cuda;

mod wave;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut x: i16 = 0;
    loop {
        match x {
            0 => println!("Press Enter for matmul..."),
            1 => {
                let device = Default::default();
                wave::generate_1d_wave_video::<Cuda>(&device)?;
            }
            2 => {
                println!("Oh look you did it! Here is some dopamine.");
                println!("Press enter to hyperfix down the rabbit hole through autistic space...");
            }
            _ => (),
        }
        x += 1;

        std::io::stdin().read_line(&mut String::new())?;

        if x > 2 {
            println!("Goodbye!");
            break;
        }
    }

    Ok(())
}