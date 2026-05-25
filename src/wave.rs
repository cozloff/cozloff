use burn::tensor::backend::Backend;
use burn::tensor::{Int, Tensor};

pub fn generate_1d_wave_video<B: burn::tensor::backend::Backend>(
    device: &B::Device,
) -> Result<(), Box<dyn std::error::Error>> {
    let rec = rerun::RecordingStreamBuilder::new("wave").save("wave.rrd")?;

    for frame in 0..240 {
        let time = frame as f32 / 30.0;
        let y = generate_1d_wave::<B>(device, 1.0, 2.0, 1.0, 0.0, time)
            .into_data()
            .to_vec::<f32>()?;
        let points = y
            .into_iter()
            .enumerate()
            .map(|(i, y)| [i as f32 / 10.0, y])
            .collect::<Vec<_>>();

        rec.set_time_sequence("frame", frame);
        rec.log("wave", &rerun::LineStrips2D::new([points]))?;
    }

    println!("Wrote wave.rrd, press enter to continue...");
    Ok(())
}


// 1D wave: y(x,t) = A * sin(kx - wt + phi)
// A: amplitude
// k: wave number
// w: angular frequency
// phi: phase
pub fn generate_1d_wave<B: Backend>(
    device: &B::Device,
    amplitude: f32,
    wavelength: f32,
    period: f32,
    phase: f32,
    time: f32,
) -> Tensor<B, 1> {
    let wave_number: f32 = calc_wave_num(wavelength);
    let angular_frequency: f32 = calc_ang_freq(period);

    // Create 1D tensor for x: 0 -> 10
    let points = 100;
    let x: Tensor<B, 1> = Tensor::<B, 1, Int>::arange(0..points, device)
        .float()
        .mul_scalar(10.0 / points as f32);

    // Apply the wave formula to each point in x
    let wave = x
        .mul_scalar(wave_number)
        .add_scalar(-angular_frequency * time + phase)
        .sin()
        .mul_scalar(amplitude);

    wave
}

// Wave number k = 2 * pi / wavelength
//  - Spatial rate of phase change
//  - # of radians / unit distance
pub fn calc_wave_num(wavelength: f32) -> f32 {
    std::f32::consts::TAU / wavelength
}

// Angular frequency w = 2 * pi / period
//  - Temporal rate of phase change
//  - # of radians / unit time
pub fn calc_ang_freq(period: f32) -> f32 {
    std::f32::consts::TAU / period
}
