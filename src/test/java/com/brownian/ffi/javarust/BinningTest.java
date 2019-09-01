package com.brownian.ffi.javarust;

import static org.junit.Assert.*;

import java.util.Arrays;
import java.util.stream.DoubleStream;
import java.util.stream.Stream;

import org.junit.Test;

import com.brownian.ffi.javarust.struct.Bin;
import com.brownian.ffi.javarust.struct.DataSet;
import com.brownian.ffi.javarust.struct.Histogram;

import com.sun.jna.Structure;

public class BinningTest {
	@Test
	public void bin_withLinearData() {
		double[] data = DoubleStream.iterate(-0.5, d -> d + 0.5)
		                .limit(22)
		                .toArray();

		Bin[] expectedBins = Stream.generate(() -> new Bin(2)).limit(10).toArray(Bin[]::new);

		Histogram.ByReference hist = new Histogram.ByReference(0.0, 10, 10.0);
		Binning.INSTANCE.bin(new DataSet.ByReference(data), hist);

		Bin[] actualBins = hist.getBins();
		assertArrayEquals("Bins should be filled with the same data as expected", expectedBins, actualBins);
		assertEquals(new Bin(1), hist.underflow);
		assertEquals(new Bin(1), hist.overflow);
	}

	@Test
	public void test_canPassDataSet() {
		double[] data = { -1.0, 2.0, 3.0, 4.0 };
		double expectedTotal = Arrays.stream(data).sum();
		DataSet.ByReference dataSet = new DataSet.ByReference(data);
		assertEquals(expectedTotal, Binning.INSTANCE.sum_samples(dataSet), 0.001);
	}

	@Test
	public void increment_bin() {
		Bin bin = new Bin();
		Binning.INSTANCE.increment(bin);
		assertEquals(1, bin.count);
	}

	@Test
	public void count_sample_main_bins() {
		Histogram.ByReference hist = new Histogram.ByReference(0.0, 5, 5.0);
		Binning.INSTANCE.count_sample(hist, 1.2);
		assertArrayEquals(new int[] { 0, 1, 0, 0, 0}, Arrays.stream(hist.getBins()).mapToInt(b -> b.count).toArray());
	}

	@Test
	public void count_sample_underflow() {
		Histogram.ByReference hist = new Histogram.ByReference(0.0, 5, 5.0);
		Binning.INSTANCE.count_sample(hist, -1);
		assertEquals(1, hist.underflow.count);
		assertEquals(0, hist.overflow.count);
		for (Bin bin : hist.getBins()) {
			assertEquals(0, bin.count);
		}
	}

	@Test
	public void count_sample_overflow() {
		Histogram.ByReference hist = new Histogram.ByReference(0.0, 5, 5.0);
		Binning.INSTANCE.count_sample(hist, 6);
		assertEquals(0, hist.underflow.count);
		assertEquals(1, hist.overflow.count);
		for (Bin bin : hist.getBins()) {
			assertEquals(0, bin.count);
		}
	}

	@Test
	public void increment_all() {
		Bin[] bins = (Bin[]) (new Bin()).toArray(4);
		Binning.INSTANCE.increment_all(bins, bins.length);
		for (Bin bin : bins) {
			assertEquals(1, bin.count);
		}
	}
}