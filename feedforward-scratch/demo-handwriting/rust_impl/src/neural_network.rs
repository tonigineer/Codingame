use std::collections::HashMap;
use std::fs::File;

use ndarray::{Array2, Axis};
use csv::ReaderBuilder;
use ndarray_csv::Array2Reader;
use ndarray_stats::QuantileExt;  // argmax trait!

#[derive(Debug)]
enum ActivationFunction {
    RELU,
    SOFTMAX,
}

// fn sigmoid_activation(z: Array2<f32>) -> Array2<f32> {
//     z.mapv(|x| sigmoid(&x))
// }

// fn sigmoid(z: &f32) -> f32 {
//     1.0 / (1.0 + std::f32::consts::E.powf(-z))
// }

fn relu_activation(z: Array2<f32>) -> Array2<f32> {
    z.mapv(|x| relu(&x))
}

fn relu(z: &f32) -> f32 {
    match *z > 0.0 {
        true => *z,
        false => 0.0,
    }
}

fn softmax_activation(x: Array2<f32>) -> Array2<f32> {
    let sum_exp = x.mapv(|i| i.exp()).sum();
    x.mapv(|i| i.exp() / sum_exp)
}

fn argmax(x: Array2<f32>) -> Array2<f32> {
    let arr: Vec<f32> = x.axis_iter(Axis(1))
        .map(|x| x.argmax().unwrap() as f32)
        .collect();
    Array2::<f32>::from_shape_vec((1, arr.len()), arr).unwrap()
}

#[derive(Debug)]
enum Layer {
    INPUT(usize),
    HIDDEN(usize, ActivationFunction),
    OUTPUT(usize, ActivationFunction)
}

pub struct NeuralNetwork{
    layers: Vec<Layer>,
    weights: HashMap<usize, Array2<f32>>,
    biases: HashMap<usize, Array2<f32>>
}

impl NeuralNetwork {
    fn read_from_tf(fullfile: String, shape: (usize, usize)) -> Array2<f32> {
        ReaderBuilder::new()
            .has_headers(false)
            .from_reader(File::open(fullfile).unwrap())
            .deserialize_array2(shape).unwrap()
    }

    pub fn init() -> NeuralNetwork {
        let mut layers: Vec<Layer> = Vec::new();
        layers.push(Layer::INPUT(784));
        layers.push(Layer::HIDDEN(256, ActivationFunction::RELU));
        layers.push(Layer::HIDDEN(128, ActivationFunction::RELU));
        layers.push( Layer::OUTPUT(10, ActivationFunction::SOFTMAX));

        let mut prev_dim: usize = 0;
        let mut weights: HashMap<usize, Array2<f32>> = HashMap::new();
        let mut biases: HashMap<usize, Array2<f32>> = HashMap::new();

        for (idx, layer) in layers.iter().enumerate() {
            match layer {
                Layer::INPUT(dim) => { prev_dim = *dim },
                Layer::HIDDEN(dim, _) |
                Layer::OUTPUT(dim, _) => {
                    weights.insert(
                        idx,
                        NeuralNetwork::read_from_tf(
                            format!("../feasibility/weights_{}.txt", idx),
                            (prev_dim, *dim)
                        )
                    );
                    prev_dim = *dim;

                    biases.insert(
                        idx,
                        NeuralNetwork::read_from_tf(
                            format!("../feasibility/biases_{}.txt", idx),
                            (*dim, 1)
                        )
                    );
                }
            }
        }

        NeuralNetwork {
            layers,
            weights,
            biases
        }
    }

    pub fn predict(&self, input: Array2<f32>) -> Array2<f32> {
        // Forward propagation
        let mut x: Array2<f32> = input;

        for (idx, layer) in self.layers.iter().enumerate() {
            match layer {
                Layer::INPUT(_) => { },
                Layer::HIDDEN(_, act_fcn) |
                Layer::OUTPUT(_, act_fcn) => {
                    let w = self.weights.get(&idx).unwrap();
                    let b = self.biases.get(&idx).unwrap(); 
                    let z = w.t().dot(&x) + b;

                    match act_fcn {
                        ActivationFunction::RELU => { x = relu_activation(z) },
                        ActivationFunction::SOFTMAX => { x = softmax_activation(z) },
                    }
                }
            }
        }

        return argmax(x);
    }
}
