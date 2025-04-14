CREATE TABLE IF NOT EXISTS users (
                                     id SERIAL PRIMARY KEY,
                                     username TEXT NOT NULL
);

CREATE TYPE mask_type AS ENUM (
    'occlusion',
    'saliency',
    'layer_gradcam',
    'integrated_gradients',
    'guided_gradcam',
    'gradient_shap'
    );

CREATE TABLE IF NOT EXISTS images (
                                      id SERIAL PRIMARY KEY,
                                      filename VARCHAR(255) NOT NULL,
                                      patient_nb VARCHAR(255) NOT NULL
);

CREATE TABLE IF NOT EXISTS masks (
                                     id SERIAL PRIMARY KEY,
                                     image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
                                     mask_type mask_type,
                                     filename VARCHAR(255) NOT NULL
);

CREATE TABLE IF NOT EXISTS preferences (
                                           id SERIAL PRIMARY KEY,
                                           user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                           image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
                                           created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                                           UNIQUE (user_id, image_id)
);

CREATE TABLE IF NOT EXISTS preference_events (
                                                 id SERIAL PRIMARY KEY,
                                                 preference_id INTEGER NOT NULL REFERENCES preferences(id) ON DELETE CASCADE,
                                                 mask_id INTEGER NOT NULL REFERENCES masks(id) ON DELETE CASCADE,
                                                 rank INTEGER NOT NULL,
                                                 UNIQUE (preference_id, mask_id)
);

-- Indices
CREATE INDEX IF NOT EXISTS idx_masks_image_id ON masks (image_id);
CREATE INDEX IF NOT EXISTS idx_preferences_user_id ON preferences (user_id);
CREATE INDEX IF NOT EXISTS idx_preferences_image_id ON preferences (image_id);
CREATE INDEX IF NOT EXISTS idx_preference_events_preference_id ON preference_events (preference_id);
CREATE INDEX IF NOT EXISTS idx_preference_events_mask_id ON preference_events (mask_id);