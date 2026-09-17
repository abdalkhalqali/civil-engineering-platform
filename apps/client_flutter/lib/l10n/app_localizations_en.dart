// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'Civil Engineering Platform';

  @override
  String get screenTitle => 'Civil Engineering Platform';

  @override
  String get bridgeDescription =>
      'Flutter to Rust engineering kernel connectivity proof';

  @override
  String get testRustKernel => 'Test Rust Kernel';

  @override
  String get kernelConnected =>
      'Connected to the engineering kernel successfully.';

  @override
  String bridgeError(String error) {
    return 'Bridge error: $error';
  }
}
