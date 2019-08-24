package com.brownian.ffi.javarust.struct;

import java.util.Arrays;
import java.util.List;
import java.util.stream.Stream;

import com.sun.jna.Structure;

public class Histogram extends Structure
{
	public Bin underflow;
	public double left;
	public Bin[] bins;
	public double right;
	public Bin overflow;

	@Override protected List<String> getFieldOrder()
	{
		return Arrays.asList("underflow", "left", "bins", "right", "overflow");
	}

	public Histogram(double left, int numBins, double right){
		this.underflow = new Bin();
		this.left = left;
		this.bins = Stream.generate(Bin::new).limit(numBins).toArray(Bin[]::new);
		this.right = right;
		this.overflow = new Bin();
	}

	@Override
	public String toString(){
		return String.format("%s %s %s", this.underflow, Arrays.toString(bins), this.overflow);
	}
}
