use opencv::{
    aruco::Dictionary,
    core::Ptr,
    types::{VectorOfVectorOfPoint2f, VectorOfi32},
};

use crate::{config::Config, types::FiducialImageObservation};

pub trait FiducialDetector {
    fn detect_fiducial(
        &mut self,
        image: &mut opencv::prelude::Mat,
        config_store: &Config,
    ) -> Vec<FiducialImageObservation>;
}

pub struct ArucoFiducialDetector {
    aruco_dict: Ptr<Dictionary>,
}

impl ArucoFiducialDetector {
    pub fn new(dictionary_id: i32) -> Self {
        let aruco_dict = opencv::aruco::get_predefined_dictionary_i32(dictionary_id).unwrap();
        Self {
            aruco_dict
        }
    }
}

impl FiducialDetector for ArucoFiducialDetector {
    fn detect_fiducial(
        &mut self,
        image: &mut opencv::prelude::Mat,
        _config_store: &Config,
    ) -> Vec<FiducialImageObservation> {
        let mut corners = VectorOfVectorOfPoint2f::default();
        let mut ids = VectorOfi32::default();
        opencv::aruco::detect_markers_def(image, &self.aruco_dict, &mut corners, &mut ids)
            .unwrap();
        opencv::aruco::draw_detected_markers(image, &corners, &ids, opencv::core::Scalar::new(0.0, 255.0, 0.0, 255.0)).unwrap();
        ids.into_iter().zip(corners).map(|(id, corners)| {
            let corner1 = corners.get(0).unwrap();
            let corner2 = corners.get(1).unwrap();
            let corner3 = corners.get(2).unwrap();
            let corner4 = corners.get(3).unwrap();
            FiducialImageObservation {
                tag_id: id as u64,
                corners: [
                    [corner1.x as f64, corner1.y as f64],
                    [corner2.x as f64, corner2.y as f64],
                    [corner3.x as f64, corner3.y as f64],
                    [corner4.x as f64, corner4.y as f64],
                ]
            }
        }).collect()
    }
}
