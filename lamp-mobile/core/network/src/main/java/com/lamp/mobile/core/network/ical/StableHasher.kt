package com.lamp.mobile.core.network.ical

/**
 * FNV-1a 64-bit hasher matching the desktop Rust implementation exactly.
 * Offset basis: 0xcbf29ce484222325
 * FNV prime: 0x100000001b3
 */
class StableHasher {
    private var hash: Long = -0x340d631b7bdddcdbL // 0xcbf29ce484222325 as signed Long

    fun write(bytes: ByteArray) {
        for (b in bytes) {
            hash = hash xor (b.toLong() and 0xFF)
            hash *= 0x100000001b3L
        }
    }

    fun writeString(s: String) = write(s.toByteArray(Charsets.UTF_8))

    fun writeOptionalString(s: String?) {
        if (s != null) {
            write(byteArrayOf(1))
            writeString(s)
        } else {
            write(byteArrayOf(0))
        }
    }

    fun writeOptionalInt(v: Int?) {
        if (v != null) {
            write(byteArrayOf(1))
            writeString(v.toString())
        } else {
            write(byteArrayOf(0))
        }
    }

    fun finish(): Long = hash
}
