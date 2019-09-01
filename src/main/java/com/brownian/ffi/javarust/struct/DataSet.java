package com.brownian.ffi.javarust.struct;

import java.util.Arrays;
import java.util.List;

import com.sun.jna.Memory;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;


public class DataSet extends Structure {
	public Pointer samples;
	public int numSamples;

	public static class ByReference extends DataSet implements Structure.ByReference {
		public ByReference(double[] samples) { super(samples); }
	}

	@Override protected List<String> getFieldOrder() {
		return Arrays.asList("samples", "numSamples");
	}

	public DataSet(double[] samples) {
		this.samples = new Memory(samples.length * Native.getNativeSize(Double.TYPE));
		this.numSamples = samples.length;
		for (int i = 0 ; i < numSamples ; i++) {
			this.samples.setDouble(i * Native.getNativeSize(Double.TYPE), samples[i]);
		}
	}
}
