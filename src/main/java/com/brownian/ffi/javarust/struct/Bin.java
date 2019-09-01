package com.brownian.ffi.javarust.struct;

import java.util.Collections;
import java.util.List;

import com.sun.jna.Structure;

@SuppressWarnings("unused")
public class Bin extends Structure {

	public static class ByReference extends Bin implements Structure.ByReference {}

	public int count;

	@Override protected List<String> getFieldOrder() {
		return Collections.singletonList("count");
	}

	public Bin() {
		this(0);
	}

	public Bin(int initialCount) {
		this.count = initialCount;
	}

	@Override
	public boolean equals(Object other) {
		return (other instanceof Bin) && ((Bin) other).count == this.count;
	}

	@Override
	public int hashCode() {
		return Integer.hashCode(this.count);
	}

	@Override
	public String toString() {
		return String.valueOf(this.count);
	}
}
