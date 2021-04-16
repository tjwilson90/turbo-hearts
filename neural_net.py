# python3 -m venv --system-site-packages ./turbo-hearts-venv
# source ./turbo-hearts-venv/bin/activate
# pip install --upgrade pip
# pip install --upgrade tensorflow==2.2.0 keras-tuner onnx onnxmltools

import os
import tensorflow as tf
from tensorflow import keras
import kerastuner as kt
import onnxmltools

all_input_desc = {
    'cards': tf.io.FixedLenFeature([208], tf.float32),
    'won_queen': tf.io.FixedLenFeature([4], tf.float32),
    'won_jack': tf.io.FixedLenFeature([4], tf.float32),
    'won_ten': tf.io.FixedLenFeature([4], tf.float32),
    'won_hearts': tf.io.FixedLenFeature([4], tf.float32),
    'charged': tf.io.FixedLenFeature([4], tf.float32),
    'led': tf.io.FixedLenFeature([3], tf.float32),
}
follow_input_desc = {
    'trick': tf.io.FixedLenFeature([15], tf.float32),
}
policy_output_desc = {
    'plays': tf.io.FixedLenFeature([52], tf.float32),
}
value_output_desc = {
    'win_queen': tf.io.FixedLenFeature([4], tf.float32),
    'win_jack': tf.io.FixedLenFeature([4], tf.float32),
    'win_ten': tf.io.FixedLenFeature([4], tf.float32),
    'win_hearts': tf.io.FixedLenFeature([4], tf.float32),
}
lead_policy_desc = {**all_input_desc, **policy_output_desc}
lead_value_desc = {**all_input_desc, **value_output_desc}
follow_policy_desc = {**all_input_desc, **follow_input_desc, **policy_output_desc}
follow_value_desc = {**all_input_desc, **follow_input_desc, **value_output_desc}

def parse_record(lead, policy, record):
    description = lead_policy_desc if lead and policy else \
            lead_value_desc if lead and not policy else \
            follow_policy_desc if not lead and policy else \
            follow_value_desc
    example = tf.io.parse_single_example(record, description)
    inputs = {}
    outputs = {}
    for feature in description:
        if feature in all_input_desc or feature in follow_input_desc:
            inputs[feature] = example[feature]
        else:
            outputs[feature] = example[feature]
    return inputs, outputs

def load_dataset(train, lead, policy):
    filenames = [
        '/Users/twilson/code/turbo-hearts/data/{}-{}-{}.tfrec.gz'.format(
            'train' if train else 'validate', 
            'lead' if lead else 'follow',
            'policy' if policy else 'value')
    ]
    dataset = tf.data.TFRecordDataset(filenames, compression_type='GZIP')
    dataset = dataset.map(lambda record: parse_record(lead, policy, record))
    return dataset.shuffle(128).batch(32)

def policy_loss(y_true, y_pred):
    cast = tf.cast(y_true >= -1e7, tf.float32)
    square = tf.square(y_true - y_pred)
    return tf.reduce_sum(cast * square) / tf.reduce_sum(cast)

def policy_metric(y_true, y_pred):
    cast = tf.cast(y_true >= -1e7, tf.float32)
    max_true = tf.argmax(y_true)
    max_pred = tf.argmax((y_pred + 1e8) * cast)
    return tf.gather(y_true, max_true) - tf.gather(y_true, max_pred)


def build_model(lead, policy, hp):
    inputs = [
        keras.Input(shape=[208], name='cards', dtype='float32'),
        keras.Input(shape=[4], name='won_queen', dtype='float32'),
        keras.Input(shape=[4], name='won_jack', dtype='float32'),
        keras.Input(shape=[4], name='won_ten', dtype='float32'),
        keras.Input(shape=[4], name='won_hearts', dtype='float32'),
        keras.Input(shape=[4], name='charged', dtype='float32'),
        keras.Input(shape=[3], name='led', dtype='float32'),
    ]

    if not lead:
        inputs.append(keras.Input(shape=[15], name='trick', dtype='float32'))

    layer = keras.layers.concatenate(inputs)
    layer = keras.layers.Dropout(0.05)(layer)

    for i in range(2 if hp is None else hp.Int('num_layers', 2, 4)):
        units = [500, 500][i] if hp is None else hp.Int('units' + str(i), min_value=384, max_value=576, step=64)
        layer = keras.layers.Dense(units = units, activation = 'relu')(layer)
        layer = keras.layers.Dropout(0.05)(layer)

    outputs = [
        keras.layers.Dense(units=52, activation='linear', name='plays')(layer),
    ] if policy else [
        keras.layers.Dense(units=4, activation='softmax', name='win_queen')(layer),
        keras.layers.Dense(units=4, activation='softmax', name='win_jack')(layer),
        keras.layers.Dense(units=4, activation='softmax', name='win_ten')(layer),
        keras.layers.Dense(units=4, activation='linear', name='win_hearts')(layer),
    ]

    model = keras.Model(inputs, outputs)
    model.compile(
        optimizer = keras.optimizers.Adam(learning_rate=1e-4),
        loss = {
            'plays': policy_loss,
        } if policy else {
            'win_queen': 'categorical_crossentropy',
            'win_jack': 'categorical_crossentropy',
            'win_ten': 'categorical_crossentropy',
            'win_hearts': 'mean_squared_error',
        },
        metrics = {
            'plays': [policy_metric],
        } if policy else {
            'win_queen': ['categorical_accuracy'],
            'win_jack': ['categorical_accuracy'],
            'win_ten': ['categorical_accuracy'],
            'win_hearts': ['mean_squared_error'],
        },
        loss_weights = {
            'plays': 1,
        } if policy else {
            'win_queen': 3,
            'win_jack': 2,
            'win_ten': 2,
            'win_hearts': 1,
        },
    )
    return model

def hypertune(lead, policy):
    tuner = kt.Hyperband(
        hypermodel = lambda hp: build_model(lead, policy, hp),
        objective = 'val_loss',
        max_epochs = 16,
    )

    tuner.search(
        load_dataset(True, lead, policy),
        epochs = 16,
        validation_data = load_dataset(False, lead, policy)
    )

    best_hps = tuner.get_best_hyperparameters(num_trials = 1)[0]
    print(best_hps.values)

def fit(lead, policy):
    model = build_model(lead, policy, None)
    model.fit(
        x = load_dataset(True, lead, policy),
        epochs = 16,
        validation_data = load_dataset(False, lead, policy)
    )
    return model

for lead in [True, False]:
    for policy in [True, False]:
        name = 'assets/{}-{}.onnx'.format('lead' if lead else 'follow', 'policy' if policy else 'value')
        print('Generating', name)
        tf_model = fit(lead, policy)
        onnx_model = onnxmltools.convert_keras(tf_model)
        onnxmltools.utils.save_model(onnx_model, name)

#hypertune(True)
