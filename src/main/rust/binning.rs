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
use std::ops::{Index, IndexMut};

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

#[derive(Debug, Copy, Clone)]
pub enum BinIndex {
    Underflow,
    Bin(usize),
    Overflow,
}

impl Histogram {
    pub fn count(self: &mut Histogram, datum: f64) {
        let idx = self.get_bin_index(datum);
        self[idx].increment();
    }

    pub fn get_bins<'a>(self: &'a Histogram) -> &'a [Bin] {
        unsafe { std::slice::from_raw_parts(self.bins, self.numBins.try_into().unwrap()) }
    }

    pub fn get_bins_mut<'a>(self: &'a mut Histogram) -> &'a mut [Bin] {
        unsafe { std::slice::from_raw_parts_mut(self.bins, self.numBins.try_into().unwrap()) }
    }

    pub fn get_bin_mut<'a>(self: &'a mut Histogram, bin_index: BinIndex) -> Option<&'a mut Bin> {
        match bin_index {
            BinIndex::Underflow => Some(&mut (self.underflow)),
            BinIndex::Overflow => Some(&mut (self.overflow)),
            BinIndex::Bin(index) => {
                if index >= self.numBins.try_into().unwrap() {
                    None
                } else {
                    Some(&mut self.get_bins_mut()[index])
                }
            }
        }
    }

    pub fn get_bin<'a>(self: &'a Histogram, bin_index: BinIndex) -> Option<&'a Bin> {
        match bin_index {
            BinIndex::Underflow => Some(&(self.underflow)),
            BinIndex::Overflow => Some(&(self.overflow)),
            BinIndex::Bin(index) => {
                if index >= self.numBins.try_into().unwrap() {
                    None
                } else {
                    Some(&self.get_bins()[index])
                }
            }
        }
    }

    pub fn get_bin_width(self: &Histogram) -> f64 {
        (self.right - self.left) / (self.numBins as f64)
    }

    pub fn get_bin_index(self: &Histogram, datum: f64) -> BinIndex {
        if datum < self.left {
            BinIndex::Underflow
        } else if datum >= self.right {
            BinIndex::Overflow
        } else {
            let index: usize = ((datum - self.left) / self.get_bin_width()) as usize;
            BinIndex::Bin(index)
        }
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

impl Index<BinIndex> for Histogram {
    type Output = Bin;

    fn index(&self, bin_index: BinIndex) -> &Self::Output {
        self.get_bin(bin_index).unwrap()
    }
}

impl IndexMut<BinIndex> for Histogram {
    fn index_mut(&mut self, bin_index: BinIndex) -> &mut Self::Output {
        self.get_bin_mut(bin_index).unwrap()
    }
}

#[no_mangle]
pub extern "C" fn bin(dataset: &DataSet, hist: &mut Histogram) {
    let samples_ref: &[f64] = dataset.get_samples();

    for &sample in samples_ref.iter() {
        hist.count(sample)
    }
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
    assert_eq!(1, hist[BinIndex::Bin(0)].count);
    assert_eq!(2, hist[BinIndex::Bin(1)].count);
    assert_eq!(0, hist[BinIndex::Bin(2)].count);
    assert_eq!(3, hist[BinIndex::Bin(3)].count);
    assert_eq!(1, hist[BinIndex::Bin(4)].count);
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
