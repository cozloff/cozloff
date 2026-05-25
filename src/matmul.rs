use burn::prelude::{Backend, Tensor};

pub fn matmul<B: Backend>(device: &B::Device) -> () {
    // Two 2D tensors (2x3 and 3x2)
    let a = Tensor::<B, 2>::from_floats(
        [[1.0, 2.0, 3.0], 
        [4.0, 5.0, 6.0]],
        device,
    );
    
    let b = Tensor::<B, 2>::from_floats(
        [[7.0, 8.0], 
        [9.0, 10.0], 
        [11.0, 12.0]],
        device,
    );

    // Perform matrix multiplication
    let c = a.matmul(b);

    // Print the result
    println!("{:?}", c);
}
