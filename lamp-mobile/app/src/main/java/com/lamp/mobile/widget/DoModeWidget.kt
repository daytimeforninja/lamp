package com.lamp.mobile.widget

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.os.SystemClock
import android.view.View
import android.widget.RemoteViews
import androidx.room.Room
import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase
import com.lamp.mobile.R
import com.lamp.mobile.core.database.LampDatabase
import com.lamp.mobile.core.database.converter.EntityMappers
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.util.UUID

class DoModeWidget : AppWidgetProvider() {

    companion object {
        private const val ACTION_TOGGLE_TIMER = "com.lamp.mobile.TOGGLE_TIMER"
        private const val ACTION_COMPLETE_TASK = "com.lamp.mobile.COMPLETE_TASK"
        private const val ACTION_NAV_UP = "com.lamp.mobile.WIDGET_NAV_UP"
        private const val ACTION_NAV_DOWN = "com.lamp.mobile.WIDGET_NAV_DOWN"
        private const val PREFS_NAME = "lamp_widget"
        private const val KEY_TIMER_TASK_ID = "timer_task_id"
        private const val KEY_TIMER_START = "timer_start"
        private const val KEY_TIMER_ELAPSED_BASE = "timer_elapsed_base"
        private const val KEY_SELECTED_INDEX = "selected_index"

        @Volatile
        private var dbInstance: LampDatabase? = null

        fun refreshAll(context: Context) {
            val manager = AppWidgetManager.getInstance(context)
            val ids = manager.getAppWidgetIds(ComponentName(context, DoModeWidget::class.java))
            if (ids.isNotEmpty()) {
                val intent = Intent(context, DoModeWidget::class.java).apply {
                    action = AppWidgetManager.ACTION_APPWIDGET_UPDATE
                    putExtra(AppWidgetManager.EXTRA_APPWIDGET_IDS, ids)
                }
                context.sendBroadcast(intent)
            }
        }

        private fun prefs(context: Context): SharedPreferences =
            context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

        private fun getDb(context: Context): LampDatabase {
            return dbInstance ?: synchronized(this) {
                dbInstance ?: buildDb(context.applicationContext).also { dbInstance = it }
            }
        }

        private fun buildDb(context: Context): LampDatabase {
            val migration5to6 = object : Migration(5, 6) {
                override fun migrate(db: SupportSQLiteDatabase) {
                    db.execSQL("ALTER TABLE tasks ADD COLUMN dayplanDate TEXT")
                }
            }
            val migration6to7 = object : Migration(6, 7) {
                override fun migrate(db: SupportSQLiteDatabase) {
                    db.execSQL("ALTER TABLE contacts ADD COLUMN groups TEXT NOT NULL DEFAULT ''")
                    db.execSQL("UPDATE contacts SET groups = category WHERE category IS NOT NULL AND category != ''")
                }
            }
            val migration7to8 = object : Migration(7, 8) {
                override fun migrate(db: SupportSQLiteDatabase) {
                    db.execSQL("ALTER TABLE tasks ADD COLUMN clockEntries TEXT NOT NULL DEFAULT '[]'")
                }
            }
            return Room.databaseBuilder(context, LampDatabase::class.java, "lamp.db")
                .fallbackToDestructiveMigrationFrom(1, 2, 3, 4)
                .addMigrations(migration5to6, migration6to7, migration7to8)
                .enableMultiInstanceInvalidation()
                .build()
        }
    }

    override fun onReceive(context: Context, intent: Intent) {
        when (intent.action) {
            ACTION_TOGGLE_TIMER -> {
                CoroutineScope(Dispatchers.IO).launch {
                    toggleTimer(context)
                    refreshWidgetViews(context)
                }
                return
            }
            ACTION_COMPLETE_TASK -> {
                CoroutineScope(Dispatchers.IO).launch {
                    completeCurrentItem(context)
                    refreshWidgetViews(context)
                }
                return
            }
            ACTION_NAV_UP -> {
                CoroutineScope(Dispatchers.IO).launch {
                    navigate(context, -1)
                    refreshWidgetViews(context)
                }
                return
            }
            ACTION_NAV_DOWN -> {
                CoroutineScope(Dispatchers.IO).launch {
                    navigate(context, 1)
                    refreshWidgetViews(context)
                }
                return
            }
        }
        super.onReceive(context, intent)
    }

    override fun onUpdate(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetIds: IntArray,
    ) {
        CoroutineScope(Dispatchers.IO).launch {
            val data = loadWidgetData(context)
            for (id in appWidgetIds) {
                appWidgetManager.updateAppWidget(id, buildRemoteViews(context, data))
            }
        }
    }

