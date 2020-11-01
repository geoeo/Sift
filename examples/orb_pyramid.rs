extern crate image as image_rs;
extern crate sift;

use std::path::Path;

use sift::pyramid::orb::{build_orb_pyramid, orb_runtime_parameters::OrbRuntimeParameters};

fn main() {
    let image_name = "circles";
    let image_format = "png";
    let image_folder = "images/";
    let image_out_folder = "output/";
    let image_path = format!("{}{}.{}",image_folder,image_name, image_format);


    let gray_image = image_rs::open(&Path::new(&image_path)).unwrap().to_luma();

    let runtime_params = OrbRuntimeParameters {
        sigma: 0.8,
        blur_radius: 5.0,
        octave_count: 5
    };
    
    let pyramid = build_orb_pyramid(&gray_image, &runtime_params);

    for i in 0..pyramid.octaves.len() {
        let octave = &pyramid.octaves[i];
        let image = &octave.images[0];
        let gray_image  = image.to_image();

        let name = format!("orb_image_{}",i);
        let file_path = format!("{}{}.{}",image_out_folder,name,image_format);
        gray_image.save(file_path).unwrap();
    }

}