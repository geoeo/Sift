use crate::features::{Feature, geometry::{point::Point,shape::circle::{Circle,circle_bresenham},Offset}};
use crate::image::Image;
use crate::{float,Float};

#[derive(Debug,Clone)]
pub struct FastFeature {
    pub location: Point<usize>,
    pub radius: usize,
    pub starting_offsets: [Offset;4],
    pub continuous_offsets: Vec<Offset>

}

impl Feature for FastFeature {
    fn get_x_image(&self) -> usize {
        self.location.x
    }

    fn get_y_image(&self) -> usize {
        self.location.y
    }

    fn get_closest_sigma_level(&self) -> usize {
        0
    }
}

impl FastFeature {

    fn new(x_center: usize, y_center: usize, radius: usize) -> FastFeature {
        FastFeature::from_circle(&circle_bresenham(x_center,y_center,radius))
    }

    fn from_circle(circle: &Circle) -> FastFeature {
        let circle_geometry = &circle.shape;
        let starting_offsets = [circle_geometry.offsets[0],circle_geometry.offsets[1],circle_geometry.offsets[2],circle_geometry.offsets[3]];
        let mut positive_y_offset = Vec::<Offset>::with_capacity(circle_geometry.offsets.len()/2);
        let mut negative_y_offset = Vec::<Offset>::with_capacity(circle_geometry.offsets.len()/2);
        let mut pos_zero = Vec::<Offset>::with_capacity(1);
        let mut neg_zero = Vec::<Offset>::with_capacity(1);
        
        for offset in &circle_geometry.offsets {
            match offset {
                val if val.y > 0 => positive_y_offset.push(*val),
                val if val.y < 0 => negative_y_offset.push(*val),
                val if val.y == 0 && val.x > 0 => pos_zero.push(*val),
                val => neg_zero.push(*val)
            };

            positive_y_offset.sort_unstable_by(|a,b| b.cmp(a));
            negative_y_offset.sort_unstable_by(|a,b| a.cmp(b));

        }



        let mut continuous_offsets = [pos_zero,positive_y_offset,neg_zero,negative_y_offset].concat();
        continuous_offsets.dedup();

        FastFeature {location: circle_geometry.center,radius: circle.radius, starting_offsets,continuous_offsets}
    }

    fn accept(image: &Image, feature: &FastFeature, threshold_factor: Float, consecutive_pixels: usize) -> (Option<usize>,Float) {

        if (feature.location.x as isize - feature.radius as isize) < 0 || feature.location.x + feature.radius >= image.buffer.ncols() ||
           (feature.location.y as isize - feature.radius as isize) < 0 || feature.location.y + feature.radius >= image.buffer.nrows() {
               return (None,float::MIN);
           }

        let sample_intensity = image.buffer[(feature.location.y,feature.location.x)];
        let t = sample_intensity*threshold_factor;
        let cutoff_max = sample_intensity + t;
        let cutoff_min = sample_intensity - t;

        let number_of_accepted_starting_offsets: usize
            = feature.starting_offsets.iter()
            .map(|x| FastFeature::sample(image, feature.location.y as isize, feature.location.x as isize, x))
            .map(|x| FastFeature::outside_range(x, cutoff_min, cutoff_max))
            .map(|x| x as usize)
            .sum();

        let perimiter_samples: Vec<Float> = feature.continuous_offsets.iter().map(|x| FastFeature::sample(image, feature.location.y as isize, feature.location.x as isize, x)).collect();
        let flagged_scores: Vec<(bool,Float)> = perimiter_samples.iter().map(|&x| (FastFeature::outside_range(x, cutoff_min, cutoff_max),FastFeature::score(sample_intensity, x, t))).collect();

        match number_of_accepted_starting_offsets {
            count if count >= 3 => {

                let mut max_score = float::MIN;
                let mut max_score_index: Option<usize> = None;
                for i in 0..feature.continuous_offsets.len() {
                    let index_slice = feature.get_wrapping_index(i, consecutive_pixels);
                    let (all_passing,total_score) = index_slice.iter()
                    .map(|&x| flagged_scores[x])
                    .fold((true,0.0),|(b_acc,s_acc),x| (b_acc && x.0,s_acc + x.1));

                    if all_passing {
                        if total_score > max_score {
                            max_score = total_score;
                            max_score_index = Some(i);
                        }
                    }
                };

                (max_score_index,max_score)
            },
            _ => (None,float::MIN)
        }

    }

