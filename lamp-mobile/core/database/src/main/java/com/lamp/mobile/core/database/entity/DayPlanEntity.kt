package com.lamp.mobile.core.database.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "day_plans")
data class DayPlanEntity(
    @PrimaryKey val date: String, // ISO date
    val spoonBudget: Int,
    val activeContexts: String, // JSON array
    val confirmedTaskIds: String, // JSON array of UUIDs
    val completedTasks: String, // JSON array of {id, title, esc}
    val spentSpoons: Int,
    val pickedMediaIds: String, // JSON array of UUIDs
    val pickedShoppingIds: String, // JSON array of UUIDs
)
