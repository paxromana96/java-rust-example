package com.brownian.ffi.javarust;

import static org.junit.Assert.*;

import java.util.Arrays;
import java.util.stream.DoubleStream;
import java.util.stream.Stream;

import org.junit.Test;

import com.brownian.ffi.javarust.struct.Bin;
import com.brownian.ffi.javarust.struct.DataSet;
import com.brownian.ffi.javarust.struct.Histogram;

public class BinningTest
{
	@Test
	public void bin_withLinearData()
	{
		double[] data = DoubleStream.iterate(-0.5, d -> d + 0.5)
				.limit(22)
				.toArray();

		Bin[] expectedBins = Stream.generate(() -> new Bin(2)).limit(10).toArray(Bin[]::new);

		Histogram actualData = new Histogram(0.0, 10, 10.0);
		Binning.INSTANCE.bin(new DataSet(data), actualData);

		assertArrayEquals("Bins should be filled with the same data as expected", expectedBins, actualData.bins);
		assertEquals(new Bin(1), actualData.underflow);
		assertEquals(new Bin(1), actualData.overflow);
	}

	@Test
	public void increment_bin(){
		Bin bin = new Bin();
		Binning.INSTANCE.increment(bin);
		assertEquals(1, bin.count);
	}

	@Test
	public void count_sample_main_bins(){
		Histogram hist = new Histogram(0.0, 5, 5.0);
		Binning.INSTANCE.count_sample(hist, 1.2);
		assertArrayEquals(new int[] { 0, 1, 0, 0, 0}, Arrays.stream(hist.bins).mapToInt(b -> b.count).toArray());
	}

	@Test
	public void count_sample_underflow(){
		Histogram hist = new Histogram(0.0, 5, 5.0);
		Binning.INSTANCE.count_sample(hist, -1);
		assertEquals(1, hist.underflow.count);
	}
}