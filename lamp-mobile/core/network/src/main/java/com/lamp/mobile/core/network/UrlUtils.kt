package com.lamp.mobile.core.network

/**
 * Upgrades http:// URLs to https:// to prevent cleartext credential transmission.
 * Android blocks cleartext traffic by default (API 28+).
 */
fun String.upgradeToHttps(): String =
    if (startsWith("http://")) replaceFirst("http://", "https://") else this
