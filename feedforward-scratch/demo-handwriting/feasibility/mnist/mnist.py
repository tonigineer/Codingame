import gzip

import numpy as np
import matplotlib.pyplot as plt

from typing import Tuple


class MNIST:
    """Handwritten digits data for Machine Learning.

    NOTE http://yann.lecun.com/exdb/mnist/

    The MNIST database of handwritten digits, available from this page,
    has a training set of 60,000 examples, and a test set of 10,000
    examples. It is a subset of a larger set available from NIST. The digits
    have been size-normalized and centered in a fixed-size image.
    """

    X_LIMITS = (0, 255)
    Y_LIMITS = (0, 9)

    data_sets = {
        'test_x': 'feasibility/mnist/t10k-images-idx3-ubyte.gz',
        'test_y': 'feasibility/mnist/t10k-labels-idx1-ubyte.gz',
        'train_x': 'feasibility/mnist/train-images-idx3-ubyte.gz',
        'train_y': 'feasibility/mnist/train-labels-idx1-ubyte.gz'
    }

    def __init__(self, data_sets: dict = None):
        if data_sets:
            self.data_sets = data_sets

        self._parse_label()
        self._parse_data()

    def _parse_label(self):
        """Parse label data as defined in source."""
        # TRAINING SET LABEL FILE (train-labels-idx1-ubyte):
        #    [offset] [type]          [value]          [description]
        #    0000     32 bit integer  0x00000801(2049) magic number (MSB first)
        #    0004     32 bit integer  60000            number of items
        #    0008     unsigned byte   ??               label
        #    0009     unsigned byte   ??               label
        #    ........
        #    xxxx     unsigned byte   ??               label
        #
        #    The labels values are 0 to 9.
        raw = gzip.open(self.data_sets[f'train_y'], 'r')
        _ = raw.read(4)

        self.size_train = int.from_bytes(raw.read(4), 'big')

        self.train_y = np.frombuffer(
            raw.read(self.size_train), dtype=np.uint8).astype(np.uint8)

        # TEST SET LABEL FILE (t10k-labels-idx1-ubyte):
        #    [offset] [type]          [value]          [description]
        #    0000     32 bit integer  0x00000801(2049) magic number (MSB first)
        #    0004     32 bit integer  10000            number of items
        #    0008     unsigned byte   ??               label
        #    0009     unsigned byte   ??               label
        #    ........
        #    xxxx     unsigned byte   ??               label
        #
        #    The labels values are 0 to 9.
        raw = gzip.open(self.data_sets[f'test_y'], 'r')
        _ = raw.read(4)

        self.size_test = int.from_bytes(raw.read(4), 'big')

        self.test_y = np.frombuffer(
            raw.read(self.size_train), dtype=np.uint8).astype(np.uint8)

    def _parse_data(self):
        """Parse data as defined in source."""
        # TRAINING SET IMAGE FILE (train-images-idx3-ubyte):
        #    [offset] [type]          [value]          [description]
        #    0000     32 bit integer  0x00000803(2051) magic number
        #    0004     32 bit integer  60000            number of images
        #    0008     32 bit integer  28               number of rows
        #    0012     32 bit integer  28               number of columns
        #    0016     unsigned byte   ??               pixel
        #    0017     unsigned byte   ??               pixel
        #    ........
        #    xxxx     unsigned byte   ??               pixel
        #
        #    Pixels are organized row-wise. Pixel values are 0 to 255.
        #    0 means background (white), 255 means foreground (black).
        raw = gzip.open(self.data_sets[f'train_x'], 'r')
        _ = raw.read(4)

        assert self.size_train == int.from_bytes(raw.read(4), 'big')
        self.num_rows = int.from_bytes(raw.read(4), 'big')
        self.num_cols = int.from_bytes(raw.read(4), 'big')

        self.train_x = np.frombuffer(
            raw.read(self.size_train * self.num_rows * self.num_cols),
            dtype=np.uint8
        ).astype(np.uint8)

        self.train_x = self.train_x.reshape(
            self.size_train, self.num_rows, self.num_cols, 1
        )

        # TEST SET IMAGE FILE (t10k-images-idx3-ubyte):
        #   [offset] [type]          [value]          [description]
        #   0000     32 bit integer  0x00000803(2051) magic number
        #   0004     32 bit integer  10000            number of images
        #   0008     32 bit integer  28               number of rows
        #   0012     32 bit integer  28               number of columns
        #   0016     unsigned byte   ??               pixel
        #   0017     unsigned byte   ??               pixel
        #   ........
        #   xxxx     unsigned byte   ??               pixel
        #
        #   Pixels are organized row-wise. Pixel values are 0 to 255.
        #   0 means background (white), 255 means foreground (black).
        raw = gzip.open(self.data_sets[f'test_x'], 'r')
        _ = raw.read(4)

        assert self.size_test == int.from_bytes(raw.read(4), 'big')
        assert self.num_rows == int.from_bytes(raw.read(4), 'big')
        assert self.num_cols == int.from_bytes(raw.read(4), 'big')

        self.test_x = np.frombuffer(
            raw.read(self.size_test * self.num_rows * self.num_cols),
            dtype=np.uint8
        ).astype(np.uint8)

        self.test_x = self.test_x.reshape(
            self.size_test, self.num_rows, self.num_cols, 1
        )

    def get_image(self, idx: int, pool: str) -> np.array:
        return np.asarray(
            (self.train_x[idx] if pool == 'train' else self.test_x[idx])
        ).squeeze()

    def get_label(self, idx: int, pool: str) -> int:
        return self.train_y[idx] if pool == 'train' else self.test_y[idx]

    def get_normalize_data_set(self, num_sample: int, pool: str) -> (np.ndarray, np.ndarray):
        X = self.train_x if pool == 'train' else self.test_x
        X = np.asarray(
            X[0:num_sample]
        ).squeeze() / self.X_LIMITS[1]
        X = X.reshape(-1, X.shape[1] * X.shape[2]).T

        Y = self.train_y if pool == 'train' else self.test_y
        Y = Y[0:num_sample].flatten()  # / self.Y_LIMITS[1]  # maybe needed
        # Y = Y.reshape([-1, 1])
        return (X, Y)

    def show_image(self, idx: int, pool: str) -> None:
        image = self.get_image(idx, pool)
        plt.imshow(image)
        plt.title(f'Labeled as `{self.get_label(idx, pool)}`')
        plt.show()


if __name__ == '__main__':
    mnist = MNIST()
    mnist.show_image(10, 'train')
    mnist.show_image(1000, 'test')
