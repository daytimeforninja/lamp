package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.model.Account
import kotlinx.coroutines.flow.Flow
import java.util.UUID

interface AccountRepository {
    fun observeAll(): Flow<List<Account>>
    suspend fun getById(id: UUID): Account?
    suspend fun save(account: Account)
    suspend fun delete(id: UUID)
    suspend fun getAll(): List<Account>
}
