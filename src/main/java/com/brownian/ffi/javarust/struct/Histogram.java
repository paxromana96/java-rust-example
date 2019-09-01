package com.brownian.ffi.javarust.struct;

import java.util.Arrays;
import java.util.List;
import java.util.stream.Stream;

import com.sun.jna.Structure;

public class Histogram extends Structure {
	public double left;
	public double right;
	public Bin underflow;
	public Bin overflow;
	public int numBins;
	public Bin.ByReference bins;

	public static class ByReference extends Histogram implements Structure.ByReference {
		public ByReference(double left, int numBins, double right) { super(left, numBins, right); }
	}

	@Override protected List<String> getFieldOrder() {
		return Arrays.asList("left", "right", "underflow", "overflow", "numBins", "bins");
	}

	public Histogram(double left, int numBins, double right) {
		if (numBins < 1) {
			throw new IllegalArgumentException("A Histogram must have at least 1 bin.");
		}

		this.left = left;
		this.right = right;
		this.underflow = new Bin();
		this.overflow = new Bin();
		this.numBins = numBins;
		this.bins = new Bin.ByReference();
		bins.toArray(numBins);
	}

	public Bin[] getBins() {
		return (Bin[]) this.bins.toArray(numBins);
	}

	@Override
	public String toString() {
		return String.format("%s %s %s", this.underflow, Arrays.toString(bins.toArray(this.numBins)), this.overflow);
	}

	public Bin getBin(double datum, Bin[] mainBins) {
		if (datum < left) return underflow;
		if (datum > right) return overflow;
		return mainBins[(int)((datum - left) / (right - left))];
	}

	public void binSimple(double[] data ) {
		Bin[] mainBins = getBins();
		for (double datum : data) {
			getBin(datum, mainBins).count++;
		}
	}

	public void binInline(double[] data ) {
		Bin[] mainBins = getBins();
		for (double datum : data) {
			if (datum < left) {
				underflow.count++;
			} else if (datum > right) {
				overflow.count++;
			} else {
				mainBins[(int)((datum - left) / (right - left))].count++;
			}
		}
	}

	public void binStreams(double [] data) {
		final Bin[] mainBins = getBins();
		Arrays.stream(data)
		.mapToObj(datum -> getBin(datum, mainBins))
		.forEach(bin -> bin.count++);
	}
}
