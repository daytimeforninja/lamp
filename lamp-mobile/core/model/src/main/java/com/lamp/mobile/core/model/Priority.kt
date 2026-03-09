package com.lamp.mobile.core.model

enum class Priority(val orgString: String, val icalPriority: Int) {
    A("[#A]", 1),
    B("[#B]", 5),
    C("[#C]", 9);

    companion object {
        fun fromOrg(s: String): Priority? = when (s.trim().uppercase().replace("[#", "").replace("]", "").replace("#", "")) {
            "A" -> A
            "B" -> B
            "C" -> C
            else -> null
        }

        fun fromIcalPriority(p: Int): Priority? = when (p) {
            1 -> A
            in 2..5 -> B
            in 6..9 -> C
            else -> null
        }
    }
}
