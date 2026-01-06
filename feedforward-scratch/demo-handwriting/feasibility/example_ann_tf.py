""" Simple implementation of an ANN to recognize handwritten digits.

Source: https://www.tensorflow.org/tutorials/quickstart/beginner
"""

import os
os.environ['TF_ENABLE_ONEDNN_OPTS'] = '0'
os.environ['TF_CPP_MIN_LOG_LEVEL'] = '1' 

import tensorflow as tf
# print("TensorFlow version:", tf.__version__)
import numpy as np


def export_weights_biases(model):
    for i in range(1, len(model.layers)):
        np.save(f"feasibility/weights_{i}", model.layers[i].get_weights()[0])
        np.save(f"feasibility/biases_{i}", model.layers[i].get_weights()[1])

        data = np.squeeze(model.layers[i].get_weights()[0])
        np.savetxt(f'feasibility/weights_{i}.txt', data, delimiter=",")
        data = np.squeeze(model.layers[i].get_weights()[1])
        np.savetxt(f'feasibility/biases_{i}.txt', data, delimiter=",")


def main(data: tuple, epochs: int = 10):
    """Train ANN and save model."""
    x_train, y_train, x_test, y_test = data

    model = tf.keras.models.Sequential([
        tf.keras.layers.Flatten(input_shape=(28, 28)),
        tf.keras.layers.Dense(256, activation='relu'),
        tf.keras.layers.Dense(128, activation='relu'),
        # tf.keras.layers.Dropout(0.2),
        tf.keras.layers.Dense(10)
    ])

    predictions = model(x_train[:1]).numpy()
    tf.nn.softmax(predictions).numpy()

    loss_fn = tf.keras.losses.SparseCategoricalCrossentropy(from_logits=True)
    loss_fn(y_train[:1], predictions).numpy()

    model.compile(optimizer='adam', loss=loss_fn, metrics=['accuracy'])
    model.fit(x_train, y_train, epochs=epochs)
    model.evaluate(x_test,  y_test, verbose=2)

    export_weights_biases(model)

    model.summary()

    # probability_model = tf.keras.Sequential([
    #   model,
    #   tf.keras.layers.Softmax()
    # ])
    # probability_model(x_test[:5])


if __name__ == "__main__":
    mnist = tf.keras.datasets.mnist

    (x_train, y_train), (x_test, y_test) = mnist.load_data()
    x_train, x_test = x_train / 255.0, x_test / 255.0

    main((x_train, y_train, x_test, y_test), epochs=10)