    private suspend fun refreshWidgetViews(context: Context) {
        val manager = AppWidgetManager.getInstance(context)
        val ids = manager.getAppWidgetIds(ComponentName(context, DoModeWidget::class.java))
        val data = loadWidgetData(context)
        for (id in ids) {
            manager.updateAppWidget(id, buildRemoteViews(context, data))
        }
    }

    /** Build a unified list of items: confirmed tasks + due habits.
     *  Also cleans up stale DONE tasks from the day plan. */
    private suspend fun buildItemList(db: LampDatabase): List<WidgetItem> {
        val today = LocalDate.now()
        val todayStr = today.format(DateTimeFormatter.ISO_LOCAL_DATE)
        val planEntity = db.dayPlanDao().getByDate(todayStr)
        var plan = if (planEntity != null) EntityMappers.mapDayPlan(planEntity) else null

        val items = mutableListOf<WidgetItem>()
        val staleIds = mutableListOf<UUID>()

        // Confirmed tasks from day plan (skip already-done tasks)
        if (plan != null) {
            for (taskId in plan.confirmedTaskIds) {
                val entity = db.taskDao().getById(taskId.toString())
                if (entity != null) {
                    val task = EntityMappers.mapTask(entity)
                    if (!task.state.isDone) {
                        items.add(WidgetItem.TaskItem(task.id, task.title, task.esc, task.totalWorkSecs()))
                    } else {
                        staleIds.add(task.id)
                    }
                } else {
                    staleIds.add(taskId)
                }
            }

            // Clean up stale DONE tasks from the day plan
            if (staleIds.isNotEmpty()) {
                val cleaned = plan.copy(
                    confirmedTaskIds = plan.confirmedTaskIds.filter { it !in staleIds },
                )
                val cleanedEntity = with(EntityMappers) { cleaned.toEntity() }
                db.dayPlanDao().upsert(cleanedEntity)
                plan = cleaned
            }
        }

        // Due habits
        val allHabits = db.habitDao().getAll()
        for (habitEntity in allHabits) {
            val taskEntity = db.taskDao().getById(habitEntity.taskId) ?: continue
            val habit = EntityMappers.mapHabit(habitEntity, taskEntity)
            if (habit.isDue(today)) {
                items.add(WidgetItem.HabitItem(habit.task.id, habit.task.title, habit.task.totalWorkSecs()))
            }
        }

        return items
    }

    private fun navigate(context: Context, delta: Int) {
        val prefs = prefs(context)
        val current = prefs.getInt(KEY_SELECTED_INDEX, 0)
        // Index is clamped via .mod() when read, so just store the raw value
        prefs.edit().putInt(KEY_SELECTED_INDEX, (current + delta).coerceAtLeast(0)).apply()
    }

    private suspend fun stopAndLogTimer(context: Context, db: LampDatabase) {
        val prefs = prefs(context)
        val runningTaskId = prefs.getString(KEY_TIMER_TASK_ID, null)
        val startStr = prefs.getString(KEY_TIMER_START, null)

        if (runningTaskId != null && startStr != null) {
            val start = LocalDateTime.parse(startStr, DateTimeFormatter.ISO_LOCAL_DATE_TIME)
            val now = LocalDateTime.now()

            val entity = db.taskDao().getById(runningTaskId)
            if (entity != null) {
                val task = EntityMappers.mapTask(entity)
                val updated = task.copy(
                    clockEntries = task.clockEntries + (start to now),
                )
                val updatedEntity = with(EntityMappers) { updated.toEntity(syncDirty = true, location = entity.location) }
                db.taskDao().upsert(updatedEntity)
            }

            prefs.edit()
                .remove(KEY_TIMER_TASK_ID)
                .remove(KEY_TIMER_START)
                .remove(KEY_TIMER_ELAPSED_BASE)
                .apply()
        }
    }

    private suspend fun toggleTimer(context: Context) {
        val prefs = prefs(context)
        val runningTaskId = prefs.getString(KEY_TIMER_TASK_ID, null)
        val startStr = prefs.getString(KEY_TIMER_START, null)

        val db = getDb(context)
        if (runningTaskId != null && startStr != null) {
            stopAndLogTimer(context, db)
        } else {
            val items = buildItemList(db)
            if (items.isEmpty()) return
            val idx = prefs.getInt(KEY_SELECTED_INDEX, 0).mod(items.size)
            val item = items[idx]
            val priorSecs = when (item) {
                is WidgetItem.TaskItem -> item.priorWorkSecs
                is WidgetItem.HabitItem -> item.priorWorkSecs
            }
            prefs.edit()
                .putString(KEY_TIMER_TASK_ID, item.id.toString())
                .putString(KEY_TIMER_START, LocalDateTime.now().format(DateTimeFormatter.ISO_LOCAL_DATE_TIME))
                .putLong(KEY_TIMER_ELAPSED_BASE, SystemClock.elapsedRealtime() - (priorSecs * 1000))
                .apply()
        }
    }

