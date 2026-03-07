package com.lamp.mobile.core.database.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "habits")
data class HabitEntity(
    @PrimaryKey val taskId: String,
    val completions: String, // JSON array of ISO datetimes
    val streak: Int,
    val bestStreak: Int,
)
