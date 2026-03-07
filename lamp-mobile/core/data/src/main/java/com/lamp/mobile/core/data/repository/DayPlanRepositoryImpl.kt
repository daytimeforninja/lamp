package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.database.converter.EntityMappers.toDomain
import com.lamp.mobile.core.database.converter.EntityMappers.toEntity
import com.lamp.mobile.core.database.dao.DayPlanDao
import com.lamp.mobile.core.model.DayPlan
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class DayPlanRepositoryImpl @Inject constructor(
    private val dayPlanDao: DayPlanDao,
) : DayPlanRepository {

    override fun observeByDate(date: LocalDate): Flow<DayPlan?> =
        dayPlanDao.observeByDate(date.format(DateTimeFormatter.ISO_LOCAL_DATE))
            .map { it?.toDomain() }

    override suspend fun getByDate(date: LocalDate): DayPlan? =
        dayPlanDao.getByDate(date.format(DateTimeFormatter.ISO_LOCAL_DATE))?.toDomain()

    override suspend fun save(dayPlan: DayPlan) =
        dayPlanDao.upsert(dayPlan.toEntity())
}
