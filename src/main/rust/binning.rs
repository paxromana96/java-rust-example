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

#[cfg(test)]
use rand::Rng;

// Corresponds to com.brownian.ffi.javarust.struct.DataSet
// Marked with repr(C), like all JNI structs,
// because that's how JNA can read the fields.
#[repr(C)]
pub struct DataSet {
    samples: Box<[f64]>,
}

impl DataSet {
    pub fn new() -> DataSet {
        DataSet { samples: vec!().into_boxed_slice() }
    }
}

impl From<Box<[f64]>> for DataSet {
    fn from(data: Box<[f64]>) -> DataSet {
        DataSet { samples: data }
    }
}

impl From<Vec<f64>> for DataSet {
    fn from(data: Vec<f64>) -> DataSet {
        DataSet { samples: data.into_boxed_slice() }
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

#[repr(C)]
pub struct Histogram {
    underflow: Bin,
    left: f64,
    bins: Box<[Bin]>,
    right: f64,
    overflow: Bin,
}

enum BinIndex {
    Underflow,
    Bin(usize),
    Overflow,
}

impl Histogram {
    pub fn count(self: &mut Histogram, datum: f64) {
        let bin: &mut Bin = match self.get_bin_index(datum) {
            BinIndex::Underflow => &mut (self.underflow),
            BinIndex::Overflow => &mut (self.overflow),
            BinIndex::Bin(index) => &mut (self.bins[index])
        };

        bin.increment()
    }

    fn get_bin_width(self: &Histogram) -> f64 {
        (self.right - self.left) / (self.bins.len() as f64)
    }

    fn get_bin_index(self: &Histogram, datum: f64) -> BinIndex {
        if datum < self.left {
            BinIndex::Underflow
        } else if datum >= self.right {
            BinIndex::Overflow
        } else {
            let index: usize = ((datum - self.left) / self.get_bin_width()) as usize;
            BinIndex::Bin(index)
        }
    }

    fn new(left: f64, num_bins: usize, right: f64) -> Histogram {
        Histogram {
            underflow: Bin::empty(),
            left,
            bins: std::iter::repeat_with(Bin::empty).take(num_bins).collect::<Vec<_>>().into_boxed_slice(),
            right,
            overflow: Bin::empty(),
        }
    }
}

#[no_mangle]
pub extern fn bin(dataset: &DataSet, hist: &mut Histogram) {
    let samples_ref: &[f64] = &*dataset.samples;
    for &sample in samples_ref.iter() {
        hist.count(sample)
    }
}

#[no_mangle]
pub extern fn increment(bin: &mut Bin) {
    bin.increment()
}

#[no_mangle]
pub extern fn count_sample(hist: &mut Histogram, sample: f64) {
    hist.count(sample)
}

#[test]
fn test_bin_linear_dataset() {
    let dataset = DataSet::from(vec![-0.5, 0.5, 1.5, 1.7, 3.1, 3.5, 3.6, 4.5, 5.5]);
    let mut hist = Histogram::new(0.0, 5, 5.0);
    bin(&dataset, &mut hist);

    assert_eq!(1, hist.underflow.count);
    assert_eq!(1, hist.bins[0].count);
    assert_eq!(2, hist.bins[1].count);
    assert_eq!(0, hist.bins[2].count);
    assert_eq!(3, hist.bins[3].count);
    assert_eq!(1, hist.bins[4].count);
    assert_eq!(1, hist.overflow.count);
}

#[test]
fn test_bin_random_dataset_counts_all() {
    let mut rng = rand::thread_rng();
    let dataset = DataSet::from(std::iter::repeat_with(|| rng.gen()).take(1000).collect::<Vec<_>>());
    let mut hist = Histogram::new(0.0, 1000, 1.0);
    bin(&dataset, &mut hist);

    assert_eq!(0, hist.underflow.count);
    assert_eq!(0, hist.overflow.count);
    assert_eq!(1000, hist.bins.iter().map(|b| b.count).sum::<u32>());
}