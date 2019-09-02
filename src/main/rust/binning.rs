/*
 * Copyright (C) 2019 JJ Brown
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::convert::TryInto; // introduces try_into() for converting i32 -> usize / usize -> isize
use std::ops::Index;

// Corresponds to com.brownian.ffi.javarust.struct.DataSet
// Marked with repr(C), like all JNI structs,
// because that's how JNA can read the fields.
#[allow(non_snake_case)]
#[repr(C)]
pub struct DataSet {
    samples: *const f64,
    numSamples: i32,
}

impl DataSet {
    pub fn get_samples(&self) -> &[f64] {
        unsafe { std::slice::from_raw_parts(self.samples, self.numSamples.try_into().unwrap()) }
    }
}

#[repr(C)]
pub struct Bin {
    count: u32,
}

impl Bin {
    pub fn empty() -> Bin {
        Bin { count: 0 }
    }

    pub fn new(count: u32) -> Bin {
        Bin { count }
    }

    pub fn increment(self: &mut Bin) {
        self.count += 1;
    }
}
// TODO: implement ops::Add for Bins

#[allow(non_snake_case)]
#[repr(C)]
pub struct Histogram {
    left: f64,
    right: f64,
    underflow: Bin,
    overflow: Bin,
    numBins: i32, // note that this SHOULD be usize, but is i32 to match the corresponding Java int field
    bins: *mut Bin, // necessary in order to accept an array from Java
}

impl Histogram {
    pub fn count(self: &mut Histogram, datum: f64) {
        let main_bins = self.get_bins_mut();
        self.get_bin_mut(main_bins, datum).increment();
    }

    pub fn count_all(self: &mut Histogram, data: &[f64]) {
        let main_bins = self.get_bins_mut();
        for &datum in data {
            self.get_bin_mut(main_bins, datum).increment();
        }
    }

    fn get_bin_mut<'a>(&'a mut self, main_bins: &'a mut [Bin], datum: f64) -> &'a mut Bin {
        if datum < self.left {
            &mut self.underflow
        } else if datum >= self.right {
            &mut self.overflow
        } else {
            let index: usize = ((datum - self.left) / self.get_bin_width()) as usize;
            &mut main_bins[index]
        }
    }

    pub fn get_bins<'a>(self: &'a Histogram) -> &'a [Bin] {
        unsafe { std::slice::from_raw_parts(self.bins, self.numBins.try_into().unwrap()) }
    }

    fn get_bins_mut<'a, 'b>(self: &mut Histogram) -> &'b mut [Bin] {
        unsafe { std::slice::from_raw_parts_mut(self.bins, self.numBins.try_into().unwrap()) }
    }

    pub fn get_bin<'a>(self: &'a Histogram, datum: f64) -> &'a Bin {
        if datum < self.left {
            &self.underflow
        } else if datum >= self.right {
            &self.overflow
        } else {
            let index: usize = ((datum - self.left) / self.get_bin_width()) as usize;
            &self.get_bins()[index]
        }
    }

    pub fn get_bin_width(self: &Histogram) -> f64 {
        (self.right - self.left) / (self.numBins as f64)
    }

    pub fn new(left: f64, num_bins: usize, right: f64) -> (Histogram, Vec<Bin>) {
        let mut bins: Vec<Bin> = std::iter::repeat_with(Bin::empty).take(num_bins).collect();
        (
            Histogram {
                left: left,
                right: right,
                underflow: Bin::empty(),
                overflow: Bin::empty(),
                numBins: num_bins.try_into().unwrap(),
                bins: bins.as_mut_ptr(),
            },
            bins,
        )
    }
}

impl Index<f64> for Histogram {
    type Output = Bin;

    fn index(&self, datum: f64) -> &Self::Output {
        self.get_bin(datum)
    }
}

#[no_mangle]
pub extern "C" fn bin(dataset: &DataSet, hist: &mut Histogram) {
    let samples_ref: &[f64] = dataset.get_samples();

    hist.count_all(samples_ref);
}

#[no_mangle]
pub extern "C" fn increment(bin: &mut Bin) {
    bin.increment()
}

#[no_mangle]
pub unsafe extern "C" fn increment_all(bins_ptr: *mut Bin, num_bins: usize) {
    let bins: &mut [Bin] = std::slice::from_raw_parts_mut(bins_ptr, num_bins);
    for bin in bins {
        bin.increment();
    }
}

#[test]
fn test_increment_all() {
    let mut bins = vec![Bin::empty(), Bin::empty(), Bin::empty(), Bin::empty()];
    unsafe {
        increment_all(bins.as_mut_ptr(), bins.len());
    }
    for bin in bins {
        assert_eq!(1, bin.count);
    }
}

#[no_mangle]
pub extern "C" fn count_sample(hist: &mut Histogram, sample: f64) {
    hist.count(sample)
}

#[test]
fn test_count_underflow() {
    let (mut hist, _bins) = Histogram::new(0.0, 5, 5.0);
    count_sample(&mut hist, -1.0);
    assert_eq!(1, hist.underflow.count);
}

#[test]
fn test_count_overflow() {
    let (mut hist, _bins) = Histogram::new(0.0, 5, 5.0);
    count_sample(&mut hist, 6.0);
    assert_eq!(1, hist.overflow.count);
}

#[test]
fn test_bin_linear_dataset() {
    let data = vec![-0.5, 0.5, 1.5, 1.7, 3.1, 3.5, 3.6, 4.5, 5.5];
    let dataset = DataSet {
        samples: data.as_ptr(),
        numSamples: data.len() as i32,
    };
    let (mut hist, _bins) = Histogram::new(0.0, 5, 5.0);
    bin(&dataset, &mut hist);

    assert_eq!(1, hist.underflow.count);
    assert_eq!(1, hist.get_bins()[0].count);
    assert_eq!(2, hist.get_bins()[1].count);
    assert_eq!(0, hist.get_bins()[2].count);
    assert_eq!(3, hist.get_bins()[3].count);
    assert_eq!(1, hist.get_bins()[4].count);
    assert_eq!(1, hist.overflow.count);
}

#[no_mangle]
pub extern "C" fn sum_samples(dataset: &DataSet) -> f64 {
    dataset.get_samples().iter().sum()
}

#[test]
fn test_sum_samples() {
    let data = vec![-1.0, 2.0, 3.0, 4.0];
    let sum: f64 = 8.0;
    let dataset = DataSet {
        samples: data.as_ptr(),
        numSamples: data.len() as i32,
    };
    assert_eq!(sum, sum_samples(&dataset));
}

#[cfg(test)]
mod bench_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_random_bins() {
        let random_data: Vec<f64> = std::iter::successors(Some(-0.01), |n| Some(n + 0.01))
            .take(10000)
            .collect();
        let data_set = DataSet {
            samples: random_data.as_ptr(),
            numSamples: random_data.len().try_into().unwrap(),
        };

        let (mut hist, _bins) = Histogram::new(0.0, 100, 99.0);
        let start_time = Instant::now();
        for _ in 0..1000 {
            bin(&data_set, &mut hist);
        }
        let end_time = Instant::now();
        let duration = end_time - start_time;
        println!(
            "Duration: {:?}s, per iter: {:?}ms, per sample: {:?}us",
            (duration.as_millis() as f32) / 1000.0,
            (duration.as_millis() as f32) / 1000.0,
            (duration.as_millis() as f32) / 10000.0
        );
        println!("{:?}", hist.get_bins()[3].count); // to avoid optimizing it away
    }
}
