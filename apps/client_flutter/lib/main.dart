import 'package:flutter/material.dart';

import 'ffi_bridge/generated/frb_generated.dart';
import 'l10n/app_localizations.dart';
import 'screens/workbench_screen.dart';

Future<void> main() async {
  // Initialize the Rust kernel before any bridge call is made.
  var kernelReady = false;
  try {
    await RustLib.init();
    kernelReady = true;
  } catch (error, stackTrace) {
    debugPrint('Rust bridge initialization failed: $error');
    debugPrintStack(stackTrace: stackTrace);
  }
  runApp(CivilEngineeringApp(kernelReady: kernelReady));
}

class CivilEngineeringApp extends StatelessWidget {
  const CivilEngineeringApp({required this.kernelReady, super.key});

  final bool kernelReady;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      onGenerateTitle: (context) => AppLocalizations.of(context)!.appTitle,
      debugShowCheckedModeBanner: false,
      locale: const Locale('ar'),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF1F4E5F)),
        useMaterial3: true,
      ),
      home: WorkbenchScreen(kernelReady: kernelReady),
    );
  }
}
