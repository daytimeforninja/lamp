package com.lamp.mobile

import android.Manifest
import android.content.pm.PackageManager
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.lamp.mobile.core.common.ui.LocalSyncTrigger
import com.lamp.mobile.core.model.AppMode
import com.lamp.mobile.navigation.DrawerDestination
import com.lamp.mobile.navigation.LampNavHost
import com.lamp.mobile.navigation.TopLevelDestination
import com.lamp.mobile.sync.SyncWorker
import com.lamp.mobile.ui.theme.LampTheme
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.launch

@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        requestContactsPermissionIfNeeded()
        setContent {
            LampTheme {
                LampApp()
            }
        }
    }

    private fun requestContactsPermissionIfNeeded() {
        val perms = arrayOf(
            Manifest.permission.READ_CONTACTS,
            Manifest.permission.WRITE_CONTACTS,
        )
        val needed = perms.filter {
            ContextCompat.checkSelfPermission(this, it) != PackageManager.PERMISSION_GRANTED
        }
        if (needed.isNotEmpty()) {
            ActivityCompat.requestPermissions(this, needed.toTypedArray(), 1001)
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LampApp() {
    val navController = rememberNavController()
    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val scope = rememberCoroutineScope()
    val currentBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = currentBackStackEntry?.destination?.route
    var appMode by remember { mutableStateOf(AppMode.PLAN) }
    val context = LocalContext.current

    CompositionLocalProvider(LocalSyncTrigger provides { SyncWorker.enqueueOneTimeSync(context) }) {
    ModalNavigationDrawer(
        drawerState = drawerState,
        gesturesEnabled = appMode == AppMode.PLAN,
        drawerContent = {
            ModalDrawerSheet(modifier = Modifier.verticalScroll(rememberScrollState())) {
                Text(
                    "Lamp",
                    modifier = Modifier.padding(horizontal = 28.dp, vertical = 24.dp),
                    style = MaterialTheme.typography.titleLarge,
                )
                HorizontalDivider()
                DrawerDestination.entries.forEach { dest ->
                    NavigationDrawerItem(
                        label = { Text(dest.label) },
                        icon = { Icon(dest.icon, contentDescription = dest.label) },
                        selected = currentRoute == dest.route,
                        onClick = {
                            navController.navigate(dest.route) {
                                popUpTo(navController.graph.startDestinationId) { saveState = true }
                                launchSingleTop = true
                                restoreState = true
                            }
                            scope.launch { drawerState.close() }
                        },
                        modifier = Modifier.padding(NavigationDrawerItemDefaults.ItemPadding),
                    )
                }
            }
        },
    ) {
        Scaffold(
            bottomBar = {
                if (appMode == AppMode.DO) {
                    // Simplified bottom bar in Do mode
                    NavigationBar {
                        NavigationBarItem(
                            selected = currentRoute == TopLevelDestination.DO_MODE.route,
                            onClick = {
                                navController.navigate(TopLevelDestination.DO_MODE.route) {
                                    popUpTo(navController.graph.startDestinationId) { saveState = true }
                                    launchSingleTop = true
                                    restoreState = true
                                }
                            },
                            icon = { Icon(TopLevelDestination.DO_MODE.selectedIcon, "Do") },
                            label = { Text("Do") },
                        )
                        NavigationBarItem(
                            selected = false,
                            onClick = {
                                appMode = AppMode.PLAN
                                navController.navigate(TopLevelDestination.DAILY_PLANNING.route) {
                                    popUpTo(navController.graph.startDestinationId) { saveState = true }
                                    launchSingleTop = true
                                    restoreState = true
                                }
                            },
                            icon = { Icon(TopLevelDestination.DAILY_PLANNING.unselectedIcon, "Plan Mode") },
                            label = { Text("Plan Mode") },
                        )
                    }
                } else {
                    NavigationBar {
                        TopLevelDestination.entries.forEach { dest ->
                            NavigationBarItem(
                                selected = currentRoute == dest.route ||
                                    (dest == TopLevelDestination.MORE && DrawerDestination.entries.any { it.route == currentRoute }),
                                onClick = {
                                    if (dest == TopLevelDestination.MORE) {
                                        scope.launch { drawerState.open() }
                                    } else if (dest == TopLevelDestination.DO_MODE) {
                                        appMode = AppMode.DO
                                        navController.navigate(dest.route) {
                                            popUpTo(navController.graph.startDestinationId) { saveState = true }
                                            launchSingleTop = true
                                            restoreState = true
                                        }
                                    } else {
                                        navController.navigate(dest.route) {
                                            popUpTo(navController.graph.startDestinationId) { saveState = true }
                                            launchSingleTop = true
                                            restoreState = true
                                        }
                                    }
                                },
                                icon = {
                                    Icon(
                                        if (currentRoute == dest.route) dest.selectedIcon else dest.unselectedIcon,
                                        contentDescription = dest.label,
                                    )
                                },
                                label = { Text(dest.label) },
                            )
                        }
                    }
                }
            },
        ) { paddingValues ->
            LampNavHost(
                navController = navController,
                modifier = Modifier.padding(paddingValues),
            )
        }
    }
    } // CompositionLocalProvider
}