    private suspend fun completeCurrentItem(context: Context) {
        val db = getDb(context)
        val items = buildItemList(db)
        if (items.isEmpty()) return
        val prefs = prefs(context)
        val idx = prefs.getInt(KEY_SELECTED_INDEX, 0).mod(items.size)
        val item = items[idx]

        // Stop timer if running
        stopAndLogTimer(context, db)

        when (item) {
            is WidgetItem.TaskItem -> {
                val entity = db.taskDao().getById(item.id.toString()) ?: return
                val task = EntityMappers.mapTask(entity)

                val completed = task.complete()
                val completedEntity = with(EntityMappers) { completed.toEntity(syncDirty = true, location = "archive") }
                db.taskDao().upsert(completedEntity)

                val todayStr = LocalDate.now().format(DateTimeFormatter.ISO_LOCAL_DATE)
                val planEntity = db.dayPlanDao().getByDate(todayStr)
                if (planEntity != null) {
                    val plan = EntityMappers.mapDayPlan(planEntity)
                    val updatedPlan = plan.completeTask(task.id, task.title, task.esc).let { p ->
                        if (p.confirmedTaskIds.contains(task.id)) {
                            p.copy(confirmedTaskIds = p.confirmedTaskIds - task.id)
                        } else p
                    }
                    val updatedPlanEntity = with(EntityMappers) { updatedPlan.toEntity() }
                    db.dayPlanDao().upsert(updatedPlanEntity)
                }
            }
            is WidgetItem.HabitItem -> {
                val habitEntity = db.habitDao().getByTaskId(item.id.toString()) ?: return
                val taskEntity = db.taskDao().getById(item.id.toString()) ?: return
                val habit = EntityMappers.mapHabit(habitEntity, taskEntity)
                val now = LocalDateTime.now()

                val updatedHabit = habit.copy(
                    completions = habit.completions + now,
                ).recalculateStreak(LocalDate.now())
                val updatedHabitEntity = with(EntityMappers) { updatedHabit.toEntity() }
                db.habitDao().upsert(updatedHabitEntity)

                val task = EntityMappers.mapTask(taskEntity)
                val updatedTask = task.copy(logbookEntries = task.logbookEntries + now)
                val updatedTaskEntity = with(EntityMappers) { updatedTask.toEntity(syncDirty = true, location = taskEntity.location) }
                db.taskDao().upsert(updatedTaskEntity)
            }
        }

        // Clamp index to new list size
        val newItems = buildItemList(db)
        if (newItems.isEmpty()) {
            prefs.edit().putInt(KEY_SELECTED_INDEX, 0).apply()
        } else {
            val newIdx = idx.coerceAtMost(newItems.size - 1)
            prefs.edit().putInt(KEY_SELECTED_INDEX, newIdx).apply()
        }
    }

    private suspend fun loadWidgetData(context: Context): WidgetData {
        val prefs = prefs(context)
        val runningTaskId = prefs.getString(KEY_TIMER_TASK_ID, null)
        val startStr = prefs.getString(KEY_TIMER_START, null)

        val db = getDb(context)
        val items = buildItemList(db)

        if (items.isEmpty()) {
            val todayStr = LocalDate.now().format(DateTimeFormatter.ISO_LOCAL_DATE)
            val planEntity = db.dayPlanDao().getByDate(todayStr)
            val plan = if (planEntity != null) EntityMappers.mapDayPlan(planEntity) else null
            val allDone = plan != null && plan.completedTasks.isNotEmpty()
            return WidgetData(
                currentItem = null,
                itemCount = 0,
                currentIndex = 0,
                isTimerRunning = false,
                chronometerBase = 0L,
                allDone = allDone,
            )
        }

        val idx = prefs.getInt(KEY_SELECTED_INDEX, 0).mod(items.size)
        val currentItem = items[idx]

        val isTimerRunning = runningTaskId != null && startStr != null
            && runningTaskId == currentItem.id.toString()
        val chronometerBase = if (isTimerRunning) {
            prefs.getLong(KEY_TIMER_ELAPSED_BASE, SystemClock.elapsedRealtime())
        } else 0L

        return WidgetData(
            currentItem = currentItem,
            itemCount = items.size,
            currentIndex = idx,
            isTimerRunning = isTimerRunning,
            chronometerBase = chronometerBase,
            allDone = false,
        )
    }

