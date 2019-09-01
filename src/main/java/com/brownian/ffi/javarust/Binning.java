package com.brownian.ffi.javarust;

import com.brownian.ffi.javarust.struct.Bin;
import com.brownian.ffi.javarust.struct.DataSet;
import com.brownian.ffi.javarust.struct.Histogram;
import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.NativeLibrary;

public interface Binning extends Library {
	String JNA_LIBRARY_NAME = "binning";
	NativeLibrary JNA_NATIVE_LIB = NativeLibrary.getInstance(JNA_LIBRARY_NAME);

	Binning INSTANCE = Native.loadLibrary(JNA_LIBRARY_NAME, Binning.class);

	void bin(DataSet.ByReference sample, Histogram.ByReference histogram);
	void increment(Bin bin);
	void count_sample(Histogram .ByReference hist, double sample);
	void increment_all(Bin[] bin, int num_bins);
	double sum_samples(DataSet.ByReference dataset);
}
