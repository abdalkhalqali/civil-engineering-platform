import 'package:flutter/material.dart';

import 'ffi_bridge/generated/api.dart';
import 'ffi_bridge/generated/frb_generated.dart';
import 'l10n/app_localizations.dart';

Future<void> main() async {
  // Initialize the Rust kernel before any bridge call is made.
  try {
    await RustLib.init();
  } catch (error, stackTrace) {
    // Keep the UI available when a platform-specific bridge artifact is
    // unavailable, while reporting the initialization failure explicitly.
    debugPrint('Rust bridge initialization failed: $error');
    debugPrintStack(stackTrace: stackTrace);
  }
  runApp(const CivilEngineeringApp());
}

class CivilEngineeringApp extends StatelessWidget {
  const CivilEngineeringApp({super.key});

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
      home: const KernelBridgeScreen(),
    );
  }
}

/// M0 proof of concept screen: calls one Rust function and shows the result.
class KernelBridgeScreen extends StatefulWidget {
  const KernelBridgeScreen({super.key});

  @override
  State<KernelBridgeScreen> createState() => _KernelBridgeScreenState();
}

class _KernelBridgeScreenState extends State<KernelBridgeScreen> {
  String? _result;
  bool _failed = false;

  void _testRustKernel() {
    final l10n = AppLocalizations.of(context)!;
    setState(() {
      try {
        getKernelStatus();
        _result = l10n.kernelConnected;
        _failed = false;
      } catch (error) {
        _result = l10n.bridgeError(error.toString());
        _failed = true;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;
    final isArabic = Localizations.localeOf(context).languageCode == 'ar';

    return Directionality(
      textDirection: isArabic ? TextDirection.rtl : TextDirection.ltr,
      child: Scaffold(
        appBar: AppBar(
          title: Text(l10n.appTitle),
        ),
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text(
                  l10n.screenTitle,
                  style: theme.textTheme.headlineSmall,
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 8),
                Text(
                  l10n.bridgeDescription,
                  style: theme.textTheme.bodyMedium
                      ?.copyWith(color: theme.colorScheme.outline),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 32),
                FilledButton(
                  onPressed: _testRustKernel,
                  child: Text(l10n.testRustKernel),
                ),
                const SizedBox(height: 32),
                if (_result != null)
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(16),
                    decoration: BoxDecoration(
                      color: _failed
                          ? theme.colorScheme.errorContainer
                          : theme.colorScheme.surfaceContainerHighest,
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: Text(
                      _result!,
                      textAlign: TextAlign.center,
                      style: theme.textTheme.bodyLarge?.copyWith(
                        color:
                            _failed ? theme.colorScheme.onErrorContainer : null,
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
