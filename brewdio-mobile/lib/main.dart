import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'src/rust/api.dart' as api;
import 'src/rust/frb_generated.dart';
import 'src/pages/home_page.dart';
import 'src/pages/settings_page.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();

  final dir = await getApplicationDocumentsDirectory();
  final dbPath = '${dir.path}/brewdio.db';
  api.initApp(dbPath: dbPath);

  runApp(const ProviderScope(child: BrewdioApp()));
}

class BrewdioApp extends StatelessWidget {
  const BrewdioApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Brewdio',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFFD97706), // amber
          brightness: Brightness.light,
        ),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFFD97706),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      home: const MainShell(),
    );
  }
}

class MainShell extends StatefulWidget {
  const MainShell({super.key});

  @override
  State<MainShell> createState() => _MainShellState();
}

class _MainShellState extends State<MainShell> {
  int _currentIndex = 0;

  static const _pages = <Widget>[
    HomePage(),
    SettingsPage(),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: _pages[_currentIndex],
      bottomNavigationBar: NavigationBar(
        selectedIndex: _currentIndex,
        onDestinationSelected: (i) => setState(() => _currentIndex = i),
        destinations: const [
          NavigationDestination(icon: Icon(Icons.home), label: 'Home'),
          NavigationDestination(icon: Icon(Icons.settings), label: 'Settings'),
        ],
      ),
    );
  }
}