    private fun buildRemoteViews(context: Context, data: WidgetData): RemoteViews {
        val views = RemoteViews(context.packageName, R.layout.widget_do_mode)

        // Tap task title → toggle timer
        val toggleIntent = Intent(context, DoModeWidget::class.java).apply { action = ACTION_TOGGLE_TIMER }
        val togglePi = PendingIntent.getBroadcast(context, 0, toggleIntent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        views.setOnClickPendingIntent(R.id.widget_task_title, togglePi)
        views.setOnClickPendingIntent(R.id.widget_timer, togglePi)

        // Dharma wheel → complete
        val completeIntent = Intent(context, DoModeWidget::class.java).apply { action = ACTION_COMPLETE_TASK }
        val completePi = PendingIntent.getBroadcast(context, 2, completeIntent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        views.setOnClickPendingIntent(R.id.widget_done_btn, completePi)

        // Nav up
        val upIntent = Intent(context, DoModeWidget::class.java).apply { action = ACTION_NAV_UP }
        val upPi = PendingIntent.getBroadcast(context, 3, upIntent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        views.setOnClickPendingIntent(R.id.widget_nav_up, upPi)

        // Nav down
        val downIntent = Intent(context, DoModeWidget::class.java).apply { action = ACTION_NAV_DOWN }
        val downPi = PendingIntent.getBroadcast(context, 4, downIntent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        views.setOnClickPendingIntent(R.id.widget_nav_down, downPi)

        val item = data.currentItem
        when {
            item != null -> {
                val prefix = when {
                    item is WidgetItem.HabitItem -> "\u0F12 "
                    data.isTimerRunning -> "\u0F14 "
                    else -> "\u0F04 "
                }
                views.setTextViewText(R.id.widget_task_title, prefix + item.title)
                views.setViewVisibility(R.id.widget_done_btn, View.VISIBLE)

                val showNav = data.itemCount > 1
                views.setViewVisibility(R.id.widget_nav_up, if (showNav) View.VISIBLE else View.GONE)
                views.setViewVisibility(R.id.widget_nav_down, if (showNav) View.VISIBLE else View.GONE)

                val priorSecs = when (item) {
                    is WidgetItem.TaskItem -> item.priorWorkSecs
                    is WidgetItem.HabitItem -> item.priorWorkSecs
                }
                if (data.isTimerRunning) {
                    views.setChronometer(R.id.widget_timer, data.chronometerBase, null, true)
                    views.setViewVisibility(R.id.widget_timer, View.VISIBLE)
                } else if (priorSecs > 0) {
                    val base = SystemClock.elapsedRealtime() - (priorSecs * 1000)
                    views.setChronometer(R.id.widget_timer, base, null, false)
                    views.setViewVisibility(R.id.widget_timer, View.VISIBLE)
                } else {
                    views.setViewVisibility(R.id.widget_timer, View.GONE)
                }
            }
            data.allDone -> {
                views.setTextViewText(R.id.widget_task_title, "\u0F0B\u0F0B\u0F0B All done \u0F0B\u0F0B\u0F0B")
                views.setViewVisibility(R.id.widget_timer, View.GONE)
                views.setViewVisibility(R.id.widget_done_btn, View.GONE)
                views.setViewVisibility(R.id.widget_nav_up, View.GONE)
                views.setViewVisibility(R.id.widget_nav_down, View.GONE)
            }
            else -> {
                views.setTextViewText(R.id.widget_task_title, "\u0F04 No tasks")
                views.setViewVisibility(R.id.widget_timer, View.GONE)
                views.setViewVisibility(R.id.widget_done_btn, View.GONE)
                views.setViewVisibility(R.id.widget_nav_up, View.GONE)
                views.setViewVisibility(R.id.widget_nav_down, View.GONE)
            }
        }

        return views
    }
}

sealed class WidgetItem {
    abstract val id: UUID
    abstract val title: String

    data class TaskItem(
        override val id: UUID,
        override val title: String,
        val esc: Int?,
        val priorWorkSecs: Long,
    ) : WidgetItem()

    data class HabitItem(
        override val id: UUID,
        override val title: String,
        val priorWorkSecs: Long,
    ) : WidgetItem()
}

private data class WidgetData(
    val currentItem: WidgetItem?,
    val itemCount: Int,
    val currentIndex: Int,
    val isTimerRunning: Boolean,
    val chronometerBase: Long,
    val allDone: Boolean,
)
