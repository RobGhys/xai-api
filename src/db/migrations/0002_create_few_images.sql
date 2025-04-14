INSERT INTO images(id, filename, patient_nb)
VALUES
    (1, 'video_0000_0658.jpg', '001');

INSERT INTO masks(image_id, mask_type, filename)
VALUES
    (1,'occlusion', 'occlusion_colored_video_0000_0658.jpg'),
    (1,'layer_gradcam',  'layer_gradcam_colored_video_0000_0658.jpg'),
    (1,'integrated_gradients', 'integrated_gradients_colored_video_0000_0658.jpg'),
    (1,'guided_gradcam', 'guided_gradcam_colored_video_0000_0658.jpg'),
    (1,'gradient_shap', 'gradient_shap_colored_video_0000_0658.jpg'),
    (1,'saliency','saliency_colored_video_0000_0658.jpg');