import numpy as np

from mnist.mnist import MNIST
from typing import Tuple


def ReLU(Z):
    return np.maximum(Z, 0)


def derivative_ReLU(Z):
    return Z > 0


def soft_max(Z):
    exp = np.exp(Z - np.max(Z))
    return exp / exp.sum(axis=0)


class NN:

    def __init__(self, layer_sizes: Tuple[int, int, int]):
        self.layer_sizes = layer_sizes
        self.input_layer_size = self.layer_sizes[0]
        self.output_layer_size = self.layer_sizes[-1]
        self.num_hidden_layer = len(self.layer_sizes) - 2

        self._initialize_parameters()

    def gradient_descent(self, X, Y, alpha, epochs, XT=None, YT=None):
        """Train neural network with gradient decent approach."""
        assert X.shape[0] == self.input_layer_size

        self.alpha = alpha
        self.num_samples = X.shape[1]
        self.oom = 1 / self.num_samples

        YY = self._reshape_label_to_layer_matrix(Y)

        for epoch in range(1, epochs + 1):
            self._forward_propagation(X)
            self._backward_propagation(X, YY)
            self._update_params()

            # Training progress output
            if epoch % 5 == 0:
                # Use test data if given to calc accuracy
                if XT is None or YT is None:
                    test_data = (self._A[-1], Y)
                else:
                    self._forward_propagation(XT)
                    test_data = (self._A[-1], YT)

                print(
                    f'\u001b[1000DEpochs: {epoch} / {epochs} ' +
                    f'- Acc: {self._get_accuracy(*test_data):6.2%}',
                    end=''
                )

        print('')

    def predict(self, X):
        self._forward_propagation(X)
        return np.argmax(self._A[-1], 0)

    def _initialize_parameters(self):
        """Create weight and bias matrices from input to output."""
        self._W, self._b = [], []

        for i in range(len(self.layer_sizes) - 1):
            self._W.append(
                np.random.rand(
                    self.layer_sizes[i+1], self.layer_sizes[i]
                ) - 0.5
            )
            self._b.append(
                np.random.rand(self.layer_sizes[i+1], 1) - 0.5
            )

    def _forward_propagation(self, X):
        self._A = [X]
        self._Z = []

        A = X
        for i in range(len(self.layer_sizes) - 1):
            Z = self._W[i].dot(A) + self._b[i]
            A = ReLU(Z) if i < (len(self.layer_sizes) - 2) else soft_max(Z)
            self._A.append(A)
            self._Z.append(Z)

    def _backward_propagation(self, X, Y):
        self._dW = []
        self._db = []

        for i in range(len(self.layer_sizes) - 1, 0, -1):
            if i == len(self.layer_sizes) - 1:
                dZ = 2 * (self._A[i] - Y)
            else:
                dZ = self._W[i].T.dot(dZ) * derivative_ReLU(self._Z[i-1])
            dW = self.oom * dZ.dot(self._A[i-1].T)
            db = self.oom * np.sum(dZ, 1)

            self._dW.append(dW)
            self._db.append(db)

        self._dW = self._dW[::-1]
        self._db = self._db[::-1]

    def _update_params(self):
        for i in range(1, len(self.layer_sizes)):
            self._W[i-1] -= self.alpha * self._dW[i-1]
            self._b[i-1] -= self.alpha * np.reshape(
                self._db[i-1], (self.layer_sizes[i], 1)
            )

    @staticmethod
    def _reshape_label_to_layer_matrix(Y):
        # NOTE: Only applicable for data with label 0 - 9!
        YY = np.zeros((Y.max() + 1, Y.size))
        YY[Y, np.arange(Y.size)] = 1
        return YY

    @staticmethod
    def _get_accuracy(A, Y):
        return np.sum(np.argmax(A, 0) == Y) / Y.size


if __name__ == "__main__":
    # Get MNIST training and test data
    mnist = MNIST()
    (X_train, Y_train) = mnist.get_normalize_data_set(60000, 'train')
    (X_test, Y_test) = mnist.get_normalize_data_set(10000, 'test')

    # Use gradient decent to train neural network
    nn = NN(layer_sizes=(X_train.shape[0], 128, 10))
    nn.gradient_descent(
        X_train, Y_train, alpha=0.25, epochs=250, XT=X_test, YT=Y_test
    )

    # Run predictions for both data sets
    print(
        'Accuracy with training data: ' +
        f'{sum(nn.predict(X_train) == Y_train) / Y_train.size:6.2%}'
    )
    print(
        'Accuracy with test data: ' +
        f'{sum(nn.predict(X_test) == Y_test) / Y_test.size:6.2%}'
    )
