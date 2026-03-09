package com.lamp.mobile.core.network

import android.util.Log
import okhttp3.Interceptor
import okhttp3.Response

/**
 * OkHttp interceptor that follows HTTP redirects while preserving the
 * original HTTP method. By default, OkHttp changes non-standard methods
 * (PROPFIND, REPORT) to GET on 301/302 redirects, which breaks
 * CalDAV/CardDAV .well-known discovery (RFC 6764).
 */
class MethodPreservingRedirectInterceptor(
    private val maxRedirects: Int = 5,
) : Interceptor {
    override fun intercept(chain: Interceptor.Chain): Response {
        var request = chain.request()
        var response = chain.proceed(request)
        var redirectCount = 0
        while (response.isRedirect && redirectCount < maxRedirects) {
            val location = response.header("Location") ?: break
            val newUrl = request.url.resolve(location) ?: break
            Log.d("RedirectInterceptor", "${request.method} ${request.url} -> ${response.code} -> $newUrl")
            response.close()
            request = request.newBuilder().url(newUrl).build()
            response = chain.proceed(request)
            redirectCount++
        }
        if (redirectCount >= maxRedirects && response.isRedirect) {
            response.close()
            throw java.io.IOException("Too many redirects (${maxRedirects}) for ${chain.request().method} ${chain.request().url}")
        }
        Log.d("RedirectInterceptor", "Final: ${request.method} ${request.url} -> ${response.code}")
        return response
    }
}
