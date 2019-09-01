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

	@Test
	public void test_all_samples_are_counted() {
		double[] randomData = DoubleStream.generate(Math::random)
		                      .map(d -> 1.05 * Math.sin((d - 0.5) * Math.PI) / 2)
		                      .limit(10000)
		                      .toArray();
		DataSet.ByReference dataSet = new DataSet.ByReference(randomData);
		Histogram.ByReference hist = new Histogram.ByReference(-1.0, 17, 1.0);
		Binning.INSTANCE.bin(dataSet, hist);
		int total = hist.underflow.count + Arrays.stream(hist.getBins()).mapToInt(b -> b.count).sum() + hist.overflow.count;
		assertEquals(10000, total);

		drawHistogram(hist);
	}

	private void drawHistogram(Histogram hist) {
		int max = Math.max(Math.max(hist.overflow.count, hist.underflow.count),
		                   Arrays.stream(hist.getBins()).mapToInt(b -> b.count).max().orElse(0));

		drawHistBar(hist.underflow.count, max);
		System.out.printf("[%5.1f] %s%n", hist.left, "-------");
		for (Bin bin : hist.getBins()) {
			drawHistBar(bin.count, max);
		}
		System.out.printf("[%5.1f] %s%n", hist.right, "-------");
		drawHistBar(hist.overflow.count, max);
	}

	private int lerp(int val, int fromMax, int toMax) {
		return (val * toMax) / fromMax;
	}

	private void drawHistBar(int val, int fromMax) {
		int histVal = lerp(val, fromMax, 10);
		for (int i = 0 ; i < 10 ; i++) {
			System.out.print(i <= histVal ? "X" : " ");
		}
		System.out.println();
	}

	@Test
	public void benchmark_java_binning_vs_rust() {
		benchmark_java_binning_vs_rust(10000, 10);
		// benchmark_java_binning_vs_rust(1000, 10);
		// benchmark_java_binning_vs_rust(100, 10);

		benchmark_java_binning_vs_rust(10000, 100);
		// benchmark_java_binning_vs_rust(100, 100);
		// benchmark_java_binning_vs_rust(10, 100);

		benchmark_java_binning_vs_rust(10000, 1000);
		// benchmark_java_binning_vs_rust(100, 1000);
		// benchmark_java_binning_vs_rust(10, 1000);

		benchmark_java_binning_vs_rust(1000, 10000);
		// benchmark_java_binning_vs_rust(10, 10000);

		benchmark_java_binning_vs_rust(100, 100000);
	}

	public void benchmark_java_binning_vs_rust(int numRepititions, int numSamples) {

		double[] randomData = DoubleStream.generate(Math::random)
		                      .map(d -> 1.05 * Math.sin((d - 0.5) * Math.PI) / 2)
		                      .limit(numSamples)
		                      .toArray();

		long javaStartTime = System.currentTimeMillis();
		{
			for (int i = 0 ; i < numRepititions; i++) {
				Histogram jhist = new Histogram(0.0, 20, 1.0);
				jhist.bin(randomData);
			}
		}
		long javaEndTime = System.currentTimeMillis();
		long javaDuration = javaEndTime - javaStartTime;

		long rustStartTime = System.currentTimeMillis();
		{
			for (int i = 0 ; i < numRepititions; i++) {
				Histogram.ByReference hist = new Histogram.ByReference(0.0, 20, 1.0);
				DataSet.ByReference dataSet = new DataSet.ByReference(randomData);
				Binning.INSTANCE.bin(dataSet, hist);
			}
		}
		long rustEndTime = System.currentTimeMillis();
		long rustDuration = rustEndTime - rustStartTime;

		DataSet.ByReference noCopyDataSet = new DataSet.ByReference(randomData);
		long rustNoCopyStartTime = System.currentTimeMillis();
		{
			for (int i = 0 ; i < numRepititions; i++) {
				Histogram.ByReference hist = new Histogram.ByReference(0.0, 20, 1.0);
				Binning.INSTANCE.bin(noCopyDataSet, hist);
			}
		}
		long rustNoCopyEndTime = System.currentTimeMillis();
		long rustNoCopyDuration = rustNoCopyEndTime - rustNoCopyStartTime;

		System.out.printf("Binning %d samples with %d iterations:%n"
		                  + "Java:      %6.3fs: %3.3f ms each iteration, %3.3f us each sample%n"
		                  + "Rust:      %6.3fs: %3.3f ms each iteration, %3.3f us each sample%n"
		                  + "  no copy: %6.3fs: %3.3f ms each iteration, %3.3f us each sample%n%n",
		                  numSamples, numRepititions,
		                  javaDuration / 1000.0, ((double) javaDuration) / numRepititions, 1000.0 * javaDuration / numRepititions / numSamples,
		                  rustDuration / 1000.0, ((double) rustDuration) / numRepititions, 1000.0 * rustDuration / numRepititions / numSamples,
		                  rustNoCopyDuration / 1000.0, ((double) rustNoCopyDuration) / numRepititions, 1000.0 * rustNoCopyDuration / numRepititions / numSamples);
		// assertTrue("Java was faster than Rust with " + numSamples + " samples and " + numRepititions + " repititions",
		// javaDuration > rustDuration);

	}
}