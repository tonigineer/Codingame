use mnist::*;
use ndarray::prelude::*;

mod neural_network;
use neural_network::NeuralNetwork;

#[warn(unused_variables)]
fn get_test_data(sample_size: usize) -> (Array2<f32>, Array2<f32>) {
    // https://docs.rs/mnist/latest/mnist/
    let samples = sample_size.min(10_000);

    let Mnist {
        trn_img,
        trn_lbl,
        tst_img,
        tst_lbl,
        ..
    } = MnistBuilder::new()
        .label_format_digit()
        .training_set_length(50_000)
        .validation_set_length(samples as u32)
        .test_set_length(samples as u32)
        .finalize();

    trn_img.len();
    trn_lbl.len();

    let test_data = Array3::from_shape_vec((samples, 784, 1), tst_img)
        .expect("Error converting images to Array3 struct")
        .map(|x| *x as f32 / 256.)
        .into_shape((samples, 784)).unwrap().t().to_owned();
 
    let test_labels: Array2<f32> = Array2::from_shape_vec((samples, 1), tst_lbl)
        .expect("Error converting testing labels to Array2 struct")
        .map(|x| *x as f32);

    (test_data, test_labels)
}


fn calculate_accuracy(predictions: Array2<f32>, labels: Array2<f32>) -> f32 {
    let correct = predictions
        .iter()
        .zip(labels.iter())
        .fold(0, |acc, x| if x.0 == x.1 { acc + 1 } else { acc });
    return correct as f32 / labels.len() as f32;
}

fn main() {
    let neural_network: NeuralNetwork = NeuralNetwork::init();

    let (test_data, labels) = get_test_data(10_000);
    let predictions = neural_network.predict(test_data);

    println!("Accuracy: {}%", calculate_accuracy(predictions, labels) * 100 as f32);
}
