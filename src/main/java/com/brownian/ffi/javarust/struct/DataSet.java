package com.brownian.ffi.javarust.struct;

import java.util.Collections;
import java.util.List;

import com.sun.jna.Structure;

public class DataSet extends Structure
{
	public double[] samples;

	@Override protected List<String> getFieldOrder()
	{
		return Collections.singletonList("samples");
	}

	public DataSet(double[] samples){
		this.samples = samples;
	}
}
