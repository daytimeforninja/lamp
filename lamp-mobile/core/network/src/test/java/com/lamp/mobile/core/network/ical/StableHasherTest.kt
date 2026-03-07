package com.lamp.mobile.core.network.ical

import org.junit.Assert.*
import org.junit.Test

class StableHasherTest {

    @Test
    fun `empty hash returns offset basis`() {
        val hasher = StableHasher()
        // FNV-1a offset: 0xcbf29ce484222325
        assertEquals(-0x340d631b7bdddcdbL, hasher.finish())
    }

    @Test
    fun `hash is deterministic`() {
        val h1 = StableHasher().apply { writeString("hello") }.finish()
        val h2 = StableHasher().apply { writeString("hello") }.finish()
        assertEquals(h1, h2)
    }

    @Test
    fun `different strings produce different hashes`() {
        val h1 = StableHasher().apply { writeString("hello") }.finish()
        val h2 = StableHasher().apply { writeString("world") }.finish()
        assertNotEquals(h1, h2)
    }

    @Test
    fun `optional string none vs some differ`() {
        val h1 = StableHasher().apply { writeOptionalString(null) }.finish()
        val h2 = StableHasher().apply { writeOptionalString("test") }.finish()
        assertNotEquals(h1, h2)
    }
}
