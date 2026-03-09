package com.lamp.mobile.feature.habits

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.HabitRepository
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.Habit
import com.lamp.mobile.core.model.Task
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.time.LocalDate
import java.time.LocalDateTime
import java.util.UUID
import javax.inject.Inject

data class HabitsUiState(
    val habits: List<Habit> = emptyList(),
    val inputText: String = "",
)

sealed class HabitsIntent {
    data class InputChanged(val text: String) : HabitsIntent()
    data object Submit : HabitsIntent()
    data class Complete(val taskId: UUID) : HabitsIntent()
    data class Delete(val taskId: UUID) : HabitsIntent()
}

@HiltViewModel
class HabitsViewModel @Inject constructor(
    private val habitRepo: HabitRepository,
    private val taskRepo: TaskRepository,
) : MviViewModel<HabitsUiState, HabitsIntent, Nothing>(HabitsUiState()) {

    init {
        habitRepo.observeAll()
            .onEach { habits ->
                val today = LocalDate.now()
                val sorted = habits.sortedWith(
                    compareByDescending<Habit> { it.isDue(today) }
                        .thenByDescending { it.streak }
                )
                updateState { copy(habits = sorted) }
            }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: HabitsIntent) {
        when (intent) {
            is HabitsIntent.InputChanged -> updateState { copy(inputText = intent.text) }
            is HabitsIntent.Submit -> {
                val title = currentState.inputText.trim()
                if (title.isNotEmpty()) {
                    val task = Task(title = title)
                    habitRepo.save(Habit(task = task))
                    updateState { copy(inputText = "") }
                }
            }
            is HabitsIntent.Complete -> {
                val habit = habitRepo.getByTaskId(intent.taskId) ?: return
                val updated = habit.copy(
                    completions = habit.completions + LocalDateTime.now()
                ).recalculateStreak(LocalDate.now())
                habitRepo.save(updated)
            }
            is HabitsIntent.Delete -> habitRepo.delete(intent.taskId)
        }
    }
}
