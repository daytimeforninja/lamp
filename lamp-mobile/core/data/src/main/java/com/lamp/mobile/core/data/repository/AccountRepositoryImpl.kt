package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.database.converter.EntityMappers.toDomain
import com.lamp.mobile.core.database.converter.EntityMappers.toEntity
import com.lamp.mobile.core.database.dao.AccountDao
import com.lamp.mobile.core.model.Account
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AccountRepositoryImpl @Inject constructor(
    private val accountDao: AccountDao,
) : AccountRepository {

    override fun observeAll(): Flow<List<Account>> =
        accountDao.observeAll().map { list -> list.map { it.toDomain() } }

    override suspend fun getById(id: UUID): Account? =
        accountDao.getById(id.toString())?.toDomain()

    override suspend fun save(account: Account) =
        accountDao.upsert(account.toEntity())

    override suspend fun delete(id: UUID) =
        accountDao.deleteById(id.toString())

    override suspend fun getAll(): List<Account> =
        accountDao.getAll().map { it.toDomain() }
}
