use nalgebra::{Isometry3, Translation3, UnitQuaternion};
use opencv::{
    core::Vec2d,
    types::{VectorOfVec2d, VectorOfVec3d, VectorOff64},
};

use crate::{
    config::Config,
    types::{
        isometry_from_opencv, translation_to_opencv, CameraPoseObservation,
        FiducialImageObservation,
    },
};

pub trait CameraPoseEstimator {
    fn solve_camera_pose(
        &mut self,
        image_observations: Vec<FiducialImageObservation>,
        config_store: &Config,
    ) -> Option<CameraPoseObservation>;
}

pub struct MultiTargetCameraPoseEstimator;

impl CameraPoseEstimator for MultiTargetCameraPoseEstimator {
    fn solve_camera_pose(
        &mut self,
        image_observations: Vec<FiducialImageObservation>,
        config_store: &Config,
    ) -> Option<CameraPoseObservation> {
        if image_observations.len() == 0 {
            return None;
        }
        let fid_size = config_store.fiducial_size_m;
        let mut object_points = VectorOfVec3d::new();
        let mut image_points = VectorOfVec2d::new();
        let mut tag_ids = Vec::new();
        let mut tag_poses = Vec::new();
        for observation in image_observations {
            if let Some(tag_pose) = config_store
                .tag_layout
                .tags
                .iter()
                .find(|x| x.id == observation.tag_id)
                .map(|x| x.pose)
            {
                let corner_0 = tag_pose
                    * Isometry3::<f64>::from_parts(
                        Translation3::new(0.0, fid_size / 2.0, -fid_size / 2.0),
                        UnitQuaternion::identity(),
                    );
                let corner_1 = tag_pose
                    * Isometry3::<f64>::from_parts(
                        Translation3::new(0.0, -fid_size / 2.0, -fid_size / 2.0),
                        UnitQuaternion::identity(),
                    );
                let corner_2 = tag_pose
                    * Isometry3::<f64>::from_parts(
                        Translation3::new(0.0, -fid_size / 2.0, fid_size / 2.0),
                        UnitQuaternion::identity(),
                    );
                let corner_3 = tag_pose
                    * Isometry3::<f64>::from_parts(
                        Translation3::new(0.0, fid_size / 2.0, fid_size / 2.0),
                        UnitQuaternion::identity(),
                    );
                object_points.extend([
                    translation_to_opencv(corner_0.translation.vector),
                    translation_to_opencv(corner_1.translation.vector),
                    translation_to_opencv(corner_2.translation.vector),
                    translation_to_opencv(corner_3.translation.vector),
                ]);

                image_points.extend([
                    Vec2d::from_array(observation.corners[0]),
                    Vec2d::from_array(observation.corners[1]),
                    Vec2d::from_array(observation.corners[2]),
                    Vec2d::from_array(observation.corners[3]),
                ]);

                tag_ids.push(observation.tag_id);
                tag_poses.push(tag_pose);
            }
        }

        if tag_ids.len() == 0 {
            return None;
        } else if tag_ids.len() == 1 {
            let mut rvecs = VectorOfVec3d::new();
            let mut tvecs = VectorOfVec3d::new();
            let mut errors = VectorOff64::new();
            if let Err(e) = opencv::calib3d::solve_pnp_generic(
                &object_points,
                &image_points,
                &config_store.camera_matrix,
                &config_store.distortion_coefficients,
                &mut rvecs,
                &mut tvecs,
                false,
                opencv::calib3d::SolvePnPMethod::SOLVEPNP_IPPE_SQUARE,
                &opencv::core::no_array(),
                &opencv::core::no_array(),
                &mut errors,
            ) {
                eprintln!("{}", e);
                return None;
            }
            let field_to_tag_pose = tag_poses[0];
            if tvecs.len() < 2 || rvecs.len() < 2 {
                println!("Invalid tvecs/rvecs");
                return None;
            }
            let camera_to_tag_pose_0 =
                isometry_from_opencv(tvecs.get(0).unwrap(), rvecs.get(0).unwrap());
            let camera_to_tag_pose_1 =
                isometry_from_opencv(tvecs.get(1).unwrap(), rvecs.get(1).unwrap());
            let field_to_camera_0 = field_to_tag_pose * camera_to_tag_pose_0.inverse();
            let field_to_camera_1 = field_to_tag_pose * camera_to_tag_pose_1.inverse();

            return Some(CameraPoseObservation {
                tag_ids,
                pose_0: field_to_camera_0,
                error_0: errors.get(0).unwrap(),
                pose_1: Some(field_to_camera_1),
                error_1: Some(errors.get(1).unwrap()),
            });
        } else {
            let mut rvecs = VectorOfVec3d::new();
            let mut tvecs = VectorOfVec3d::new();
            let mut errors = VectorOff64::new();
            if let Err(e) = opencv::calib3d::solve_pnp_generic(
                &object_points,
                &image_points,
                &config_store.camera_matrix,
                &config_store.distortion_coefficients,
                &mut rvecs,
                &mut tvecs,
                false,
                opencv::calib3d::SolvePnPMethod::SOLVEPNP_SQPNP,
                &opencv::core::no_array(),
                &opencv::core::no_array(),
                &mut errors,
            ) {
                eprintln!("{}", e);
                return None;
            }

            let camera_to_field_pose =
                isometry_from_opencv(tvecs.get(0).unwrap(), rvecs.get(0).unwrap());
            let field_to_camera = camera_to_field_pose.inverse();
            return Some(CameraPoseObservation {
                tag_ids,
                pose_0: field_to_camera,
                error_0: errors.get(0).unwrap(),
                pose_1: None,
                error_1: None,
            });
        }
    }
}
