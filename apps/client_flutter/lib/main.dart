import 'package:flutter/material.dart';

import 'ffi_bridge/generated/api.dart';
import 'ffi_bridge/generated/frb_generated.dart';

Future<void> main() async {
  // Initialize the Rust kernel before any bridge call is made.
  await RustLib.init();
  runApp(const CivilEngineeringApp());
}

class CivilEngineeringApp extends StatelessWidget {
  const CivilEngineeringApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Civil Engineering Platform',
      debugShowCheckedModeBanner: false,
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
    setState(() {
      try {
        _result = getKernelStatus();
        _failed = false;
      } catch (error) {
        _result = 'Bridge error: $error';
        _failed = true;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Civil Engineering Platform'),
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Text(
                'Civil Engineering Platform',
                style: theme.textTheme.headlineSmall,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 8),
              Text(
                'M0 - Rust <-> Flutter bridge proof of concept',
                style: theme.textTheme.bodyMedium
                    ?.copyWith(color: theme.colorScheme.outline),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 32),
              FilledButton(
                onPressed: _testRustKernel,
                child: const Text('Test Rust Kernel'),
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
                      color: _failed ? theme.colorScheme.onErrorContainer : null,
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