    pub fn compute_valid_feature(image: &Image, radius: usize,  threshold_factor: Float, consecutive_pixels: usize, x_grid_start: usize, y_grid_start: usize, x_grid_size: usize, y_grid_size: usize) -> Option<(FastFeature,usize)> {
        let mut grid_max_option: Option<(FastFeature,usize)> = None;
        let mut grid_max_score = float::MIN;
        
        for r_grid in y_grid_start..y_grid_start+y_grid_size {
            for c_grid in x_grid_start..x_grid_start+x_grid_size {
                let feature = FastFeature::new(c_grid, r_grid, radius);
                let (start_option,score) = FastFeature::accept(image, &feature, threshold_factor, consecutive_pixels);
                if start_option.is_some() && score > grid_max_score {
                    grid_max_score = score;
                    grid_max_option = Some((feature,start_option.unwrap()));
                }

            }
        }

        grid_max_option

    }

    pub fn compute_valid_features(image: &Image, radius: usize,  threshold_factor: Float, consecutive_pixels: usize, grid_size: (usize,usize), offsets: (usize,usize)) -> Vec<(FastFeature,usize)> {
        let x_grid = grid_size.0;
        let y_grid = grid_size.1;
        let x_offset = offsets.0;
        let y_offset = offsets.1;
        

        let mut result = Vec::<(FastFeature,usize)>::new();
        for r in (y_offset..image.buffer.nrows() - y_offset).step_by(y_grid) {
            for c in (x_offset..image.buffer.ncols() - x_offset).step_by(x_grid) {

                match FastFeature::compute_valid_feature(image,radius,threshold_factor,consecutive_pixels,c,r,x_grid,y_grid) {
                    Some(v) => result.push(v),
                    None => {}
                }

            }
        }

        result

    }

    fn sample(image: &Image, y_center: isize, x_center: isize, offset: &Offset) -> Float {
        image.buffer[((y_center + offset.y) as usize, (x_center + offset.x) as usize)]
    }

    fn outside_range(perimiter_sample: Float, cutoff_min: Float, cutoff_max: Float) -> bool {
        FastFeature::is_bright(perimiter_sample, cutoff_max) || FastFeature::is_dark(perimiter_sample, cutoff_min)
    }

    fn score(sample :Float, perimiter_sample: Float, t: Float) -> Float {
        (sample-perimiter_sample).abs() - t
    }

    fn is_bright(perimiter_sample: Float, cutoff_max: Float) -> bool {
        perimiter_sample >= cutoff_max
    }

    fn is_dark(perimiter_sample: Float, cutoff_min: Float) -> bool {
        perimiter_sample <= cutoff_min
    }

    pub fn get_wrapping_slice(&self, start: usize, size: usize) -> Vec<Offset> {
        let len = self.continuous_offsets.len();
        assert!(size <= len);

        let total = start + size;
        if total < len {
            self.continuous_offsets[start..total].to_vec()
        } else {
            let left_over = total - len;
            [self.continuous_offsets[start..len].to_vec(),self.continuous_offsets[0..left_over].to_vec()].concat()
        }

    }

    pub fn get_wrapping_index(&self, start: usize, size: usize) -> Vec<usize> {
        let len = self.continuous_offsets.len();
        assert!(size <= len);

        let total = start + size;
        if total < len {
            (start..total).collect()
        } else {
            let left_over = total - len;
            [(start..len).collect::<Vec<usize>>(),(0..left_over).collect()].concat()
        }

    }

    pub fn get_full_circle(&self) -> Circle {
        let continuous_offset_half = self.continuous_offsets.len()/2;

        let mut offsets = Vec::<Offset>::new();
        let mut pos_offsets = Vec::<Offset>::new();
        let mut neg_offsets = Vec::<Offset>::new();

        offsets.push(self.continuous_offsets[0]);
        offsets.push(self.continuous_offsets[continuous_offset_half]);

        for i in 1..continuous_offset_half {
            let pos_y_idx = i;
            let neg_y_idy = continuous_offset_half + i;

            let pos_y_offset = self.continuous_offsets[pos_y_idx];
            let neg_y_offset = self.continuous_offsets[neg_y_idy];

            pos_offsets.push(pos_y_offset);
            neg_offsets.push(neg_y_offset);
        }
        neg_offsets.reverse();
        let offset_pairs_iter = pos_offsets.iter().zip(neg_offsets.iter());

        for (p,n) in offset_pairs_iter {
            for y in n.y..p.y+1{
                offsets.push(Offset{x:p.x,y});
            }

        }

        Circle::new(self.location.x,self.location.y, self.radius, offsets)
    }

    pub fn print_continuous_offsets(feature: &FastFeature) -> () {
        for offset in &feature.continuous_offsets {
            println!("{:?}",offset);
        } 
    }


}