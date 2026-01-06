from example_ann_scratch import NN

import numpy as np
import tensorflow as tf
# print("TensorFlow version:", tf.__version__)


def main():
    # Create ANN and set parameters from the tf-trained network
    w1 = np.load("feasibility/weights_1.npy")
    w2 = np.load("feasibility/weights_2.npy")
    b1 = np.load("feasibility/biases_1.npy")
    b2 = np.load("feasibility/biases_2.npy")

    print(f"w1: {w1.shape} b1: {b1.shape} w2: {w2.shape} b2: {b2.shape}")

    nn = NN(layer_sizes=(w1.shape[0], w2.shape[0], w2.shape[1]))

    nn._W[0] = w1.T
    nn._b[0] = b1.reshape(128,1)
    nn._W[1] = w2.T
    nn._b[1] = b2.reshape(10,1)

    # Use the same test data (MNIST from the keras)
    mnist = tf.keras.datasets.mnist
    (x_train, _), (x_test, y_test) = mnist.load_data()
    x_train, x_test = x_train / 255.0, x_test / 255.0
    x_test = x_test.reshape(10000, 784).T

    # x_test = x_test[:, 0:2]
    # y_test = y_test[0:2]
    print(nn.predict(x_test))
    print(
        'Accuracy with test data: ' +
        f'{sum(nn.predict(x_test) == y_test) / y_test.size:6.2%}'
    )

    # # Use own implementation
    # from mnist.mnist import MNIST
    # mnist = MNIST()
    # (x_train, _) = mnist.get_normalize_data_set(60000, 'train')
    # (x_test, y_test) = mnist.get_normalize_data_set(10000, 'test')

    # print(
    #     'Accuracy with test data: ' +
    #     f'{sum(nn.predict(x_test) == y_test) / y_test.size:6.2%}'
    # )

if __name__ == "__main__":
    main()
