package com.lamp.mobile.feature.accounts

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.AccountRepository
import com.lamp.mobile.core.model.Account
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.time.LocalDate
import java.util.UUID
import javax.inject.Inject

data class AccountsUiState(
    val accounts: List<Account> = emptyList(),
    val inputText: String = "",
    val expandedId: UUID? = null,
)

sealed class AccountsIntent {
    data class InputChanged(val text: String) : AccountsIntent()
    data object Submit : AccountsIntent()
    data class ToggleExpanded(val id: UUID) : AccountsIntent()
    data class UpdateName(val id: UUID, val name: String) : AccountsIntent()
    data class UpdateUrl(val id: UUID, val url: String) : AccountsIntent()
    data class UpdateNotes(val id: UUID, val notes: String) : AccountsIntent()
    data class MarkChecked(val id: UUID) : AccountsIntent()
    data class Delete(val id: UUID) : AccountsIntent()
}

@HiltViewModel
class AccountsViewModel @Inject constructor(
    private val accountRepo: AccountRepository,
) : MviViewModel<AccountsUiState, AccountsIntent, Nothing>(AccountsUiState()) {

    init {
        accountRepo.observeAll()
            .onEach { accounts -> updateState { copy(accounts = accounts) } }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: AccountsIntent) {
        when (intent) {
            is AccountsIntent.InputChanged -> updateState { copy(inputText = intent.text) }
            is AccountsIntent.Submit -> {
                val name = currentState.inputText.trim()
                if (name.isNotEmpty()) {
                    accountRepo.save(Account(name = name))
                    updateState { copy(inputText = "") }
                }
            }
            is AccountsIntent.ToggleExpanded -> {
                updateState {
                    copy(expandedId = if (expandedId == intent.id) null else intent.id)
                }
            }
            is AccountsIntent.UpdateName -> {
                val acc = accountRepo.getById(intent.id) ?: return
                accountRepo.save(acc.copy(name = intent.name))
            }
            is AccountsIntent.UpdateUrl -> {
                val acc = accountRepo.getById(intent.id) ?: return
                accountRepo.save(acc.copy(url = intent.url))
            }
            is AccountsIntent.UpdateNotes -> {
                val acc = accountRepo.getById(intent.id) ?: return
                accountRepo.save(acc.copy(notes = intent.notes))
            }
            is AccountsIntent.MarkChecked -> {
                val acc = accountRepo.getById(intent.id) ?: return
                accountRepo.save(acc.copy(lastChecked = LocalDate.now()))
            }
            is AccountsIntent.Delete -> accountRepo.delete(intent.id)
        }
    }
}
