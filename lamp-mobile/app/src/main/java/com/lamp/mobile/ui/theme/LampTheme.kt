package com.lamp.mobile.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext

private val LightColorScheme = lightColorScheme(
    primary = Color(0xFF4A6741),
    onPrimary = Color.White,
    primaryContainer = Color(0xFFCBEFBD),
    onPrimaryContainer = Color(0xFF072100),
    secondary = Color(0xFF526350),
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFD5E8D0),
    onSecondaryContainer = Color(0xFF101F10),
    tertiary = Color(0xFF39656B),
    onTertiary = Color.White,
    tertiaryContainer = Color(0xFFBDEBF2),
    onTertiaryContainer = Color(0xFF001F23),
    error = Color(0xFFBA1A1A),
    surface = Color(0xFFF7FBF1),
    onSurface = Color(0xFF181D17),
)

private val DarkColorScheme = darkColorScheme(
    primary = Color(0xFFB0D3A3),
    onPrimary = Color(0xFF1D3714),
    primaryContainer = Color(0xFF334F2B),
    onPrimaryContainer = Color(0xFFCBEFBD),
    secondary = Color(0xFFB9CCB5),
    onSecondary = Color(0xFF253424),
    secondaryContainer = Color(0xFF3B4B39),
    onSecondaryContainer = Color(0xFFD5E8D0),
    tertiary = Color(0xFFA1CFD5),
    onTertiary = Color(0xFF00363C),
    tertiaryContainer = Color(0xFF1F4D53),
    onTertiaryContainer = Color(0xFFBDEBF2),
    error = Color(0xFFFFB4AB),
    surface = Color(0xFF10150F),
    onSurface = Color(0xFFE0E4DA),
)

@Composable
fun LampTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    dynamicColor: Boolean = true,
    content: @Composable () -> Unit,
) {
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }
        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }

    MaterialTheme(
        colorScheme = colorScheme,
        content = content,
    )
}
