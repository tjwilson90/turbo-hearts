# python3 -m venv --system-site-packages ./turbo-hearts-venv
# source ./turbo-hearts-venv/bin/activate
# pip install --upgrade pip
# pip install --upgrade tensorflow keras-tuner onnx onnxmltools

import glob
import os
import tensorflow as tf
from tensorflow import keras
import kerastuner as kt
import onnxmltools

lead_desc = {
    'cards': tf.io.FixedLenFeature([260], tf.float32),
    'won_queen': tf.io.FixedLenFeature([4], tf.float32),
    'won_jack': tf.io.FixedLenFeature([4], tf.float32),
    'won_ten': tf.io.FixedLenFeature([4], tf.float32),
    'won_hearts': tf.io.FixedLenFeature([4], tf.float32),
    'charged': tf.io.FixedLenFeature([4], tf.float32),
    'led': tf.io.FixedLenFeature([3], tf.float32),
    'win_queen': tf.io.FixedLenFeature([4], tf.float32),
    'win_jack': tf.io.FixedLenFeature([4], tf.float32),
    'win_ten': tf.io.FixedLenFeature([4], tf.float32),
    'win_hearts': tf.io.FixedLenFeature([4], tf.float32),
}
follow_desc = lead_desc.copy()
follow_desc['trick'] = tf.io.FixedLenFeature([62], tf.float32)

def parse_record(lead, record):
    description = lead_desc if lead else follow_desc
    example = tf.io.parse_single_example(record, description)
    inputs = {}
    outputs = {}
    for feature in description:
        if feature.startswith('win_'):
            outputs[feature] = example[feature]
        else:
            inputs[feature] = example[feature]
    return inputs, outputs

def load_dataset(train_or_validate, lead):
    filenames = glob.glob("/Users/twilson/code/turbo-hearts/data/" \
            + train_or_validate + "/" \
            + ('lead' if lead else 'follow') + "/*.tfrec.gz")
    dataset = tf.data.TFRecordDataset(filenames, compression_type='GZIP')
    dataset = dataset.map(lambda record: parse_record(lead, record))
    dataset = dataset.batch(32)
    return dataset

def build_model(lead, hp):
    inputs = [
        keras.Input(shape=[260], name='cards', dtype='float32'),
        keras.Input(shape=[4], name='won_queen', dtype='float32'),
        keras.Input(shape=[4], name='won_jack', dtype='float32'),
        keras.Input(shape=[4], name='won_ten', dtype='float32'),
        keras.Input(shape=[4], name='won_hearts', dtype='float32'),
        keras.Input(shape=[4], name='charged', dtype='float32'),
        keras.Input(shape=[3], name='led', dtype='float32'),
    ]

    if not lead:
        inputs.append(keras.Input(shape=[62], name='trick', dtype='float32'))

    layer = keras.layers.concatenate(inputs)
    layer = keras.layers.Dropout(0.05)(layer)

    for i in range(2 if hp is None else hp.Int('num_layers', 2, 4)):
        units = [500, 500][i] if hp is None else hp.Int('units' + str(i), min_value=384, max_value=576, step=64)
        layer = keras.layers.Dense(units = units, activation = 'relu')(layer)
        layer = keras.layers.Dropout(0.05)(layer)

    outputs = [
        keras.layers.Dense(units=4, activation='softmax', name='win_queen')(layer),
        keras.layers.Dense(units=4, activation='softmax', name='win_jack')(layer),
        keras.layers.Dense(units=4, activation='softmax', name='win_ten')(layer),
        keras.layers.Dense(units=4, activation='linear', name='win_hearts')(layer),
    ]
    model = keras.Model(inputs, outputs)
    model.compile(
        optimizer = keras.optimizers.Adam(learning_rate=1e-4),
        loss = {
            'win_queen': 'categorical_crossentropy',
            'win_jack': 'categorical_crossentropy',
            'win_ten': 'categorical_crossentropy',
            'win_hearts': 'mean_squared_error',
        },
        metrics = {
            'win_queen': ['categorical_accuracy'],
            'win_jack': ['categorical_accuracy'],
            'win_ten': ['categorical_accuracy'],
            'win_hearts': ['mean_squared_error'],
        },
        loss_weights = {
            'win_queen': 3,
            'win_jack': 2,
            'win_ten': 2,
            'win_hearts': 1,
        },
    )
    return model

def hypertune(lead):
    tuner = kt.Hyperband(
        hypermodel = lambda hp: build_model(lead, hp),
        objective = 'val_loss',
        max_epochs = 16,
    )

    tuner.search(
        load_dataset('train', lead),
        epochs = 16,
        validation_data = load_dataset('validate', lead)
    )

    best_hps = tuner.get_best_hyperparameters(num_trials = 1)[0]
    print(best_hps.values)

def fit(lead):
    model = build_model(lead, None)
    model.fit(
        x = load_dataset('train', lead),
        epochs = 16,
        validation_data = load_dataset('validate', lead)
    )
    return model

lead_model = fit(True)
onnx_lead_model = onnxmltools.convert_keras(lead_model)
onnxmltools.utils.save_model(onnx_lead_model, 'assets/lead-model.onnx')

follow_model = fit(False)
onnx_follow_model = onnxmltools.convert_keras(follow_model)
onnxmltools.utils.save_model(onnx_follow_model, 'assets/follow-model.onnx')

#hypertune(True)
