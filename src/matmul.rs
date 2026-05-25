use burn::tensor::Tensor;
use burn::tensor::backend::Backend;

pub fn matmul<B: Backend>(device: &B::Device) -> () {
    // Two 2D tensors (2x3 and 3x2)
    let a =
        Tensor::<B, 2>::from_floats([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [4.0, 5.0, 6.0]], device);

    let b = Tensor::<B, 2>::from_floats([[7.0, 8.0], [9.0, 10.0], [11.0, 12.0]], device);

    // Perform matrix multiplication
    let c = a.matmul(b);

    // Print the result
    print_matrix(c);
}

fn print_matrix<B: Backend>(matrix: Tensor<B, 2>) {
    let data = matrix.into_data();
    let dims = data.shape.as_slice().to_vec();
    let values = data.into_vec::<f32>().unwrap();
    let rows = dims[0];
    let cols = dims[1];

    println!("Result:");
    for row in values.chunks(cols).take(rows) {
        print!("[");
        for (index, value) in row.iter().enumerate() {
            if index > 0 {
                print!(" ");
            }
            print!("{value:>8.2}");
        }
        println!(" ]");
    }
}
