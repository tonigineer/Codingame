# Using a Neural Network in a Bot Game on Codingame

After competing in multiple challenges on Codingame with bots using simple heuristics, it's time for something new: the goal is to learn and advance to the Gold league in an upcoming challenge.

> [!IMPORTANT]
> "Codingame doesn't have NN specific libraries. For Codingame people mostly train locally using NN libraries, and just copy the learned weights and implement the forward propagation themselves in their language." - [jacek](https://www.codingame.com/profile/f0c03de45623ce8d5e05cf647e381a807876313) from [Codingame](https://www.codingame.com/playgrounds/59631/neural-network-xor-example-from-scratch-no-libs)

```shell
git clone https://github.com/tonigineer/nn-codingame.git ~/journey && cd ~/journey

# Optional, but recommended
python -m venv .venv
source .venv/bin/activate

pip install -r requirements.txt
```

## 1️⃣ Check feasibility with Python

Using the [MNIST](https://paperswithcode.com/dataset/mnist) dataset to train a simple Neural Network with [TensorFlow](https://www.tensorflow.org/datasets/keras_example) and then transferring the parameters to a Neural Network implemented from [scratch](https://www.kaggle.com/code/wwsalmon/simple-mnist-nn-from-scratch-numpy-no-tf-keras/notebook).

```shell
python feasibility/example_ann_tf.py

...
Epoch 10/10
1875/1875 ━━━━━━━━━━━━━━━━━━━━ 1s 531us/step - accuracy: 0.9960 - loss: 0.0140
313/313 - 0s - 459us/step - accuracy: 0.9794 - loss: 0.0787

```

The weights and biases have been exported. The ANN, implemented in `example_ann_scratch.py`, also includes backpropagation and gradient descent, which are not necessary for this task.

```shell
python feasibility/ann_with_tf_param.py

...
Accuracy with test data: 97.94%
```

> ✔️ Both evaluations yield the same accuracy on the test data from `mnist = tf.keras.datasets.mnist`. Therefore, transferring weights and biases to a custom implementation from scratch proves to be effective.

Lessons learned for the next parts:

- [ ] Adapt sizes to TensorFlow conventions from the outset
- [ ] Implement a dynamic architecture based on weights and biases

## 2️⃣ Implemented Neural Network in Rust

Todo



## Keep in mind

- [ ] 100k limit on characters from Codingame
- [ ] plain text export of weights to use on Codingame 
- [ ] Reduce size of numbers
