package com.lamp.mobile.feature.lists

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.ListItemRepository
import com.lamp.mobile.core.model.ListItem
import com.lamp.mobile.core.model.ListKind
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.util.UUID
import javax.inject.Inject

data class ListsUiState(
    val items: List<ListItem> = emptyList(),
    val inputText: String = "",
    val kind: ListKind = ListKind.MEDIA,
)

sealed class ListsIntent {
    data class InputChanged(val text: String) : ListsIntent()
    data object Submit : ListsIntent()
    data class ToggleDone(val itemId: UUID) : ListsIntent()
    data class Delete(val itemId: UUID) : ListsIntent()
    data class SetKind(val kind: ListKind) : ListsIntent()
}

@HiltViewModel
class ListsViewModel @Inject constructor(
    private val listItemRepo: ListItemRepository,
) : MviViewModel<ListsUiState, ListsIntent, Nothing>(ListsUiState()) {

    init {
        observeKind(ListKind.MEDIA)
    }

    private fun observeKind(kind: ListKind) {
        listItemRepo.observeByKind(kind)
            .onEach { items -> updateState { copy(items = items) } }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: ListsIntent) {
        when (intent) {
            is ListsIntent.InputChanged -> updateState { copy(inputText = intent.text) }
            is ListsIntent.Submit -> {
                val title = currentState.inputText.trim()
                if (title.isNotEmpty()) {
                    listItemRepo.save(ListItem(title = title, kind = currentState.kind))
                    updateState { copy(inputText = "") }
                }
            }
            is ListsIntent.ToggleDone -> {
                val item = listItemRepo.getById(intent.itemId) ?: return
                listItemRepo.save(item.copy(done = !item.done))
            }
            is ListsIntent.Delete -> listItemRepo.delete(intent.itemId)
            is ListsIntent.SetKind -> {
                updateState { copy(kind = intent.kind) }
                observeKind(intent.kind)
            }
        }
    }
}
