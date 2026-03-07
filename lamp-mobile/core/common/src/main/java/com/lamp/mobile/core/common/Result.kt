package com.lamp.mobile.core.common

/**
 * A simple result wrapper for async operations.
 */
sealed class LampResult<out T> {
    data class Success<T>(val data: T) : LampResult<T>()
    data class Error(val message: String, val cause: Throwable? = null) : LampResult<Nothing>()
    data object Loading : LampResult<Nothing>()

    val isSuccess: Boolean get() = this is Success
    val isError: Boolean get() = this is Error
    val isLoading: Boolean get() = this is Loading

    fun getOrNull(): T? = (this as? Success)?.data

    fun <R> map(transform: (T) -> R): LampResult<R> = when (this) {
        is Success -> Success(transform(data))
        is Error -> this
        is Loading -> Loading
    }
}
